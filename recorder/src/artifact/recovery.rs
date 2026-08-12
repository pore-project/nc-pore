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
            if !registry.contains(&artifact.id) {
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
    use super::*;
    use crate::artifact::{ArtifactId, RecordingArtifact};
    use crate::persistence::InMemoryPersistenceProvider;
    use crate::session::RecordingSessionId;

    // TEST-26
    //
    // Protects ADR-053:
    // Recovery rebuilds local registry knowledge
    // from persisted artifacts.
    #[test]
    fn recovery_rebuilds_registry_from_persisted_artifacts() {
        let mut persistence = InMemoryPersistenceProvider::new();

        persistence.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        let mut registry = LocalArtifactRegistry::new();

        let recovery = ArtifactRecoveryService::new();

        recovery.recover(&persistence, &mut registry);

        assert!(registry.contains(&ArtifactId::new("artifact-001")));
    }

    // TEST-31
    //
    // Recovery preserves existing registry knowledge.
    #[test]
    fn recovery_preserves_existing_registry_entry() {
        let mut persistence = InMemoryPersistenceProvider::new();
        persistence.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        let mut registry = LocalArtifactRegistry::new();
        registry.register(ArtifactRegistryEntry::new(
            "artifact-001".to_string(),
            RecordingSessionId::new("existing-session"),
        ));

        let recovery = ArtifactRecoveryService::new();
        recovery.recover(&persistence, &mut registry);

        assert!(registry.contains(&ArtifactId::new("artifact-001")));
        assert_eq!(
            registry
                .find(&ArtifactId::new("artifact-001"))
                .unwrap()
                .recording_session_id
                .value(),
            "existing-session"
        );
    }

    // TEST-32
    //
    // Recovery adds missing registry knowledge without overwriting existing entries.
    #[test]
    fn recovery_adds_missing_entry_and_preserves_existing_registry_entry() {
        let mut persistence = InMemoryPersistenceProvider::new();

        persistence.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("persisted-session-001"),
        ));
        persistence.store(RecordingArtifact::new(
            "artifact-002",
            RecordingSessionId::new("persisted-session-002"),
        ));

        let mut registry = LocalArtifactRegistry::new();
        registry.register(ArtifactRegistryEntry::new(
            "artifact-001".to_string(),
            RecordingSessionId::new("existing-session"),
        ));

        let recovery = ArtifactRecoveryService::new();
        recovery.recover(&persistence, &mut registry);

        assert_eq!(
            registry
                .find(&ArtifactId::new("artifact-001"))
                .unwrap()
                .recording_session_id
                .value(),
            "existing-session"
        );
        assert_eq!(
            registry
                .find(&ArtifactId::new("artifact-002"))
                .unwrap()
                .recording_session_id
                .value(),
            "persisted-session-002"
        );
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

        assert!(!registry.contains(&ArtifactId::new("artifact-001")));
    }
}
