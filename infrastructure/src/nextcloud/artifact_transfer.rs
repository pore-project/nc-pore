use crate::nextcloud::{NextcloudConnection, NextcloudProviderError, WebDavClient};
use chrono::{DateTime, FixedOffset};
use nc_pore_application::{ArtifactTransfer, ArtifactTransferRequest, ArtifactTransferResult};
use recorder::artifact::RecordingArtifact;
use recorder::persistence::{PersistenceLoadResult, PersistenceProvider};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const LARGE_FILE_THRESHOLD: usize = 10 * 1024 * 1024;
const CHUNK_SIZE: usize = 10 * 1024 * 1024;

/// Human-readable information supplied to a provider when available.
///
/// The provider decides how this information is represented remotely. It is
/// intentionally not part of the Nextcloud remote artifact identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NextcloudTransferMetadata {
    pub recording_started_at: Option<DateTime<FixedOffset>>,
    pub display_name: Option<String>,
}

/// Transfers complete local recording artifacts into a Nextcloud account.
///
/// The local persistence provider remains the source of the artifact payload;
/// Nextcloud is only responsible for the remote representation and transfer.
pub struct NextcloudArtifactTransfer<P> {
    connection: NextcloudConnection,
    persistence: P,
}

impl<P> NextcloudArtifactTransfer<P>
where
    P: PersistenceProvider,
{
    pub fn new(connection: NextcloudConnection, persistence: P) -> Self {
        Self {
            connection,
            persistence,
        }
    }

    pub fn transfer_with_metadata(
        &mut self,
        request: &ArtifactTransferRequest,
        metadata: &NextcloudTransferMetadata,
    ) -> ArtifactTransferResult {
        let artifact = match self.persistence.load(request.artifact_id().value()) {
            PersistenceLoadResult::Valid(artifact) => artifact,
            PersistenceLoadResult::NotFound => {
                return ArtifactTransferResult::PermanentFailure {
                    reason: format!("local artifact {} was not found", request.artifact_id().value()),
                };
            }
            PersistenceLoadResult::Incomplete => {
                return ArtifactTransferResult::IntegrityFailure {
                    reason: format!("local artifact {} is incomplete", request.artifact_id().value()),
                };
            }
            PersistenceLoadResult::Inconsistent => {
                return ArtifactTransferResult::IntegrityFailure {
                    reason: format!("local artifact {} is inconsistent", request.artifact_id().value()),
                };
            }
        };

        if artifact.id.value() != request.artifact_id().value() {
            return ArtifactTransferResult::IntegrityFailure {
                reason: "loaded artifact identity does not match synchronization request".into(),
            };
        }

        let client = match self.connection.client() {
            Ok(client) => client,
            Err(error) => return map_provider_error(error),
        };

        match self.transfer_artifact(&client, &artifact, request, metadata) {
            Ok(()) => ArtifactTransferResult::Succeeded,
            Err(error) => map_provider_error(error),
        }
    }

    fn transfer_artifact<T>(
        &self,
        client: &WebDavClient<T>,
        artifact: &RecordingArtifact,
        request: &ArtifactTransferRequest,
        metadata: &NextcloudTransferMetadata,
    ) -> Result<(), NextcloudProviderError>
    where
        T: crate::nextcloud::WebDavTransport,
    {
        let artifact_path = self.artifact_path(artifact, metadata)?;
        let manifest_path = format!("{artifact_path}/manifest.json");

        if let Some(remote_manifest) = client.get_optional(&manifest_path)? {
            let remote: RemoteManifest = serde_json::from_slice(&remote_manifest.body).map_err(|error| {
                NextcloudProviderError::InvalidConfiguration(format!(
                    "remote artifact manifest is invalid: {error}"
                ))
            })?;
            if remote.artifact_id == artifact.id.value()
                && remote.manifest_hash == hex_hash(request.manifest_hash())
            {
                return Ok(());
            }
            return Err(NextcloudProviderError::Remote {
                status: 409,
                operation: "artifact conflict",
            });
        }

        self.ensure_directory_tree(client, &artifact_path)?;

        let manifest = build_manifest(artifact, request.manifest_hash(), metadata);
        let manifest_body = serde_json::to_vec_pretty(&manifest).map_err(|error| {
            NextcloudProviderError::InvalidConfiguration(format!("manifest serialization failed: {error}"))
        })?;

        for (track_index, track) in artifact.tracks().iter().enumerate() {
            let track_id = sanitize_component(track.id.value());
            let track_path = format!("{artifact_path}/tracks/track-{:02}-{track_id}", track_index + 1);
            let chunk_path = format!("{track_path}/chunks");
            self.ensure_directory_tree(client, &chunk_path)?;

            for (chunk_index, chunk) in track.chunks().iter().enumerate() {
                let payload_path = format!("{chunk_path}/chunk-{:06}.payload", chunk_index + 1);
                self.upload_payload(client, &payload_path, chunk.payload().data().to_vec())?;
            }
        }

        client.put_with_headers(
            &manifest_path,
            manifest_body,
            &[("Content-Type", "application/json")],
        )?;

        let Some(remote_manifest) = client.get_optional(&manifest_path)? else {
            return Err(NextcloudProviderError::Remote {
                status: 404,
                operation: "manifest verification",
            });
        };
        let verified: RemoteManifest = serde_json::from_slice(&remote_manifest.body).map_err(|error| {
            NextcloudProviderError::InvalidConfiguration(format!(
                "uploaded artifact manifest is invalid: {error}"
            ))
        })?;
        if verified.artifact_id != artifact.id.value()
            || verified.manifest_hash != hex_hash(request.manifest_hash())
        {
            return Err(NextcloudProviderError::Remote {
                status: 409,
                operation: "manifest verification",
            });
        }

        Ok(())
    }

    fn artifact_path(
        &self,
        artifact: &RecordingArtifact,
        metadata: &NextcloudTransferMetadata,
    ) -> Result<String, NextcloudProviderError> {
        let root = self.connection.config().remote_root().trim_matches('/');
        let date_path = metadata
            .recording_started_at
            .map(|timestamp| timestamp.format("%Y/%m/%d").to_string())
            .unwrap_or_else(|| "undated".to_owned());
        let minute = metadata
            .recording_started_at
            .map(|timestamp| timestamp.format("%H-%M").to_string());
        let display_name = metadata
            .display_name
            .as_deref()
            .map(sanitize_component)
            .filter(|value| !value.is_empty());
        let artifact_id = sanitize_component(artifact.id.value());

        let folder_name = match (minute, display_name) {
            (Some(minute), Some(name)) => format!("{minute} - {name} - {artifact_id}"),
            (Some(minute), None) => format!("{minute} - {artifact_id}"),
            (None, Some(name)) => format!("{name} - {artifact_id}"),
            (None, None) => artifact_id,
        };

        Ok(format!(
            "remote.php/dav/files/{}/{root}/{date_path}/{folder_name}",
            self.connection.config().username()
        ))
    }

    fn ensure_directory_tree<T>(
        &self,
        client: &WebDavClient<T>,
        path: &str,
    ) -> Result<(), NextcloudProviderError>
    where
        T: crate::nextcloud::WebDavTransport,
    {
        let mut current = String::new();
        for component in path.split('/') {
            if component.is_empty() {
                continue;
            }
            if !current.is_empty() {
                current.push('/');
            }
            current.push_str(component);
            match client.mkcol(&current) {
                Ok(()) => {}
                Err(NextcloudProviderError::Remote { status: 405, .. }) => {}
                Err(error) => return Err(error),
            }
        }
        Ok(())
    }

    fn upload_payload<T>(
        &self,
        client: &WebDavClient<T>,
        destination_path: &str,
        data: Vec<u8>,
    ) -> Result<(), NextcloudProviderError>
    where
        T: crate::nextcloud::WebDavTransport,
    {
        if data.len() < LARGE_FILE_THRESHOLD {
            let checksum = sha256_hex(&data);
            return client.put_with_headers(
                destination_path,
                data,
                &[("OC-Checksum", &format!("sha256:{checksum}"))],
            );
        }

        let destination = client.url_for(destination_path)?.to_string();
        let upload_id = format!("nc-pore-{}", sha256_hex(destination_path.as_bytes()));
        let upload_root = format!(
            "remote.php/dav/uploads/{}/{upload_id}",
            self.connection.config().username()
        );
        match client.mkcol_with_headers(&upload_root, &[("Destination", &destination)]) {
            Ok(()) => {}
            Err(NextcloudProviderError::Remote { status: 405, .. }) => {}
            Err(error) => return Err(error),
        }

        for (index, chunk) in data.chunks(CHUNK_SIZE).enumerate() {
            let chunk_number = index + 1;
            if chunk_number > 10_000 {
                return Err(NextcloudProviderError::InvalidConfiguration(
                    "Nextcloud chunked upload supports at most 10000 chunks".into(),
                ));
            }
            let chunk_path = format!("{upload_root}/{chunk_number:05}");
            let total_length = data.len().to_string();
            client.put_with_headers(
                &chunk_path,
                chunk.to_vec(),
                &[
                    ("Destination", &destination),
                    ("OC-Total-Length", &total_length),
                ],
            )?;
        }

        let source = format!("{upload_root}/.file");
        let total_length = data.len().to_string();
        client.move_with_headers(
            &source,
            &[
                ("Destination", &destination),
                ("OC-Total-Length", &total_length),
            ],
        )
    }
}

