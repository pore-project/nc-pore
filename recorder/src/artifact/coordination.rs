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

use crate::artifact::RecordingArtifact;
use crate::artifact::registry::{ArtifactRegistryEntry, LocalArtifactRegistry};
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
        Self {
            registry: LocalArtifactRegistry::new(),
            persistence,
        }
    }

    pub fn register_and_store(&mut self, mut artifact: RecordingArtifact) -> RecordingArtifact {
        self.registry.register(ArtifactRegistryEntry::new(
            artifact.id.clone(),
            artifact.recording_session_id.clone(),
        ));

        self.persistence.store(artifact.clone());

        artifact.store();

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

    #[test]
    fn coordinator_registers_artifact_reference() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact = RecordingArtifact::new("artifact-001", "session-001");

        coordinator.register_and_store(artifact);

        assert!(coordinator.registry().contains("artifact-001"));
    }

    #[test]
    fn coordinator_persists_artifact() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact = RecordingArtifact::new("artifact-001", "session-001");

        let stored_artifact = coordinator.register_and_store(artifact);

        assert_eq!(stored_artifact.status(), &ArtifactStatus::Stored);

        assert!(coordinator.persistence().load("artifact-001").is_some());
    }
}
