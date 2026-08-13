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
use crate::artifact::ArtifactId;
use crate::persistence::{PersistenceLoadResult, PersistenceProvider};

/// Outcome of one recovery pass over persisted artifact candidates.
///
/// The recovery result keeps persistence assessment visible to its caller
/// without making the local artifact registry responsible for persistence
/// states.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ArtifactRecoveryResult {
    pub recovered: Vec<ArtifactId>,
    pub incomplete: Vec<ArtifactId>,
    pub inconsistent: Vec<ArtifactId>,
    pub not_found: Vec<ArtifactId>,
}

/// Restores local artifact discovery state from persistence.
pub struct ArtifactRecoveryService;

impl ArtifactRecoveryService {
    /// Creates a new recovery service.
    pub fn new() -> Self {
        Self
    }

    /// Restores registry entries from persisted artifact candidates.
    ///
    /// Existing registry entries are preserved.
    /// Recovery only adds missing knowledge for valid persisted artifacts.
    /// Persistence assessment outcomes remain visible in the returned result.
    pub fn recover<P: PersistenceProvider>(
        &self,
        persistence: &P,
        registry: &mut LocalArtifactRegistry,
    ) -> ArtifactRecoveryResult {
        let mut result = ArtifactRecoveryResult::default();

        for id in persistence.list_ids() {
            let artifact_id = ArtifactId::new(id.clone());

            match persistence.load(&id) {
                PersistenceLoadResult::Valid(artifact) => {
                    if !registry.contains(&artifact.id) {
                        registry.register(ArtifactRegistryEntry::new(
                            artifact.id.value().to_string(),
                            artifact.recording_session_id,
                        ));
                    }
                    result.recovered.push(artifact_id);
                }
                PersistenceLoadResult::Incomplete => result.incomplete.push(artifact_id),
                PersistenceLoadResult::Inconsistent => result.inconsistent.push(artifact_id),
                PersistenceLoadResult::NotFound => result.not_found.push(artifact_id),
            }
        }

        result
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
    use crate::artifact::RecordingArtifact;
    use crate::persistence::InMemoryPersistenceProvider;
    use crate::session::RecordingSessionId;

    // TEST-26
    //
    // Protects ADR-053:
    // Recovery rebuilds local registry knowledge from valid persisted artifacts.
    #[test]
    fn recovery_rebuilds_registry_from_persisted_artifacts() {
        let mut persistence = InMemoryPersistenceProvider::new();

        persistence.store(RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("session-001"),
        ));

        let mut registry = LocalArtifactRegistry::new();
        let recovery = ArtifactRecoveryService::new();

        let result = recovery.recover(&persistence, &mut registry);

        assert!(registry.contains(&ArtifactId::new("artifact-001")));
        assert_eq!(result.recovered, vec![ArtifactId::new("artifact-001")]);
        assert!(result.incomplete.is_empty());
        assert!(result.inconsistent.is_empty());
        assert!(result.not_found.is_empty());
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
    // Recovery does not create entries when there are no persistence candidates.
    #[test]
    fn recovery_ignores_missing_artifacts() {
        let persistence = InMemoryPersistenceProvider::new();
        let mut registry = LocalArtifactRegistry::new();
        let recovery = ArtifactRecoveryService::new();

        let result = recovery.recover(&persistence, &mut registry);

        assert!(!registry.contains(&ArtifactId::new("artifact-001")));
        assert!(result.recovered.is_empty());
        assert!(result.incomplete.is_empty());
        assert!(result.inconsistent.is_empty());
        assert!(result.not_found.is_empty());
    }

    struct AssessmentPersistenceProvider {
        candidates: Vec<(String, PersistenceLoadResult)>,
    }

    impl AssessmentPersistenceProvider {
        fn new(candidates: Vec<(String, PersistenceLoadResult)>) -> Self {
            Self { candidates }
        }
    }

    impl PersistenceProvider for AssessmentPersistenceProvider {
        fn store(&mut self, _artifact: RecordingArtifact) {
            unreachable!("store is not needed for recovery assessment tests");
        }

        fn load(&self, id: &str) -> PersistenceLoadResult {
            self.candidates
                .iter()
                .find(|(candidate_id, _)| candidate_id == id)
                .map(|(_, result)| result.clone())
                .unwrap_or(PersistenceLoadResult::NotFound)
        }

        fn list_ids(&self) -> Vec<String> {
            self.candidates.iter().map(|(id, _)| id.clone()).collect()
        }

        fn list(&self) -> Vec<RecordingArtifact> {
            Vec::new()
        }

        fn remove(&mut self, _id: &str) {
            unreachable!("remove is not needed for recovery assessment tests");
        }
    }

    // TEST-40
    //
    // Protects the persistence assessment boundary:
    // recovery keeps valid, incomplete, inconsistent and not-found outcomes distinct.
    #[test]
    fn recovery_reports_persistence_assessment_outcomes() {
        let persistence = AssessmentPersistenceProvider::new(vec![
            (
                "artifact-valid".to_string(),
                PersistenceLoadResult::Valid(RecordingArtifact::new(
                    "artifact-valid",
                    RecordingSessionId::new("session-valid"),
                )),
            ),
            (
                "artifact-incomplete".to_string(),
                PersistenceLoadResult::Incomplete,
            ),
            (
                "artifact-inconsistent".to_string(),
                PersistenceLoadResult::Inconsistent,
            ),
            (
                "artifact-not-found".to_string(),
                PersistenceLoadResult::NotFound,
            ),
        ]);

        let mut registry = LocalArtifactRegistry::new();
        let recovery = ArtifactRecoveryService::new();

        let result = recovery.recover(&persistence, &mut registry);

        assert_eq!(result.recovered, vec![ArtifactId::new("artifact-valid")]);
        assert_eq!(
            result.incomplete,
            vec![ArtifactId::new("artifact-incomplete")]
        );
        assert_eq!(
            result.inconsistent,
            vec![ArtifactId::new("artifact-inconsistent")]
        );
        assert_eq!(
            result.not_found,
            vec![ArtifactId::new("artifact-not-found")]
        );
        assert!(registry.contains(&ArtifactId::new("artifact-valid")));
        assert!(!registry.contains(&ArtifactId::new("artifact-incomplete")));
        assert!(!registry.contains(&ArtifactId::new("artifact-inconsistent")));
        assert!(!registry.contains(&ArtifactId::new("artifact-not-found")));
    }
}
