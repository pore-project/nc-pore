use crate::nextcloud::{NextcloudConnection, NextcloudProviderError, WebDavClient};
use chrono::{DateTime, Utc};
use recorder::completion::PreparedUpload;
use recorder::remote::{
    RemoteArtifactMetadata, RemoteArtifactUploader, RemoteUploadReceipt, RemoteUploadTrackReceipt,
};
use serde::{Deserialize, Serialize};

const LARGE_FILE_THRESHOLD: usize = 10 * 1024 * 1024;
const CHUNK_SIZE: usize = 10 * 1024 * 1024;

/// Concrete Nextcloud adapter for the provider-neutral prepared-upload boundary.
///
/// The configured remote root is the only host-owned part of the path. PoRe
/// owns the complete hierarchy below it and uses the recording start time for
/// chronological placement.
pub struct NextcloudArtifactUploader {
    connection: NextcloudConnection,
    metadata: RemoteArtifactMetadata,
}

impl NextcloudArtifactUploader {
    pub fn new(connection: NextcloudConnection, metadata: RemoteArtifactMetadata) -> Self {
        Self {
            connection,
            metadata,
        }
    }

    pub fn connection(&self) -> &NextcloudConnection {
        &self.connection
    }

    pub fn metadata(&self) -> &RemoteArtifactMetadata {
        &self.metadata
    }

    pub fn upload_with_client<T: crate::nextcloud::WebDavTransport>(
        &mut self,
        client: &WebDavClient<T>,
        upload: &PreparedUpload,
    ) -> Result<RemoteUploadReceipt, NextcloudProviderError> {
        let artifact_path = self.artifact_path(upload.artifact_id())?;
        let manifest_path = format!("{artifact_path}/manifest.json");
        self.ensure_directory_tree(client, &artifact_path)?;

        for (index, track) in upload.tracks().iter().enumerate() {
            let track_path = format!(
                "{artifact_path}/tracks/track-{:02}-{}.flac",
                index + 1,
                sanitize_component(track.track_id())
            );
            self.upload_payload(client, &track_path, track.data().to_vec())?;
        }

        let manifest = RemoteManifest::from_upload(upload, &self.metadata);
        let manifest_body = serde_json::to_vec_pretty(&manifest).map_err(|error| {
            NextcloudProviderError::InvalidConfiguration(format!(
                "manifest serialization failed: {error}"
            ))
        })?;
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
        let verified: RemoteManifest =
            serde_json::from_slice(&remote_manifest.body).map_err(|error| {
                NextcloudProviderError::InvalidConfiguration(format!(
                    "uploaded artifact manifest is invalid: {error}"
                ))
            })?;
        if verified != manifest {
            return Err(NextcloudProviderError::Remote {
                status: 409,
                operation: "manifest verification",
            });
        }

        let mut receipts = Vec::with_capacity(upload.tracks().len());
        for (index, track) in upload.tracks().iter().enumerate() {
            let track_path = format!(
                "{artifact_path}/tracks/track-{:02}-{}.flac",
                index + 1,
                sanitize_component(track.track_id())
            );
            let Some(remote_track) = client.get_optional(&track_path)? else {
                return Err(NextcloudProviderError::Remote {
                    status: 404,
                    operation: "track verification",
                });
            };
            let hash = recorder::artifact::PayloadHash::from_bytes(&remote_track.body);
            if remote_track.body.len() as u64 != track.size_bytes() || hash != track.hash() {
                return Err(NextcloudProviderError::Remote {
                    status: 409,
                    operation: "track verification",
                });
            }
            receipts.push(RemoteUploadTrackReceipt::new(
                track.track_id(),
                remote_track.body.len() as u64,
                hash,
            ));
        }

        Ok(RemoteUploadReceipt::new(
            upload.artifact_id(),
            upload.manifest_hash(),
            receipts,
        ))
    }

