//! Artifact coordination boundary.
//!
//! This module coordinates the technical handling of Recording Artifacts
//! after their creation.
//!
//! It connects:
//! - LocalArtifactRegistry
//! - PersistenceProvider
//!
//! It intentionally does not contain:
//! - artifact lifecycle rules
//! - storage implementations
//! - recovery algorithms
//! - synchronization logic
//!
//! See:
//! - ADR-047 Local Artifact Registry and Discovery Strategy

use crate::artifact::recovery::ArtifactRecoveryService;
use crate::artifact::registry::{ArtifactRegistryEntry, LocalArtifactRegistry};
use crate::artifact::{ArtifactId, RecordingArtifact};
use crate::persistence::PersistenceProvider;

/// Coordinates artifact registration and persistence.
///
/// The coordinator connects artifact discovery with persistence
/// without coupling either component to the other.
pub struct ArtifactCoordinator<P>
where
    P: PersistenceProvider,
{
    registry: LocalArtifactRegistry,
    persistence: P,
}

impl<P> ArtifactCoordinator<P>
where
    P: PersistenceProvider,
{
    pub fn new(persistence: P) -> Self {
    let mut registry = LocalArtifactRegistry::new();

    ArtifactRecoveryService::new().recover(&persistence, &mut registry);

    Self {
        registry,
        persistence,
    }
}

pub fn register_and_store(&mut self, mut artifact: RecordingArtifact) -> RecordingArtifact {
        self.registry.register(ArtifactRegistryEntry::new(
            artifact.id.value().to_string(),
            artifact.recording_session_id.clone(),
        ));

        artifact.store();

        self.persistence.store(artifact.clone());

        artifact
    }

    pub fn registry(&self) -> &LocalArtifactRegistry {
        &self.registry
    }

    pub fn persistence(&self) -> &P {
        &self.persistence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactStatus;
    use crate::persistence::InMemoryPersistenceProvider;
    use crate::session::RecordingSessionId;

    #[test]
    fn coordinator_registers_artifact_reference() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        coordinator.register_and_store(artifact);

        assert!(
            coordinator
                .registry()
                .contains(&ArtifactId::new("artifact-001"))
        );
    }

// TEST-33
//
// Protects ADR-053:
// Artifact coordination restores registry knowledge
// from already persisted artifacts during initialization.
#[test]
fn coordinator_recovers_persisted_artifact_registry_entries() {
    let mut persistence = InMemoryPersistenceProvider::new();

    persistence.store(RecordingArtifact::new(
        "artifact-033",
        RecordingSessionId::new("session-033"),
    ));

    let coordinator = ArtifactCoordinator::new(persistence);

    assert!(
        coordinator
            .registry()
            .contains(&ArtifactId::new("artifact-033"))
    );
}

#[test]
fn coordinator_persists_artifact() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        let stored_artifact = coordinator.register_and_store(artifact);

        assert_eq!(stored_artifact.status(), &ArtifactStatus::Stored);

        assert!(coordinator.persistence().load("artifact-001").is_some());
    }

    // TEST-26
    //
    // Protects ADR-047 and ADR-052:
    //
    // Artifact coordination works with a concrete filesystem
    // persistence implementation through the persistence boundary.
    #[test]
    fn coordinator_persists_artifact_with_filesystem_provider() {
        let path = std::env::temp_dir().join("nc-pore-test-26");

        let persistence = crate::persistence::FilesystemPersistenceProvider::new(&path);

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact =
            RecordingArtifact::new("artifact-026", RecordingSessionId::new("session-026"));

        let stored_artifact = coordinator.register_and_store(artifact);

        assert_eq!(stored_artifact.status(), &ArtifactStatus::Stored);

        assert!(coordinator.persistence().load("artifact-026").is_some());

        let _ = std::fs::remove_dir_all(path);
    }
}
