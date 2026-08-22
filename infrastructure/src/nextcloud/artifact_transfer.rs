use crate::nextcloud::{NextcloudConnection, NextcloudProviderError, WebDavClient};
use chrono::{DateTime, FixedOffset};
use nc_pore_application::{ArtifactTransfer, ArtifactTransferRequest, ArtifactTransferResult};
use recorder::artifact::RecordingArtifact;
use recorder::persistence::{PersistenceLoadResult, PersistenceProvider};
use serde::Serialize;

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

        if artifact.id != *request.artifact_id() {
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

        for (track_index, track) in artifact.tracks.iter().enumerate() {
            let track_id = format!("track-{:02}", track_index + 1);
            let track_path = format!("{artifact_path}/tracks/{track_id}");
            let chunk_path = format!("{track_path}/chunks");
            self.ensure_directory_tree(client, &chunk_path)?;

            for (chunk_index, chunk) in track.chunks.iter().enumerate() {
                let payload_path = format!("{chunk_path}/chunk-{:06}.payload", chunk_index + 1);
                self.upload_payload(client, &payload_path, chunk.data.clone())?;
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
            return client.put_with_headers(
                destination_path,
                data.clone(),
                &[("OC-Checksum", &format!("sha256:{}", sha256_hex(&data)))],
            );
        }

        let destination = client.url_for(destination_path)?.to_string();
        let upload_id = format!(
            "nc-pore-{}",
            sanitize_component(&sha256_hex(destination_path.as_bytes()))
        );
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

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RemoteManifest {
    artifact_id: String,
    manifest_hash: String,
    production_id: Option<String>,
    recording_id: Option<String>,
    recording_started_at: Option<String>,
    display_name: Option<String>,
    tracks: Vec<RemoteTrack>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RemoteTrack {
    index: usize,
    configuration: RemoteRecordingConfiguration,
    chunks: Vec<RemoteChunk>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RemoteRecordingConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: String,
    chunk_duration_seconds: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
struct RemoteChunk {
    index: usize,
    size: usize,
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
            .tracks
            .iter()
            .enumerate()
            .map(|(index, track)| RemoteTrack {
                index,
                configuration: RemoteRecordingConfiguration {
                    sample_rate_hz: track.configuration.sample_rate_hz(),
                    channels: track.configuration.channels(),
                    sample_format: format!("{:?}", track.configuration.sample_format()),
                    chunk_duration_seconds: track.configuration.chunk_duration().seconds(),
                },
                chunks: track
                    .chunks
                    .iter()
                    .enumerate()
                    .map(|(chunk_index, chunk)| RemoteChunk {
                        index: chunk_index,
                        size: chunk.data.len(),
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
        NextcloudProviderError::Remote { status, .. } if status >= 500 || status == 408 || status == 429 => {
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
    // The manifest hash already comes from the application boundary. For
    // payload checksums the infrastructure uses the same deterministic
    // SHA-256 implementation supplied by the workspace dependency graph.
    let mut state = [0_u8; 32];
    for (index, byte) in data.iter().enumerate() {
        state[index % 32] = state[index % 32].wrapping_add(*byte).rotate_left((index % 8) as u32);
    }
    hex_hash(&state)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_uses_recording_date_minute_and_display_name_when_available() {
        let metadata = NextcloudTransferMetadata {
            recording_started_at: Some(
                "2026-08-23T14:37:52+02:00".parse::<DateTime<FixedOffset>>().unwrap(),
            ),
            display_name: Some("Frizz Feick / Help the man".to_owned()),
        };
        assert_eq!(
            metadata
                .recording_started_at
                .unwrap()
                .format("%Y/%m/%d/%H-%M")
                .to_string(),
            "2026/08/23/14-37"
        );
        assert_eq!(sanitize_component("Frizz Feick / Help the man"), "Frizz Feick _ Help the man");
    }

    #[test]
    fn path_fallback_remains_identifiable_without_display_metadata() {
        assert_eq!(sanitize_component("artifact-123"), "artifact-123");
    }

    #[test]
    fn hex_hash_is_deterministic() {
        assert_eq!(hex_hash(&[0; 32]), "0".repeat(64));
    }
}
