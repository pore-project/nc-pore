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

use crate::artifact::{ArtifactStatus, RecordingArtifact, RecordingChunk, RecordingTrack};
use crate::persistence::PersistenceProvider;
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
    chunks: Vec<PersistedRecordingChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedRecordingChunk {
    sequence: u32,
    payload_reference: String,
    payload_size_bytes: u64,
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
                    chunks: track
                        .chunks()
                        .iter()
                        .map(|chunk| PersistedRecordingChunk {
                            sequence: chunk.sequence,
                            payload_reference: chunk.payload().reference().value().to_string(),
                            payload_size_bytes: chunk.payload().size_bytes(),
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl PersistedRecordingArtifact {
    fn into_recording_artifact(self, artifact_dir: &Path) -> Option<RecordingArtifact> {
        let mut artifact =
            RecordingArtifact::new(self.id, RecordingSessionId::new(self.recording_session_id));

        if let (Some(production_id), Some(recording_id)) = (self.production_id, self.recording_id) {
            artifact.set_domain_association(production_id, recording_id);
        }

        for persisted_track in self.tracks {
            let mut track = RecordingTrack::new(persisted_track.id.clone());

            for persisted_chunk in persisted_track.chunks {
                let payload_path = FilesystemPersistenceProvider::payload_path(
                    artifact_dir,
                    &persisted_track.id,
                    persisted_chunk.sequence,
                );
                let payload = fs::read(payload_path).ok()?;

                if payload.len() as u64 != persisted_chunk.payload_size_bytes {
                    return None;
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
            _ => return None,
        }

        Some(artifact)
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

    fn read_artifact(path: &Path) -> Option<RecordingArtifact> {
        let metadata_path = path.join("artifact.json");
        let content = fs::read_to_string(metadata_path).ok()?;
        let persisted: PersistedRecordingArtifact = serde_json::from_str(&content).ok()?;
        persisted.into_recording_artifact(path)
    }
}

impl PersistenceProvider for FilesystemPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.write_artifact(&artifact);
    }

    fn load(&self, id: &str) -> Option<RecordingArtifact> {
        if !Self::validate_id(id) {
            return None;
        }

        Self::read_artifact(&self.artifact_dir(id))
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

                Self::read_artifact(&entry.path())
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
        let artifact = provider.load("artifact-001").expect("artifact missing");

        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks().len(), 2);
        assert_eq!(
            artifact.tracks()[0].chunks()[0].payload().data(),
            &[1, 2, 3]
        );
        assert_eq!(artifact.tracks()[1].chunks()[1].payload().data(), &[4, 5]);

        let _ = fs::remove_dir_all(path);
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

    // TEST-19
    // Protects ADR-055: removal removes the complete artifact directory.
    #[test]
    fn test_19_filesystem_provider_can_remove_artifact() {
        let path = test_directory("test-19");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        provider.remove("artifact-001");

        assert!(!path.join("artifact-001").exists());
        assert!(provider.load("artifact-001").is_none());

        let _ = fs::remove_dir_all(path);
    }

    // TEST-37
    // Protects ADR-059: an incomplete payload cannot be restored as a
    // complete artifact.
    #[test]
    fn missing_payload_makes_artifact_unloadable() {
        let path = test_directory("missing-payload");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::remove_file(path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload"))
            .unwrap();

        assert!(provider.load("artifact-001").is_none());
        assert!(provider.list().is_empty());

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn invalid_ids_cannot_escape_persistence_root() {
        let path = test_directory("invalid-id");
        let provider = FilesystemPersistenceProvider::new(&path);

        assert!(provider.load("../outside").is_none());
        assert!(provider.load("nested/id").is_none());
        assert!(provider.load(r"nested\id").is_none());

        let _ = fs::remove_dir_all(path);
    }
}
