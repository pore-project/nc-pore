//! Filesystem persistence provider.
//!
//! This module provides a concrete local filesystem implementation
//! of the PersistenceProvider boundary.
//!
//! The implementation intentionally remains separated from:
//! - Recorder workflow logic
//! - Artifact processing logic
//! - Artifact domain representation
//!
//! See:
//! - ADR-052 Local Filesystem Persistence Provider

use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact::{ArtifactStatus, RecordingArtifact};
use crate::persistence::PersistenceProvider;
use crate::session::RecordingSessionId;

#[derive(Debug)]
struct PersistedRecordingArtifact {
    id: String,
    recording_session_id: String,
    status: String,
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
        }
    }
}

impl PersistedRecordingArtifact {
    fn into_recording_artifact(self) -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new(self.id, RecordingSessionId::new(self.recording_session_id));

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
/// Each RecordingArtifact is stored as an individual file.
///
/// The concrete file layout is intentionally kept simple
/// until further requirements exist.
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

    fn artifact_path(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }

    fn write_artifact(&self, artifact: &RecordingArtifact) {
        let persisted = PersistedRecordingArtifact::from(artifact);

        let content = format!(
            "{}\n{}\n{}",
            persisted.id, persisted.recording_session_id, persisted.status
        );

        fs::write(self.artifact_path(&persisted.id), content).expect("failed to write artifact");
    }

    fn read_artifact(path: &Path) -> Option<RecordingArtifact> {
        let content = fs::read_to_string(path).ok()?;

        let mut lines = content.lines();

        let persisted = PersistedRecordingArtifact {
            id: lines.next()?.to_string(),
            recording_session_id: lines.next()?.to_string(),
            status: lines.next()?.to_string(),
        };

        Some(persisted.into_recording_artifact())
    }
}

impl PersistenceProvider for FilesystemPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.write_artifact(&artifact);
    }

    fn load(&self, id: &str) -> Option<RecordingArtifact> {
        Self::read_artifact(&self.artifact_path(id))
    }

    fn list(&self) -> Vec<RecordingArtifact> {
        let entries = fs::read_dir(&self.root).expect("failed to read persistence directory");

        entries
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| Self::read_artifact(&entry.path()))
            .collect()
    }

    fn remove(&mut self, id: &str) {
        let _ = fs::remove_file(self.artifact_path(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("nc-pore-{name}"))
    }

    // TEST-16
    //
    // Protects ADR-052:
    // Filesystem persistence stores artifacts through
    // the PersistenceProvider boundary.
    #[test]
    fn test_16_filesystem_provider_can_store_artifact() {
        let path = test_directory("test-16");

        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        assert!(provider.load("artifact-001").is_some());

        let _ = fs::remove_dir_all(path);
    }

    // TEST-17
    //
    // Protects ADR-052:
    // Filesystem persistence can restore stored artifacts.
    #[test]
    fn test_17_filesystem_provider_can_load_artifact() {
        let path = test_directory("test-17");

        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        let artifact = provider.load("artifact-001");

        assert!(artifact.is_some());
        assert_eq!(
            artifact.unwrap().recording_session_id.value(),
            "session-001"
        );

        let _ = fs::remove_dir_all(path);
    }

    // TEST-18
    //
    // Protects ADR-052:
    // Filesystem persistence supports artifact discovery.
    #[test]
    fn test_18_filesystem_provider_can_list_artifacts() {
        let path = test_directory("test-18");

        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        provider.store(RecordingArtifact::new(
            "artifact-002",
            RecordingSessionId::new("session-001"),
        ));

        assert_eq!(provider.list().len(), 2);

        let _ = fs::remove_dir_all(path);
    }

    // TEST-19
    //
    // Protects ADR-052:
    // Filesystem persistence supports artifact removal.
    #[test]
    fn test_19_filesystem_provider_can_remove_artifact() {
        let path = test_directory("test-19");

        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        provider.remove("artifact-001");

        assert!(provider.load("artifact-001").is_none());

        let _ = fs::remove_dir_all(path);
    }
}
