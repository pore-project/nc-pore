//! Persistence boundary implementations.
//!
//! The Recorder interacts with persistence only through the
//! PersistenceProvider interface.
//!
//! Concrete persistence technologies remain behind this boundary.
//!
//! See:
//! - ADR-043 Local Recording Persistence Boundary
//! - ADR-044 Persistence Provider Interface
//! - ADR-052 Local Filesystem Persistence Provider

mod filesystem;
mod provider;

pub use filesystem::FilesystemPersistenceProvider;
pub use provider::PersistenceProvider;

use crate::artifact::RecordingArtifact;

/// Reference implementation used for development and tests.
///
/// This implementation validates the persistence boundary without
/// committing NC-PoRe to a specific storage technology.
pub struct InMemoryPersistenceProvider {
    artifacts: Vec<RecordingArtifact>,
}

impl InMemoryPersistenceProvider {
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }
}

impl Default for InMemoryPersistenceProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl PersistenceProvider for InMemoryPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.artifacts.push(artifact);
    }

    fn load(&self, id: &str) -> Option<RecordingArtifact> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.id.value() == id)
            .cloned()
    }

    fn list(&self) -> Vec<RecordingArtifact> {
        self.artifacts.clone()
    }

    fn remove(&mut self, id: &str) {
        self.artifacts.retain(|artifact| artifact.id.value() != id);
    }
}

#[cfg(test)]
mod tests {
use crate::session::RecordingSessionId;
    use super::*;
    use crate::artifact::RecordingArtifact;
    use crate::persistence::FilesystemPersistenceProvider;

    // TEST-12
    //
    // Protects ADR-044:
    // Recording persistence is accessed through the provider boundary.
    #[test]
    fn test_12_provider_can_store_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider.store(RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001")));

        assert_eq!(provider.list().len(), 1);
    }

    // TEST-13
    //
    // Protects ADR-044:
    // Persisted artifacts are retrieved through the provider contract.
    #[test]
    fn test_13_provider_can_load_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider.store(RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001")));

        let artifact = provider.load("artifact-001");

        assert!(artifact.is_some());
        assert_eq!(artifact.unwrap().id.value(), "artifact-001");
    }

    // TEST-14
    //
    // Protects ADR-044:
    // The provider boundary supports retrieving persisted artifacts.
    #[test]
    fn test_14_provider_can_list_artifacts() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider.store(RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001")));

        provider.store(RecordingArtifact::new("artifact-002", RecordingSessionId::new("session-001")));

        assert_eq!(provider.list().len(), 2);
    }

    // TEST-15
    //
    // Protects ADR-044:
    // Removal is part of the persistence contract.
    #[test]
    fn test_15_provider_can_remove_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider.store(RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001")));

        provider.remove("artifact-001");

        assert!(provider.load("artifact-001").is_none());
    }

    // TEST-20
    //
    // Protects ADR-052:
    // The filesystem persistence implementation is exposed
    // through the persistence boundary.
    #[test]
    fn test_20_filesystem_provider_is_available_through_boundary() {
        let path = std::env::temp_dir().join("nc-pore-test-20");

        let provider = FilesystemPersistenceProvider::new(&path);

        drop(provider);

        let _ = std::fs::remove_dir_all(path);
    }
}
