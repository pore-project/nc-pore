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

    pub fn register_and_store(&mut self, artifact: RecordingArtifact) {
        self.registry.register(ArtifactRegistryEntry::new(
            artifact.id.clone(),
            artifact.recording_session_id.clone(),
        ));

        self.persistence.store(artifact);
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
    use crate::persistence::InMemoryPersistenceProvider;

    // TEST-20
    //
    // Protects ADR-047:
    // Artifact coordination registers technical references
    // independently from artifact storage.
    #[test]
    fn coordinator_registers_artifact_reference() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact = RecordingArtifact::new("artifact-001", "session-001");

        coordinator.register_and_store(artifact);

        assert!(coordinator.registry().contains("artifact-001"));
    }

    // TEST-21
    //
    // Protects ADR-043 and ADR-044:
    // Artifact coordination stores artifacts through the persistence boundary.
    #[test]
    fn coordinator_persists_artifact() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut coordinator = ArtifactCoordinator::new(persistence);

        let artifact = RecordingArtifact::new("artifact-001", "session-001");

        coordinator.register_and_store(artifact);

        assert!(coordinator.persistence().load("artifact-001").is_some());
    }
}
