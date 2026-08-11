//! Recording Artifact recovery boundary.
//!
//! This module restores consistency between persisted artifacts
//! and the local artifact registry.
//!
//! It connects:
//! - PersistenceProvider
//! - LocalArtifactRegistry
//!
//! It intentionally does not contain:
//! - workflow coordination
//! - artifact creation logic
//! - storage implementation details
//! - synchronization logic
//!
//! See:
//! - ADR-053 Artifact Recovery and Consistency Boundary

use crate::artifact::registry::{ArtifactRegistryEntry, LocalArtifactRegistry};
use crate::persistence::PersistenceProvider;

/// Restores local artifact discovery state from persistence.
///
/// The recovery service rebuilds registry knowledge from persisted
/// RecordingArtifacts without modifying artifact lifecycle rules.
pub struct ArtifactRecoveryService;

impl ArtifactRecoveryService {
    /// Creates a new recovery service.
    pub fn new() -> Self {
        Self
    }

    /// Restores registry entries from persisted artifacts.
    ///
    /// Existing registry entries are preserved.
    /// Recovery only adds missing knowledge.
    pub fn recover<P: PersistenceProvider>(
        &self,
        persistence: &P,
        registry: &mut LocalArtifactRegistry,
    ) {
        for artifact in persistence.list() {
            if !registry.contains(artifact.id.value()) {
                registry.register(ArtifactRegistryEntry::new(
                    artifact.id.value().to_string(),
                    artifact.recording_session_id,
                ));
            }
        }
    }
}

impl Default for ArtifactRecoveryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
use crate::session::RecordingSessionId;
    use super::*;
    use crate::artifact::RecordingArtifact;
    use crate::persistence::InMemoryPersistenceProvider;

    // TEST-26
    //
    // Protects ADR-053:
    // Recovery rebuilds local registry knowledge
    // from persisted artifacts.
    #[test]
    fn recovery_rebuilds_registry_from_persisted_artifacts() {
        let mut persistence = InMemoryPersistenceProvider::new();

        persistence.store(RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001")));

        let mut registry = LocalArtifactRegistry::new();

        let recovery = ArtifactRecoveryService::new();

        recovery.recover(&persistence, &mut registry);

        assert!(registry.contains("artifact-001"));
    }

    // TEST-27
    //
    // Protects ADR-053:
    // Recovery does not create entries for missing persistence data.
    #[test]
    fn recovery_ignores_missing_artifacts() {
        let persistence = InMemoryPersistenceProvider::new();

        let mut registry = LocalArtifactRegistry::new();

        let recovery = ArtifactRecoveryService::new();

        recovery.recover(&persistence, &mut registry);

        assert!(!registry.contains("artifact-001"));
    }
}