    fn artifact_path(&self, artifact_id: &str) -> Result<String, NextcloudProviderError> {
        let root = self.connection.config().remote_root().trim_matches('/');
        let started_at: DateTime<Utc> = self.metadata.recording_started_at().into();
        let date_path = started_at.format("%Y/%m/%d");
        let minute = started_at.format("%H-%M");
        let display_name = self
            .metadata
            .display_name()
            .map(sanitize_component)
            .filter(|value| !value.is_empty());
        let artifact_id = sanitize_component(artifact_id);
        if artifact_id.is_empty() {
            return Err(NextcloudProviderError::InvalidConfiguration(
                "artifact id must not be empty".into(),
            ));
        }

        let folder_name = match display_name {
            Some(name) => format!("{minute} - {name} - {artifact_id}"),
            None => format!("{minute} - {artifact_id}"),
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
            let checksum = hex_hash(recorder::artifact::PayloadHash::from_bytes(&data).as_bytes());
            return client.put_with_headers(
                destination_path,
                data,
                &[("OC-Checksum", &format!("sha256:{checksum}"))],
            );
        }

        let destination = client.url_for(destination_path)?.to_string();
        let upload_id = format!(
            "nc-pore-{}",
            hex_hash(
                recorder::artifact::PayloadHash::from_bytes(destination_path.as_bytes()).as_bytes(),
            )
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

impl RemoteArtifactUploader for NextcloudArtifactUploader {
    type Error = NextcloudProviderError;

    fn upload(&mut self, upload: &PreparedUpload) -> Result<RemoteUploadReceipt, Self::Error> {
        let connection = self.connection.clone();
        let client = connection.client()?;
        self.upload_with_client(&client, upload)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RemoteManifest {
    artifact_id: String,
    manifest_hash: String,
    recording_started_at: String,
    display_name: Option<String>,
    tracks: Vec<RemoteManifestTrack>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct RemoteManifestTrack {
    index: usize,
    track_id: String,
    size_bytes: u64,
    sha256: String,
}

impl RemoteManifest {
    fn from_upload(upload: &PreparedUpload, metadata: &RemoteArtifactMetadata) -> Self {
        let started_at: DateTime<Utc> = metadata.recording_started_at().into();
        Self {
            artifact_id: upload.artifact_id().to_owned(),
            manifest_hash: hex_hash(upload.manifest_hash().as_bytes()),
            recording_started_at: started_at.to_rfc3339(),
            display_name: metadata.display_name().map(str::to_owned),
            tracks: upload
                .tracks()
                .iter()
                .enumerate()
                .map(|(index, track)| RemoteManifestTrack {
                    index,
                    track_id: track.track_id().to_owned(),
                    size_bytes: track.size_bytes(),
                    sha256: hex_hash(track.hash().as_bytes()),
                })
                .collect(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use recorder::artifact::PayloadHash;
    use recorder::remote::RemoteUploadTrackReceipt;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    #[test]
    fn path_uses_recording_start_and_default_audio_root() {
        let start = UNIX_EPOCH + Duration::from_secs(1_756_000_000);
        let metadata = RemoteArtifactMetadata::new(start, Some("Interview Frizz Feick".into()));
        let connection =
            NextcloudConnection::new(crate::nextcloud::NextcloudConnectionConfig::new(
                "https://cloud.example.test",
                crate::nextcloud::NextcloudCredentials::new("host-user", "password"),
            ))
            .unwrap();
        let uploader = NextcloudArtifactUploader::new(connection, metadata);
        let path = uploader.artifact_path("artifact-123").unwrap();
        assert!(path.starts_with("remote.php/dav/files/host-user/audio/"));
        assert!(path.ends_with(" - Interview Frizz Feick - artifact-123"));
    }

    #[test]
    fn display_name_is_sanitized_for_path() {
        let metadata = RemoteArtifactMetadata::new(
            SystemTime::UNIX_EPOCH,
            Some("Frizz Feick / Help the man".into()),
        );
        let connection =
            NextcloudConnection::new(crate::nextcloud::NextcloudConnectionConfig::new(
                "https://cloud.example.test",
                crate::nextcloud::NextcloudCredentials::new("user", "password"),
            ))
            .unwrap();
        let uploader = NextcloudArtifactUploader::new(connection, metadata);
        let path = uploader.artifact_path("artifact-123").unwrap();
        assert!(path.ends_with(" - Frizz Feick _ Help the man - artifact-123"));
    }

    #[test]
    fn receipt_hash_type_remains_sha256() {
        let hash = PayloadHash::from_bytes(b"abc");
        assert_eq!(hash, PayloadHash::from_bytes(b"abc"));
        let receipt = RemoteUploadTrackReceipt::new("track-a", 3, hash);
        assert_eq!(receipt.size_bytes(), 3);
    }
}
