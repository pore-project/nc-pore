//! Filesystem persistence provider.
//!
//! This module provides the concrete local filesystem implementation
//! of the PersistenceProvider boundary.
//!
//! The physical layout follows ADR-055:
//!
//! <root>/
//!   <artifact-id>/
//!     artifact.json
//!     tracks/
//!       <track-id>/
//!         chunks/
//!
//! RecordingChunk currently contains only its technical sequence number.
//! Therefore this implementation persists chunk metadata, but does not
//! invent audio payload files that the capture model cannot provide yet.
//!
//! See:
//! - ADR-052 Local Filesystem Persistence Provider
//! - ADR-054 Recording Artifact and Local Recording Data Association
//! - ADR-055 Filesystem Persistence Layout

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
}

impl From<&RecordingArtifact> for PersistedRecordingArtifact {
    fn from(artifact: &RecordingArtifact) -> Self {
        Self {
            id: artifact.id.value().to_string(),
            recording_session_id: artifact.recording_session_id.value().to_string(),
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
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

impl PersistedRecordingArtifact {
    fn into_recording_artifact(self) -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new(self.id, RecordingSessionId::new(self.recording_session_id));

        for persisted_track in self.tracks {
            let mut track = RecordingTrack::new(persisted_track.id);

            for persisted_chunk in persisted_track.chunks {
                track.add_chunk(RecordingChunk::new(persisted_chunk.sequence));
            }

            artifact.add_track(track);
        }

        match self.status.as_str() {
            "Available" => artifact.make_available(),
            "Stored" => {
                artifact.make_available();
                artifact.store();
            }
            _ => {}
        }

        artifact
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
        !id.is_empty()
            && id != "."
            && id != ".."
            && !id.contains('/')
            && !id.contains('\\')
    }

    fn artifact_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn artifact_metadata_path(&self, id: &str) -> PathBuf {
        self.artifact_dir(id).join("artifact.json")
    }

    fn write_artifact(&self, artifact: &RecordingArtifact) {
        assert!(Self::validate_id(artifact.id.value()), "invalid artifact id");

        for track in artifact.tracks() {
            assert!(Self::validate_id(track.id.value()), "invalid track id");
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

        for track in &persisted.tracks {
            let chunks_dir = temp_dir.join("tracks").join(&track.id).join("chunks");
            fs::create_dir_all(&chunks_dir).expect("failed to create chunk directory");
        }

        let _ = fs::remove_dir_all(&artifact_dir);
        fs::rename(&temp_dir, &artifact_dir).expect("failed to publish artifact directory");
    }

    fn read_artifact(path: &Path) -> Option<RecordingArtifact> {
        let metadata_path = path.join("artifact.json");
        let content = fs::read_to_string(metadata_path).ok()?;
        let persisted: PersistedRecordingArtifact = serde_json::from_str(&content).ok()?;

        Some(persisted.into_recording_artifact())
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

        let mut host = RecordingTrack::new("track-host");
        host.add_chunk(RecordingChunk::new(1));
        host.add_chunk(RecordingChunk::new(2));
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
        assert!(path.join("artifact-001/tracks/track-host/chunks").is_dir());

        let _ = fs::remove_dir_all(path);
    }

    // TEST-17
    // Protects ADR-054/055: persisted tracks and chunks can be restored.
    #[test]
    fn test_17_filesystem_provider_can_load_artifact() {
        let path = test_directory("test-17");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        let artifact = provider.load("artifact-001").expect("artifact missing");

        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks().len(), 2);
        assert_eq!(artifact.tracks()[0].chunks()[1].sequence, 2);

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

    #[test]
    fn invalid_ids_cannot_escape_persistence_root() {
        let path = test_directory("invalid-id");
        let provider = FilesystemPersistenceProvider::new(&path);

        assert!(provider.load("../outside").is_none());
        assert!(provider.load("nested/id").is_none());
        assert!(provider.load(r"nested\\id").is_none());

        let _ = fs::remove_dir_all(path);
    }
}
