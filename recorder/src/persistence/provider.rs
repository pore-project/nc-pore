//! Persistence Provider Interface.

//!
//! This module defines the boundary between the Recorder workflow
//! and concrete persistence implementations.
//!
//! The interface intentionally contains no storage technology details.
//! Concrete decisions such as filesystem layout, databases or cloud
//! storage are handled by separate implementations.

use crate::artifact::RecordingArtifact;
use crate::persistence::PersistenceLoadResult;

/// Outcome of an attempted artifact persistence operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceStoreError {
    /// A valid artifact with the same identity already exists but differs
    /// from the incoming artifact.
    Conflict { artifact_id: String },
    /// Persistence infrastructure failed while writing the artifact.
    Io(String),
}

/// Technical evidence found while recovering one domain recording.
#[derive(Debug, Clone)]
pub enum PersistenceRecoveryLookup {
    /// Exactly one valid artifact is associated with the requested recording.
    Valid(RecordingArtifact),
    /// A persisted candidate exists but is incomplete.
    Incomplete { artifact_id: String },
    /// A persisted candidate exists but is inconsistent.
    Inconsistent { artifact_id: String },
    /// No persisted artifact candidate is associated with the recording.
    NotFound,
    /// More than one persisted artifact candidate is associated with the recording.
    Conflict { artifact_ids: Vec<String> },
}

impl PersistenceRecoveryLookup {
    fn artifact_id(&self) -> Option<&str> {
        match self {
            Self::Valid(artifact) => Some(artifact.id.value()),
            Self::Incomplete { artifact_id } | Self::Inconsistent { artifact_id } => {
                Some(artifact_id)
            }
            Self::NotFound | Self::Conflict { .. } => None,
        }
    }
}

/// Resolves persistence recovery evidence deterministically.
///
/// Recovery must never choose one candidate merely because it happens to be
/// returned first. A single candidate may be classified explicitly, while
/// multiple candidates always become a conflict, regardless of whether they
/// are valid, incomplete, or inconsistent.
fn resolve_recovery_candidates(
    candidates: Vec<PersistenceRecoveryLookup>,
) -> PersistenceRecoveryLookup {
    match candidates.len() {
        0 => PersistenceRecoveryLookup::NotFound,
        1 => candidates
            .into_iter()
            .next()
            .expect("candidate count guarantees one candidate"),
        _ => {
            let mut artifact_ids: Vec<String> = candidates
                .iter()
                .filter_map(PersistenceRecoveryLookup::artifact_id)
                .map(str::to_owned)
                .collect();
            artifact_ids.sort();
            artifact_ids.dedup();

            PersistenceRecoveryLookup::Conflict { artifact_ids }
        }
    }
}

/// Persistence contract used by the Recorder workflow.
///
/// The workflow depends on this abstraction instead of concrete storage.
/// This keeps persistence replaceable and prevents storage concerns from
/// leaking into recording logic.
pub trait PersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact);

    /// Persists an artifact while returning the lifecycle result explicitly.
    ///
    /// The default implementation preserves the existing provider contract
    /// during migration. Providers that can report persistence failures or
    /// idempotent conflicts override this method.
    fn store_checked(
        &mut self,
        mut artifact: RecordingArtifact,
    ) -> Result<RecordingArtifact, PersistenceStoreError> {
        artifact.store();
        self.store(artifact.clone());
        Ok(artifact)
    }

    /// Assesses the persisted representation of one artifact.
    ///
    /// The boundary deliberately distinguishes a valid artifact from
    /// incomplete or inconsistent persisted data. This prevents recovery
    /// code from treating every existing artifact directory as valid.
    fn load(&self, id: &str) -> PersistenceLoadResult;

    /// Looks up technical recovery evidence for one concrete domain recording.
    ///
    /// Candidate discovery remains inside the persistence boundary. Callers
    /// therefore do not need to enumerate artifact identifiers or inspect
    /// concrete storage layouts. Providers that can associate incomplete or
    /// inconsistent persisted candidates with the requested recording should
    /// override this method and return those states explicitly.
    fn find_for_recording(
        &self,
        production_id: &str,
        recording_id: &str,
    ) -> PersistenceRecoveryLookup {
        let mut matches = Vec::new();

        for artifact_id in self.list_ids() {
            match self.load(&artifact_id) {
                PersistenceLoadResult::Valid(artifact)
                    if artifact.production_id() == Some(production_id)
                        && artifact.recording_id() == Some(recording_id) =>
                {
                    matches.push(PersistenceRecoveryLookup::Valid(artifact));
                }
                PersistenceLoadResult::Incomplete | PersistenceLoadResult::Inconsistent => {}
                PersistenceLoadResult::NotFound => {}
                PersistenceLoadResult::Valid(_) => {}
            }
        }

        resolve_recovery_candidates(matches)
    }

    /// Lists identifiers for persisted artifact candidates without loading
    /// their artifact representations.
    ///
    /// This remains part of the low-level persistence contract for existing
    /// maintenance and inspection use cases. Recovery callers should use
    /// `find_for_recording` instead.
    fn list_ids(&self) -> Vec<String>;

    #[allow(dead_code)]
    fn list(&self) -> Vec<RecordingArtifact>;

    #[allow(dead_code)]
    fn remove(&mut self, id: &str);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::RecordingSessionId;

    #[test]
    fn recovery_with_no_candidates_is_not_found() {
        assert!(matches!(
            resolve_recovery_candidates(Vec::new()),
            PersistenceRecoveryLookup::NotFound
        ));
    }

    #[test]
    fn recovery_preserves_a_single_incomplete_candidate() {
        assert!(matches!(
            resolve_recovery_candidates(vec![PersistenceRecoveryLookup::Incomplete {
                artifact_id: "artifact-002".to_owned(),
            }]),
            PersistenceRecoveryLookup::Incomplete { artifact_id } if artifact_id == "artifact-002"
        ));
    }

    #[test]
    fn recovery_preserves_a_single_inconsistent_candidate() {
        assert!(matches!(
            resolve_recovery_candidates(vec![PersistenceRecoveryLookup::Inconsistent {
                artifact_id: "artifact-003".to_owned(),
            }]),
            PersistenceRecoveryLookup::Inconsistent { artifact_id } if artifact_id == "artifact-003"
        ));
    }

    #[test]
    fn recovery_turns_multiple_candidates_into_sorted_conflict() {
        let artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        assert!(matches!(
            resolve_recovery_candidates(vec![
                PersistenceRecoveryLookup::Inconsistent {
                    artifact_id: "artifact-003".to_owned(),
                },
                PersistenceRecoveryLookup::Valid(artifact),
                PersistenceRecoveryLookup::Incomplete {
                    artifact_id: "artifact-002".to_owned(),
                },
            ]),
            PersistenceRecoveryLookup::Conflict { artifact_ids }
                if artifact_ids == vec![
                    "artifact-001".to_owned(),
                    "artifact-002".to_owned(),
                    "artifact-003".to_owned(),
                ]
        ));
    }
}