impl<P> ArtifactTransfer for NextcloudArtifactTransfer<P>
where
    P: PersistenceProvider,
{
    fn transfer(&mut self, request: &ArtifactTransferRequest) -> ArtifactTransferResult {
        self.transfer_with_metadata(request, &NextcloudTransferMetadata::default())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RemoteManifest {
    artifact_id: String,
    manifest_hash: String,
    production_id: Option<String>,
    recording_id: Option<String>,
    recording_started_at: Option<String>,
    display_name: Option<String>,
    tracks: Vec<RemoteTrack>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RemoteTrack {
    index: usize,
    track_id: String,
    configuration: Option<RemoteRecordingConfiguration>,
    chunks: Vec<RemoteChunk>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RemoteRecordingConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: String,
    chunk_duration_seconds: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
struct RemoteChunk {
    index: usize,
    sequence: u32,
    sample_offset: u64,
    reference: String,
    size: u64,
    sha256: String,
}

fn build_manifest(
    artifact: &RecordingArtifact,
    manifest_hash: &[u8; 32],
    metadata: &NextcloudTransferMetadata,
) -> RemoteManifest {
    RemoteManifest {
        artifact_id: artifact.id.value().to_owned(),
        manifest_hash: hex_hash(manifest_hash),
        production_id: artifact.production_id().map(str::to_owned),
        recording_id: artifact.recording_id().map(str::to_owned),
        recording_started_at: metadata.recording_started_at.map(|value| value.to_rfc3339()),
        display_name: metadata.display_name.clone(),
        tracks: artifact
            .tracks()
            .iter()
            .enumerate()
            .map(|(index, track)| RemoteTrack {
                index,
                track_id: track.id.value().to_owned(),
                configuration: track.configuration().map(|configuration| {
                    RemoteRecordingConfiguration {
                        sample_rate_hz: configuration.sample_rate_hz(),
                        channels: configuration.channels(),
                        sample_format: format!("{:?}", configuration.sample_format()),
                        chunk_duration_seconds: configuration.chunk_duration().seconds(),
                    }
                }),
                chunks: track
                    .chunks()
                    .iter()
                    .map(|chunk| RemoteChunk {
                        index: chunk.sequence as usize,
                        sequence: chunk.sequence,
                        sample_offset: chunk.sample_offset(),
                        reference: chunk.payload().reference().value().to_owned(),
                        size: chunk.payload().size_bytes(),
                        sha256: hex_hash(chunk.payload().hash().as_bytes()),
                    })
                    .collect(),
            })
            .collect(),
    }
}

fn map_provider_error(error: NextcloudProviderError) -> ArtifactTransferResult {
    match error {
        NextcloudProviderError::Authentication
        | NextcloudProviderError::InvalidConfiguration(_) => ArtifactTransferResult::PermanentFailure {
            reason: error.to_string(),
        },
        NextcloudProviderError::Remote { status, .. } if status == 409 => {
            ArtifactTransferResult::Conflict {
                reason: error.to_string(),
            }
        }
        NextcloudProviderError::Remote { status, .. }
            if status >= 500 || status == 408 || status == 429 =>
        {
            ArtifactTransferResult::RetryableFailure {
                reason: error.to_string(),
                continuation: None,
            }
        }
        NextcloudProviderError::Transport(_) => ArtifactTransferResult::RetryableFailure {
            reason: error.to_string(),
            continuation: None,
        },
        NextcloudProviderError::Remote { .. } => ArtifactTransferResult::PermanentFailure {
            reason: error.to_string(),
        },
    }
}

fn sanitize_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, ' ' | '-' | '_' | '.') {
                character
            } else {
                '_'
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches([' ', '.'])
        .to_owned()
}

fn hex_hash(value: &[u8; 32]) -> String {
    value.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex_hash(&hasher.finalize().into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_path_uses_date_and_minute() {
        let timestamp = "2026-08-23T14:37:52+02:00"
            .parse::<DateTime<FixedOffset>>()
            .unwrap();
        assert_eq!(
            timestamp.format("%Y/%m/%d/%H-%M").to_string(),
            "2026/08/23/14-37"
        );
    }

    #[test]
    fn display_name_is_sanitized_without_losing_human_readability() {
        assert_eq!(
            sanitize_component("Frizz Feick / Help the man"),
            "Frizz Feick _ Help the man"
        );
    }

    #[test]
    fn fallback_name_remains_identifiable() {
        assert_eq!(sanitize_component("artifact-123"), "artifact-123");
    }

    #[test]
    fn sha256_is_real_and_deterministic() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
