#![allow(dead_code)]

//! Filesystem persistence provider.
//!
//! This module provides the concrete local filesystem implementation
//! of the PersistenceProvider boundary.
//!
//! The physical layout follows ADR-055. Actual chunk payload bytes are
//! stored as opaque `.payload` files because this issue does not decide
//! an audio codec or container format.
//!
//! See:
//! - ADR-052 Local Filesystem Persistence Provider
//! - ADR-054 Recording Artifact and Local Recording Data Association
//! - ADR-055 Filesystem Persistence Layout
//! - ADR-058 Recording Payload Representation
//! - ADR-059 Recording Payload Filesystem Persistence

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::artifact::{
    ArtifactStatus, PayloadHash, RecordingArtifact, RecordingChunk, RecordingTrack,
};
use crate::audio::{RecordingChunkDuration, RecordingConfiguration, SampleFormat};
use crate::persistence::{
    PersistenceLoadResult, PersistenceProvider, PersistenceStoreError, artifacts_are_equivalent,
};
use crate::session::RecordingSessionId;

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRecordingArtifact {
    id: String,
    recording_session_id: String,
    #[serde(default)]
    production_id: Option<String>,
    #[serde(default)]
    recording_id: Option<String>,
    status: String,
    tracks: Vec<PersistedRecordingTrack>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRecordingTrack {
    id: String,
    #[serde(default)]
    configuration: Option<PersistedRecordingConfiguration>,
    chunks: Vec<PersistedRecordingChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRecordingConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: PersistedSampleFormat,
    chunk_duration_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
enum PersistedSampleFormat {
    Pcm24,
    F32,
}

impl From<RecordingConfiguration> for PersistedRecordingConfiguration {
    fn from(configuration: RecordingConfiguration) -> Self {
        Self {
            sample_rate_hz: configuration.sample_rate_hz(),
            channels: configuration.channels(),
            sample_format: match configuration.sample_format() {
                SampleFormat::Pcm24 => PersistedSampleFormat::Pcm24,
                SampleFormat::F32 => PersistedSampleFormat::F32,
            },
            chunk_duration_seconds: configuration.chunk_duration().seconds(),
        }
    }
}

impl PersistedRecordingConfiguration {
    fn into_recording_configuration(self) -> Option<RecordingConfiguration> {
        let chunk_duration = match self.chunk_duration_seconds {
            10 => RecordingChunkDuration::TenSeconds,
            30 => RecordingChunkDuration::ThirtySeconds,
            60 => RecordingChunkDuration::OneMinute,
            120 => RecordingChunkDuration::TwoMinutes,
            300 => RecordingChunkDuration::FiveMinutes,
            600 => RecordingChunkDuration::TenMinutes,
            _ => return None,
        };

        let sample_format = match self.sample_format {
            PersistedSampleFormat::Pcm24 => SampleFormat::Pcm24,
            PersistedSampleFormat::F32 => SampleFormat::F32,
        };

        Some(RecordingConfiguration::with_chunk_duration(
            self.sample_rate_hz,
            self.channels,
            sample_format,
            chunk_duration,
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRecordingChunk {
    sequence: u32,
    payload_reference: String,
    payload_size_bytes: u64,
    payload_hash: [u8; 32],
}

impl From<&RecordingArtifact> for PersistedRecordingArtifact {
    fn from(artifact: &RecordingArtifact) -> Self {
        Self {
            id: artifact.id.value().to_string(),
            recording_session_id: artifact.recording_session_id.value().to_string(),
            production_id: artifact.production_id().map(str::to_string),
            recording_id: artifact.recording_id().map(str::to_string),
            status: match artifact.status() {
                ArtifactStatus::Created => "Created".to_string(),
                ArtifactStatus::Available => "Available".to_string(),
                ArtifactStatus::Stored => "Stored".to_string(),
            },
            tracks: artifact
                .tracks()
                .iter()
                .map(|track| PersistedRecordingTrack {
                    id: track.id.value().to_string(),
                    configuration: track.configuration().map(Into::into),
                    chunks: track
                        .chunks()
                        .iter()
                        .map(|chunk| PersistedRecordingChunk {
                            sequence: chunk.sequence,
                            payload_reference: chunk.payload().reference().value().to_string(),
                            payload_size_bytes: chunk.payload().size_bytes(),
                            payload_hash: *chunk.payload().hash().as_bytes(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl PersistedRecordingArtifact {
    fn into_recording_artifact(self, artifact_dir: &Path) -> PersistenceLoadResult {
        let mut artifact =
            RecordingArtifact::new(self.id, RecordingSessionId::new(self.recording_session_id));

        if let (Some(production_id), Some(recording_id)) = (self.production_id, self.recording_id) {
            artifact.set_domain_association(production_id, recording_id);
        }

        for persisted_track in self.tracks {
            let mut track = match persisted_track.configuration {
                Some(configuration) => {
                    let Some(configuration) = configuration.into_recording_configuration() else {
                        return PersistenceLoadResult::Inconsistent;
                    };
                    RecordingTrack::with_configuration(persisted_track.id.clone(), configuration)
                }
                None => RecordingTrack::new(persisted_track.id.clone()),
            };

            for persisted_chunk in persisted_track.chunks {
                let payload_path = FilesystemPersistenceProvider::payload_path(
                    artifact_dir,
                    &persisted_track.id,
                    persisted_chunk.sequence,
                );

                if !payload_path.is_file() {
                    return PersistenceLoadResult::Incomplete;
                }

                let payload = match fs::read(payload_path) {
                    Ok(payload) => payload,
                    Err(_) => return PersistenceLoadResult::Inconsistent,
                };

                if payload.len() as u64 != persisted_chunk.payload_size_bytes {
                    return PersistenceLoadResult::Inconsistent;
                }

                let payload_hash = PayloadHash::from_bytes(&payload);
                if payload_hash.as_bytes() != &persisted_chunk.payload_hash {
                    return PersistenceLoadResult::Inconsistent;
                }

                track.add_chunk(RecordingChunk::with_payload(
                    persisted_chunk.sequence,
                    persisted_chunk.payload_reference,
                    payload,
                ));
            }

            artifact.add_track(track);
        }

        match self.status.as_str() {
            "Available" => artifact.make_available(),
            "Stored" => {
                artifact.make_available();
                artifact.store();
            }
            "Created" => {}
            _ => return PersistenceLoadResult::Inconsistent,
        }

        PersistenceLoadResult::Valid(artifact)
    }
}

/// Filesystem based PersistenceProvider implementation.
///
/// Each RecordingArtifact owns one directory below the persistence root.
pub struct FilesystemPersistenceProvider {
    root: PathBuf,
}

impl FilesystemPersistenceProvider {
    /// Creates a filesystem persistence provider.
    ///
    /// The directory is created if it does not exist.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let root = path.into();
        fs::create_dir_all(&root).expect("failed to create persistence directory");
        Self { root }
    }

    fn validate_id(id: &str) -> bool {
        !id.is_empty() && id != "." && id != ".." && !id.contains('/') && !id.contains('\\')
    }

    fn artifact_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn payload_path(artifact_dir: &Path, track_id: &str, sequence: u32) -> PathBuf {
        artifact_dir
            .join("tracks")
            .join(track_id)
            .join("chunks")
            .join(format!("chunk-{sequence:06}.payload"))
    }

    fn write_artifact(&self, artifact: &RecordingArtifact) {
        assert!(
            Self::validate_id(artifact.id.value()),
            "invalid artifact id"
        );

        for track in artifact.tracks() {
            assert!(Self::validate_id(track.id.value()), "invalid track id");
        }

        if let Some(production_id) = artifact.production_id() {
            assert!(
                Self::validate_id(production_id),
                "invalid production id in artifact association"
            );
        }

        if let Some(recording_id) = artifact.recording_id() {
            assert!(
                Self::validate_id(recording_id),
                "invalid recording id in artifact association"
            );
        }

        let persisted = PersistedRecordingArtifact::from(artifact);
        let artifact_dir = self.artifact_dir(&persisted.id);
        let temp_dir = self.root.join(format!(".{}.tmp", persisted.id));

        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).expect("failed to create temporary artifact directory");

        let content = serde_json::to_string_pretty(&persisted)
            .expect("failed to serialize recording artifact");

        fs::write(temp_dir.join("artifact.json"), content)
            .expect("failed to write artifact metadata");

        for track in artifact.tracks() {
            for chunk in track.chunks() {
                let payload_path = Self::payload_path(&temp_dir, track.id.value(), chunk.sequence);
                if let Some(parent) = payload_path.parent() {
                    fs::create_dir_all(parent).expect("failed to create payload directory");
                }

                let temp_payload = payload_path.with_extension("payload.tmp");
                fs::write(&temp_payload, chunk.payload().data())
                    .expect("failed to write recording payload");
                fs::rename(&temp_payload, &payload_path)
                    .expect("failed to publish recording payload");
            }
        }

        let _ = fs::remove_dir_all(&artifact_dir);
        fs::rename(&temp_dir, &artifact_dir).expect("failed to publish artifact directory");
    }

    fn read_artifact(path: &Path) -> PersistenceLoadResult {
        if !path.is_dir() {
            return PersistenceLoadResult::NotFound;
        }

        let metadata_path = path.join("artifact.json");
        if !metadata_path.is_file() {
            return PersistenceLoadResult::Incomplete;
        }

        let content = match fs::read_to_string(metadata_path) {
            Ok(content) => content,
            Err(_) => return PersistenceLoadResult::Inconsistent,
        };

        let persisted: PersistedRecordingArtifact = match serde_json::from_str(&content) {
            Ok(persisted) => persisted,
            Err(_) => return PersistenceLoadResult::Inconsistent,
        };

        persisted.into_recording_artifact(path)
    }
}

impl PersistenceProvider for FilesystemPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.write_artifact(&artifact);
    }

    fn store_checked(
        &mut self,
        mut artifact: RecordingArtifact,
    ) -> Result<RecordingArtifact, PersistenceStoreError> {
        match self.load(artifact.id.value()) {
            PersistenceLoadResult::NotFound => {
                artifact.store();
                self.write_artifact(&artifact);
                Ok(artifact)
            }
            PersistenceLoadResult::Valid(existing) => {
                if artifacts_are_equivalent(&existing, &artifact) {
                    Ok(existing)
                } else {
                    Err(PersistenceStoreError::Conflict {
                        artifact_id: artifact.id.value().to_owned(),
                    })
                }
            }
            PersistenceLoadResult::Incomplete => Err(PersistenceStoreError::Io(format!(
                "cannot persist artifact {}: existing persisted representation is incomplete",
                artifact.id.value()
            ))),
            PersistenceLoadResult::Inconsistent => Err(PersistenceStoreError::Io(format!(
                "cannot persist artifact {}: existing persisted representation is inconsistent",
                artifact.id.value()
            ))),
        }
    }

    fn load(&self, id: &str) -> PersistenceLoadResult {
        if !Self::validate_id(id) {
            return PersistenceLoadResult::Inconsistent;
        }

        Self::read_artifact(&self.artifact_dir(id))
    }

    fn list_ids(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;

                if name.starts_with('.') || !Self::validate_id(name) {
                    return None;
                }

                Some(name.to_owned())
            })
            .collect()
    }

    fn list(&self) -> Vec<RecordingArtifact> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;

                if name.starts_with('.') || !Self::validate_id(name) {
                    return None;
                }

                match Self::read_artifact(&entry.path()) {
                    PersistenceLoadResult::Valid(artifact) => Some(artifact),
                    PersistenceLoadResult::Incomplete
                    | PersistenceLoadResult::Inconsistent
                    | PersistenceLoadResult::NotFound => None,
                }
            })
            .collect()
    }

    fn remove(&mut self, id: &str) {
        if !Self::validate_id(id) {
            return;
        }

        let _ = fs::remove_dir_all(self.artifact_dir(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("nc-pore-{name}"));
        let _ = fs::remove_dir_all(&path);
        path
    }

    fn test_artifact() -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.set_domain_association("production-001", "recording-017");

        let mut host = RecordingTrack::new("track-host");
        host.add_chunk(RecordingChunk::with_payload(
            1,
            "track-host/chunk-000001",
            vec![1, 2, 3],
        ));
        host.add_chunk(RecordingChunk::with_payload(
            2,
            "track-host/chunk-000002",
            vec![4, 5],
        ));
        artifact.add_track(host);
        artifact.make_available();
        artifact.store();

        artifact
    }

    fn configured_test_artifact() -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new("artifact-configured", RecordingSessionId::new("session-002"));
        let configuration = RecordingConfiguration::with_chunk_duration(
            48_000,
            2,
            SampleFormat::Pcm24,
            RecordingChunkDuration::ThirtySeconds,
        );
        let mut track = RecordingTrack::with_configuration("track-configured", configuration);
        track.add_chunk(RecordingChunk::with_sample_offset(
            1,
            48_000,
            "track-configured/chunk-000001",
            vec![1, 2, 3],
        ));
        artifact.add_track(track);
        artifact.make_available();
        artifact.store();
        artifact
    }

    // TEST-16
    // Protects ADR-055: artifacts are stored below their own directory.
    #[test]
    fn test_16_filesystem_provider_can_store_artifact() {
        let path = test_directory("test-16");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());

        assert!(path.join("artifact-001/artifact.json").is_file());
        assert!(
            path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload")
                .is_file()
        );
        assert!(
            path.join("artifact-001/tracks/track-host/chunks/chunk-000002.payload")
                .is_file()
        );

        let _ = fs::remove_dir_all(path);
    }

    // TEST-17
    // Protects ADR-054/055 and ADR-059: persisted tracks, chunks,
    // payloads and the domain association can be restored.
    #[test]
    fn test_17_filesystem_provider_can_load_artifact_and_payload() {
        let path = test_directory("test-17");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        let result = provider.load("artifact-001");
        let PersistenceLoadResult::Valid(artifact) = result else {
            panic!("expected valid persisted artifact");
        };

        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks().len(), 2);
        assert_eq!(
            artifact.tracks()[0].chunks()[0].payload().data(),
            &[1, 2, 3]
        );
        assert_eq!(artifact.tracks()[0].chunks()[1].payload().data(), &[4, 5]);

        let _ = fs::remove_dir_all(path);
    }

    fn assert_test_directory_removed(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn configured_track_configuration_survives_filesystem_roundtrip() {
        let path = test_directory("configured-track-roundtrip");
        let mut provider = FilesystemPersistenceProvider::new(&path);
        let artifact = configured_test_artifact();
        let expected_manifest = artifact.manifest_hash();

        provider.store(artifact);
        let PersistenceLoadResult::Valid(restored) = provider.load("artifact-configured") else {
            panic!("expected configured artifact to round-trip as valid");
        };

        assert_eq!(
            restored.tracks()[0].configuration(),
            Some(RecordingConfiguration::with_chunk_duration(
                48_000,
                2,
                SampleFormat::Pcm24,
                RecordingChunkDuration::ThirtySeconds,
            ))
        );
        assert_eq!(restored.manifest_hash(), expected_manifest);

        assert_test_directory_removed(&path);
    }

    // TEST-18
    // Protects ADR-053/055: incomplete temporary directories are ignored.
    #[test]
    fn test_18_filesystem_provider_can_list_artifacts() {
        let path = test_directory("test-18");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::create_dir_all(path.join(".artifact-002.tmp")).unwrap();
        fs::write(path.join(".artifact-002.tmp/partial"), "incomplete").unwrap();

        assert_eq!(provider.list().len(), 1);

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn provider_can_list_artifact_ids() {
        let path = test_directory("list-ids");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::create_dir_all(path.join(".artifact-002.tmp")).unwrap();
        fs::create_dir_all(path.join("artifact-003")).unwrap();

        let ids = provider.list_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"artifact-001".to_owned()));
        assert!(ids.contains(&"artifact-003".to_owned()));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-19
    // Protects ADR-055: removal removes the complete artifact directory.
    #[test]
    fn test_19_filesystem_provider_can_remove_artifact() {
        let path = test_directory("test-19");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        provider.remove("artifact-001");

        assert!(!path.join("artifact-001").exists());
        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::NotFound
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-37
    // Protects ADR-059 and the persistence assessment boundary:
    // a missing payload is incomplete persisted data.
    #[test]
    fn missing_payload_is_incomplete() {
        let path = test_directory("missing-payload");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::remove_file(path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload"))
            .unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Incomplete
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-38
    // Protects the persistence assessment boundary:
    // payload size disagreement means persisted metadata and payload
    // disagree and therefore the artifact is inconsistent.
    #[test]
    fn payload_size_mismatch_is_inconsistent() {
        let path = test_directory("payload-size-mismatch");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());

        let metadata_path = path.join("artifact-001/artifact.json");
        let content = fs::read_to_string(&metadata_path).unwrap();
        let content = content.replace("\"payload_size_bytes\": 3", "\"payload_size_bytes\": 4");
        fs::write(metadata_path, content).unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-43
    // Protects the persistence integrity boundary:
    // payload corruption is detected even when the payload size is unchanged.
    #[test]
    fn payload_content_mismatch_is_inconsistent() {
        let path = test_directory("payload-content-mismatch");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::write(
            path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload"),
            [9, 8, 7],
        )
        .unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-44
    // Protects the persistence integrity boundary:
    // changing the persisted expected hash is detected independently of
    // the payload bytes themselves.
    #[test]
    fn payload_hash_mismatch_is_inconsistent() {
        let path = test_directory("payload-hash-mismatch");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());

        let metadata_path = path.join("artifact-001/artifact.json");
        let content = fs::read_to_string(&metadata_path).unwrap();
        let content = content.replacen("\"payload_hash\": [", "\"payload_hash\": [0,", 1);
        fs::write(metadata_path, content).unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-39
    // Protects the persistence assessment boundary:
    // malformed metadata is inconsistent persisted data.
    #[test]
    fn malformed_metadata_is_inconsistent() {
        let path = test_directory("malformed-metadata");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::write(path.join("artifact-001/artifact.json"), "not-json").unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-41
    // Protects ADR-060:
    // equivalent persisted content is an idempotent success and returns
    // the already Stored artifact rather than creating another artifact.
    #[test]
    fn store_checked_is_idempotent_for_equivalent_artifact() {
        let path = test_directory("store-checked-idempotent");
        let mut provider = FilesystemPersistenceProvider::new(&path);
        let artifact = test_artifact();

        let first = provider
            .store_checked(artifact.clone())
            .expect("first store should succeed");
        let second = provider
            .store_checked(artifact)
            .expect("equivalent store should be idempotent");

        assert_eq!(first.status(), &ArtifactStatus::Stored);
        assert_eq!(second.status(), &ArtifactStatus::Stored);
        assert_eq!(provider.list().len(), 1);

        let _ = fs::remove_dir_all(path);
    }

    // TEST-42
    // Protects ADR-060:
    // a reused artifact identity with different persisted content is a
    // conflict and must not replace the existing artifact.
    #[test]
    fn store_checked_rejects_conflicting_artifact() {
        let path = test_directory("store-checked-conflict");
        let mut provider = FilesystemPersistenceProvider::new(&path);
        let first = test_artifact();
        let conflicting =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-conflict"));

        provider
            .store_checked(first)
            .expect("first store should succeed");

        assert!(matches!(
            provider.store_checked(conflicting),
            Err(PersistenceStoreError::Conflict { artifact_id }) if artifact_id == "artifact-001"
        ));

        assert_eq!(provider.list().len(), 1);

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn store_checked_rejects_existing_incomplete_artifact() {
        let path = test_directory("store-checked-incomplete");
        let mut provider = FilesystemPersistenceProvider::new(&path);
        fs::create_dir_all(path.join("artifact-001")).unwrap();

        assert!(matches!(
            provider.store_checked(test_artifact()),
            Err(PersistenceStoreError::Io(message))
                if message.contains("existing persisted representation is incomplete")
        ));

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn invalid_ids_cannot_escape_persistence_root() {
        let path = test_directory("invalid-id");
        let provider = FilesystemPersistenceProvider::new(&path);

        assert!(matches!(
            provider.load("../outside"),
            PersistenceLoadResult::Inconsistent
        ));
        assert!(matches!(
            provider.load("nested/id"),
            PersistenceLoadResult::Inconsistent
        ));
        assert!(matches!(
            provider.load(r"nested\id"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }
}
