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

mod assessment;
mod filesystem;
mod provider;

pub use assessment::PersistenceLoadResult;
pub use filesystem::FilesystemPersistenceProvider;
pub use provider::{PersistenceProvider, PersistenceStoreError};

use crate::artifact::RecordingArtifact;

/// Returns whether two artifacts represent the same persisted content.
///
/// Lifecycle status is intentionally excluded: an incoming Available artifact
/// is equivalent to an already persisted Stored artifact when all persisted
/// content and identity fields match.
fn artifacts_are_equivalent(left: &RecordingArtifact, right: &RecordingArtifact) -> bool {
    left.id == right.id
        && left.recording_session_id == right.recording_session_id
        && left.tracks() == right.tracks()
        && left.association() == right.association()
}

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
    fn store(
        &mut self,
        mut artifact: RecordingArtifact,
    ) -> Result<RecordingArtifact, PersistenceStoreError> {
        if let Some(existing) = self
            .artifacts
            .iter()
            .find(|existing| existing.id == artifact.id)
        {
            if artifacts_are_equivalent(existing, &artifact) {
                return Ok(existing.clone());
            }

            return Err(PersistenceStoreError::Conflict {
                artifact_id: artifact.id.value().to_owned(),
            });
        }

        artifact.store();
        self.artifacts.push(artifact.clone());
        Ok(artifact)
    }

    fn load(&self, id: &str) -> PersistenceLoadResult {
        self.artifacts
            .iter()
            .find(|artifact| artifact.id.value() == id)
            .cloned()
            .map(PersistenceLoadResult::Valid)
            .unwrap_or(PersistenceLoadResult::NotFound)
    }

    fn list_ids(&self) -> Vec<String> {
        self.artifacts
            .iter()
            .map(|artifact| artifact.id.value().to_owned())
            .collect()
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
    use super::*;
    use crate::artifact::RecordingArtifact;
    use crate::persistence::FilesystemPersistenceProvider;
    use crate::session::RecordingSessionId;

    // TEST-12
    //
    // Protects ADR-044:
    // Recording persistence is accessed through the provider boundary.
    #[test]
    fn test_12_provider_can_store_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider
            .store(RecordingArtifact::new(
                "artifact-001",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        assert_eq!(provider.list().len(), 1);
    }

    // TEST-13
    //
    // Protects ADR-044:
    // Persisted artifacts are retrieved through the provider contract.
    #[test]
    fn test_13_provider_can_load_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider
            .store(RecordingArtifact::new(
                "artifact-001",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Valid(artifact) if artifact.id.value() == "artifact-001"
        ));
    }

    // TEST-14
    //
    // Protects ADR-044:
    // The provider boundary supports retrieving persisted artifacts.
    #[test]
    fn test_14_provider_can_list_artifacts() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider
            .store(RecordingArtifact::new(
                "artifact-001",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        provider
            .store(RecordingArtifact::new(
                "artifact-002",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        assert_eq!(provider.list().len(), 2);
    }

    #[test]
    fn provider_can_list_artifact_ids() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider
            .store(RecordingArtifact::new(
                "artifact-001",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        provider
            .store(RecordingArtifact::new(
                "artifact-002",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        assert_eq!(
            provider.list_ids(),
            vec!["artifact-001".to_owned(), "artifact-002".to_owned()]
        );
    }

    // TEST-15
    //
    // Protects ADR-044:
    // Removal is part of the persistence contract.
    #[test]
    fn test_15_provider_can_remove_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        provider
            .store(RecordingArtifact::new(
                "artifact-001",
                RecordingSessionId::new("session-001"),
            ))
            .expect("artifact should be stored");

        provider.remove("artifact-001");

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::NotFound
        ));
    }

    // TEST-21
    //
    // Protects the persistence assessment boundary:
    // a missing persisted artifact is distinct from an invalid one.
    #[test]
    fn test_21_provider_reports_missing_artifact() {
        let provider = InMemoryPersistenceProvider::new();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::NotFound
        ));
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

    // TEST-37
    //
    // Protects ADR-060:
    // Storing the same artifact identity and equivalent content is an
    // idempotent success and does not create a duplicate.
    #[test]
    fn test_37_provider_is_idempotent_for_equivalent_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();
        let artifact = RecordingArtifact::new("artifact-037", RecordingSessionId::new("session-037"));

        provider.store(artifact.clone()).expect("first store should succeed");
        provider.store(artifact).expect("equivalent store should be a no-op");

        assert_eq!(provider.list().len(), 1);
    }

    // TEST-38
    //
    // Protects ADR-060:
    // A different artifact under an already used identity is rejected
    // instead of silently replacing the persisted artifact.
    #[test]
    fn test_38_provider_rejects_conflicting_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();
        let first = RecordingArtifact::new("artifact-038", RecordingSessionId::new("session-038-a"));
        let conflicting = RecordingArtifact::new("artifact-038", RecordingSessionId::new("session-038-b"));

        provider.store(first).expect("first store should succeed");

        assert!(matches!(
            provider.store(conflicting),
            Err(PersistenceStoreError::Conflict { artifact_id }) if artifact_id == "artifact-038"
        ));

        assert_eq!(provider.list().len(), 1);
    }
}