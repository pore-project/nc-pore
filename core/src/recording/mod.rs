//! Recording domain model.
//!
//! This module contains domain concepts related to recordings.
//!
//! It intentionally does not contain:
//! - audio backend access
//! - file handling
//! - hardware interaction
//! - synchronization transport logic
//!
//! See:
//! - ADR-039 Recording Architecture and Capture Boundary (future)
//! - ADR-038 Core Implementation Structure and Module Organization
//!
//! The domain records only an opaque `RecordingArtifactId` reference. The
//! technical `RecordingArtifact` and its lifecycle remain outside the Core.
//! A domain artifact association exists only for a completed recording.

pub mod artifact_id;
pub mod coordination;
pub mod id;
pub mod synchronization;
pub mod workflow;

pub use artifact_id::RecordingArtifactId;
pub use coordination::{
    RecordingCoordination, RecordingCoordinationError, RecordingCoordinationStatus,
};
pub use synchronization::{
    RecordingArtifactSynchronization, RecordingArtifactSynchronizationError,
    RecordingArtifactSynchronizationStatus,
};
pub use id::RecordingId;
pub use workflow::{RecordingWorkflow, RecordingWorkflowError, RecordingWorkflowStatus};

use crate::participant::ParticipantId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStatus {
    Prepared,
    Recording,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingLifecycleError {
    InvalidTransition {
        from: RecordingStatus,
        to: RecordingStatus,
    },
    ArtifactConflict {
        existing: RecordingArtifactId,
        requested: RecordingArtifactId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    id: RecordingId,
    participant_id: Option<ParticipantId>,
    status: RecordingStatus,
    artifact_id: Option<RecordingArtifactId>,
}

impl Recording {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingId::new(id),
            participant_id: None,
            status: RecordingStatus::Prepared,
            artifact_id: None,
        }
    }

    pub fn reconstitute(
        id: RecordingId,
        participant_id: Option<ParticipantId>,
        status: RecordingStatus,
        artifact_id: Option<RecordingArtifactId>,
    ) -> Self {
        Self {
            id,
            participant_id,
            status,
            artifact_id,
        }
    }

    pub fn id(&self) -> &RecordingId {
        &self.id
    }
    pub fn participant_id(&self) -> Option<&ParticipantId> {
        self.participant_id.as_ref()
    }
    pub fn status(&self) -> RecordingStatus {
        self.status
    }
    pub fn artifact_id(&self) -> Option<&RecordingArtifactId> {
        self.artifact_id.as_ref()
    }

    pub fn assign_participant(&mut self, participant_id: ParticipantId) {
        self.participant_id = Some(participant_id);
    }

    pub fn start(&mut self) -> Result<(), RecordingLifecycleError> {
        self.transition_to(RecordingStatus::Recording)?;
        Ok(())
    }

    pub fn complete(
        &mut self,
        artifact_id: RecordingArtifactId,
    ) -> Result<(), RecordingLifecycleError> {
        if self.status == RecordingStatus::Completed {
            return match self.artifact_id.as_ref() {
                Some(existing) if existing == &artifact_id => Ok(()),
                Some(existing) => Err(RecordingLifecycleError::ArtifactConflict {
                    existing: existing.clone(),
                    requested: artifact_id,
                }),
                None => Err(RecordingLifecycleError::InvalidTransition {
                    from: RecordingStatus::Completed,
                    to: RecordingStatus::Completed,
                }),
            };
        }
        self.transition_to(RecordingStatus::Completed)?;
        self.artifact_id = Some(artifact_id);
        Ok(())
    }

    fn transition_to(&mut self, target: RecordingStatus) -> Result<(), RecordingLifecycleError> {
        if !matches!(
            (self.status, target),
            (RecordingStatus::Prepared, RecordingStatus::Recording)
                | (RecordingStatus::Recording, RecordingStatus::Completed)
        ) {
            return Err(RecordingLifecycleError::InvalidTransition {
                from: self.status,
                to: target,
            });
        }
        self.status = target;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant_id() -> ParticipantId {
        ParticipantId::new("participant-test-01")
    }

    fn artifact_id() -> RecordingArtifactId {
        RecordingArtifactId::new("artifact-test-01")
    }

    #[test]
    fn new_recording_starts_as_prepared_without_participant_or_artifact() {
        let recording = Recording::new("recording-test-01");
        assert_eq!(recording.status(), RecordingStatus::Prepared);
        assert_eq!(recording.participant_id(), None);
        assert_eq!(recording.artifact_id(), None);
    }

    #[test]
    fn recording_can_be_assigned_to_participant() {
        let participant = participant_id();
        let mut recording = Recording::new("recording-test-02");
        recording.assign_participant(participant.clone());
        assert_eq!(recording.participant_id(), Some(&participant));
    }

    #[test]
    fn prepared_recording_can_transition_to_recording() {
        let mut recording = Recording::new("recording-test-03");
        assert_eq!(recording.start(), Ok(()));
        assert_eq!(recording.status(), RecordingStatus::Recording);
        assert_eq!(recording.artifact_id(), None);
    }

    #[test]
    fn recording_can_transition_to_completed_with_artifact() {
        let mut recording = Recording::new("recording-test-04");
        let expected_artifact = artifact_id();
        recording.start().unwrap();
        assert_eq!(recording.complete(expected_artifact.clone()), Ok(()));
        assert_eq!(recording.status(), RecordingStatus::Completed);
        assert_eq!(recording.artifact_id(), Some(&expected_artifact));
    }

    #[test]
    fn prepared_recording_cannot_be_completed() {
        let mut recording = Recording::new("recording-test-05");
        let result = recording.complete(artifact_id());
        assert_eq!(
            result,
            Err(RecordingLifecycleError::InvalidTransition {
                from: RecordingStatus::Prepared,
                to: RecordingStatus::Completed,
            })
        );
        assert_eq!(recording.status(), RecordingStatus::Prepared);
        assert_eq!(recording.artifact_id(), None);
    }

    #[test]
    fn recording_cannot_be_started_twice() {
        let mut recording = Recording::new("recording-test-06");
        recording.start().unwrap();
        let result = recording.start();
        assert_eq!(
            result,
            Err(RecordingLifecycleError::InvalidTransition {
                from: RecordingStatus::Recording,
                to: RecordingStatus::Recording,
            })
        );
        assert_eq!(recording.status(), RecordingStatus::Recording);
    }

    #[test]
    fn completed_recording_cannot_be_started_again() {
        let mut recording = Recording::new("recording-test-07");
        recording.start().unwrap();
        recording.complete(artifact_id()).unwrap();
        let result = recording.start();
        assert_eq!(
            result,
            Err(RecordingLifecycleError::InvalidTransition {
                from: RecordingStatus::Completed,
                to: RecordingStatus::Recording,
            })
        );
        assert_eq!(recording.status(), RecordingStatus::Completed);
    }

    #[test]
    fn completed_recording_is_idempotent_for_same_artifact() {
        let mut recording = Recording::new("recording-test-08");
        let artifact = artifact_id();
        recording.start().unwrap();
        recording.complete(artifact.clone()).unwrap();
        let result = recording.complete(artifact.clone());
        assert_eq!(result, Ok(()));
        assert_eq!(recording.status(), RecordingStatus::Completed);
        assert_eq!(recording.artifact_id(), Some(&artifact));
    }

    #[test]
    fn completed_recording_rejects_different_artifact() {
        let mut recording = Recording::new("recording-test-09");
        let existing = artifact_id();
        let requested = RecordingArtifactId::new("artifact-test-02");
        recording.start().unwrap();
        recording.complete(existing.clone()).unwrap();
        let result = recording.complete(requested.clone());
        assert_eq!(
            result,
            Err(RecordingLifecycleError::ArtifactConflict {
                existing,
                requested,
            })
        );
    }
}
