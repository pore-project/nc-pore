//! Recording domain model.
//
//! This module contains domain concepts related to recordings.
//!
//! It intentionally does not contain:
//! - audio backend access
//! - file handling
//! - hardware interaction
//! - synchronization logic
//!
//! See:
//! - ADR-039 Recording Architecture and Capture Boundary
//! - ADR-084 Recording Artifact Aggregation and Completion Semantics
//! - ADR-038 Core Implementation Structure and Module Organization

pub mod artifact_id;
pub mod coordination;
pub mod id;
pub mod synchronization;
pub mod workflow;

pub use artifact_id::RecordingArtifactId;
pub use coordination::{
    RecordingCoordination, RecordingCoordinationError, RecordingCoordinationStatus,
};
pub use id::RecordingId;
pub use synchronization::{
    RecordingArtifactSynchronization, RecordingArtifactSynchronizationError,
    RecordingArtifactSynchronizationStatus,
};
pub use workflow::{RecordingWorkflow, RecordingWorkflowError, RecordingWorkflowStatus};

use crate::participant::ParticipantId;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStatus {
    Prepared,
    Recording,
    Stopped,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingLifecycleError {
    InvalidTransition {
        from: RecordingStatus,
        to: RecordingStatus,
    },
    DuplicateExpectedParticipant {
        participant_id: ParticipantId,
    },
    NoExpectedParticipants,
    ParticipantNotExpected {
        participant_id: ParticipantId,
    },
    ArtifactConflict {
        existing: RecordingArtifactId,
        requested: RecordingArtifactId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingArtifactSlot {
    participant_id: ParticipantId,
    artifact_id: Option<RecordingArtifactId>,
}

impl RecordingArtifactSlot {
    pub fn new(participant_id: ParticipantId) -> Self {
        Self {
            participant_id,
            artifact_id: None,
        }
    }

    pub fn reconstitute(
        participant_id: ParticipantId,
        artifact_id: Option<RecordingArtifactId>,
    ) -> Self {
        Self {
            participant_id,
            artifact_id,
        }
    }

    pub fn participant_id(&self) -> &ParticipantId {
        &self.participant_id
    }

    pub fn artifact_id(&self) -> Option<&RecordingArtifactId> {
        self.artifact_id.as_ref()
    }

    fn complete(
        &mut self,
        artifact_id: RecordingArtifactId,
    ) -> Result<bool, RecordingLifecycleError> {
        match self.artifact_id.as_ref() {
            Some(existing) if existing == &artifact_id => Ok(false),
            Some(existing) => Err(RecordingLifecycleError::ArtifactConflict {
                existing: existing.clone(),
                requested: artifact_id,
            }),
            None => {
                self.artifact_id = Some(artifact_id);
                Ok(true)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    id: RecordingId,
    artifact_slots: Vec<RecordingArtifactSlot>,
    status: RecordingStatus,
    stopped_at: Option<SystemTime>,
}

impl Recording {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingId::new(id),
            artifact_slots: Vec::new(),
            status: RecordingStatus::Prepared,
            stopped_at: None,
        }
    }

    pub fn reconstitute(
        id: RecordingId,
        participant_id: Option<ParticipantId>,
        status: RecordingStatus,
        artifact_id: Option<RecordingArtifactId>,
    ) -> Self {
        let artifact_slots = participant_id
            .map(|participant_id| {
                vec![RecordingArtifactSlot::reconstitute(
                    participant_id,
                    artifact_id,
                )]
            })
            .unwrap_or_default();

        Self {
            id,
            artifact_slots,
            status,
            stopped_at: None,
        }
    }

    pub fn reconstitute_with_artifact_slots(
        id: RecordingId,
        artifact_slots: Vec<RecordingArtifactSlot>,
        status: RecordingStatus,
        stopped_at: Option<SystemTime>,
    ) -> Self {
        Self {
            id,
            artifact_slots,
            status,
            stopped_at,
        }
    }

    pub fn id(&self) -> &RecordingId {
        &self.id
    }

    pub fn status(&self) -> RecordingStatus {
        self.status
    }

    pub fn stopped_at(&self) -> Option<SystemTime> {
        self.stopped_at
    }

    pub fn artifact_slots(&self) -> &[RecordingArtifactSlot] {
        &self.artifact_slots
    }

    pub fn expected_participant_ids(&self) -> impl Iterator<Item = &ParticipantId> {
        self.artifact_slots
            .iter()
            .map(RecordingArtifactSlot::participant_id)
    }

    pub fn artifact_for_participant(
        &self,
        participant_id: &ParticipantId,
    ) -> Option<&RecordingArtifactId> {
        self.artifact_slots
            .iter()
            .find(|slot| slot.participant_id() == participant_id)
            .and_then(RecordingArtifactSlot::artifact_id)
    }

    pub fn set_expected_participants(
        &mut self,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<(), RecordingLifecycleError> {
        if self.status != RecordingStatus::Prepared {
            return Err(RecordingLifecycleError::InvalidTransition {
                from: self.status,
                to: RecordingStatus::Prepared,
            });
        }

        let participants: Vec<_> = participants.into_iter().collect();
        if participants.is_empty() {
            return Err(RecordingLifecycleError::NoExpectedParticipants);
        }

        for (index, participant_id) in participants.iter().enumerate() {
            if participants[..index].contains(participant_id) {
                return Err(RecordingLifecycleError::DuplicateExpectedParticipant {
                    participant_id: participant_id.clone(),
                });
            }
        }

        if !self.artifact_slots.is_empty() {
            let existing: Vec<_> = self
                .artifact_slots
                .iter()
                .map(|slot| slot.participant_id().clone())
                .collect();
            if existing == participants {
                return Ok(());
            }
            return Err(RecordingLifecycleError::InvalidTransition {
                from: self.status,
                to: RecordingStatus::Prepared,
            });
        }

        self.artifact_slots = participants
            .into_iter()
            .map(RecordingArtifactSlot::new)
            .collect();
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), RecordingLifecycleError> {
        if self.artifact_slots.is_empty() {
            return Err(RecordingLifecycleError::NoExpectedParticipants);
        }
        self.transition_to(RecordingStatus::Recording)?;
        Ok(())
    }

    pub fn stop_at(&mut self, timestamp: SystemTime) -> Result<(), RecordingLifecycleError> {
        self.transition_to(RecordingStatus::Stopped)?;
        self.stopped_at = Some(timestamp);
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), RecordingLifecycleError> {
        self.stop_at(SystemTime::now())
    }

    pub fn complete_for_participant(
        &mut self,
        participant_id: &ParticipantId,
        artifact_id: RecordingArtifactId,
    ) -> Result<bool, RecordingLifecycleError> {
        if !matches!(
            self.status,
            RecordingStatus::Stopped | RecordingStatus::Completed
        ) {
            return Err(RecordingLifecycleError::InvalidTransition {
                from: self.status,
                to: RecordingStatus::Completed,
            });
        }

        let slot = self
            .artifact_slots
            .iter_mut()
            .find(|slot| slot.participant_id() == participant_id)
            .ok_or_else(|| RecordingLifecycleError::ParticipantNotExpected {
                participant_id: participant_id.clone(),
            })?;

        let changed = slot.complete(artifact_id)?;
        if self.all_artifacts_confirmed() {
            self.status = RecordingStatus::Completed;
        }

        Ok(changed)
    }

    pub fn all_artifacts_confirmed(&self) -> bool {
        !self.artifact_slots.is_empty()
            && self
                .artifact_slots
                .iter()
                .all(|slot| slot.artifact_id().is_some())
    }

    pub fn has_pending_artifacts(&self) -> bool {
        !self.artifact_slots.is_empty() && !self.all_artifacts_confirmed()
    }

    pub fn has_expected_participant(&self, participant_id: &ParticipantId) -> bool {
        self.artifact_slots
            .iter()
            .any(|slot| slot.participant_id() == participant_id)
    }

    pub fn should_timeout(&self, now: SystemTime, timeout: std::time::Duration) -> bool {
        self.status == RecordingStatus::Stopped
            && self.has_pending_artifacts()
            && self
                .stopped_at
                .and_then(|stopped_at| now.duration_since(stopped_at).ok())
                .is_some_and(|elapsed| elapsed >= timeout)
    }

    fn transition_to(&mut self, target: RecordingStatus) -> Result<(), RecordingLifecycleError> {
        if !matches!(
            (self.status, target),
            (RecordingStatus::Prepared, RecordingStatus::Recording)
                | (RecordingStatus::Recording, RecordingStatus::Stopped)
                | (RecordingStatus::Stopped, RecordingStatus::Completed)
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
    use std::time::{Duration, UNIX_EPOCH};

    fn participant_id(value: &str) -> ParticipantId {
        ParticipantId::new(value)
    }

    fn artifact_id(value: &str) -> RecordingArtifactId {
        RecordingArtifactId::new(value)
    }

    fn prepared_recording() -> Recording {
        let mut recording = Recording::new("recording-test");
        recording
            .set_expected_participants([
                participant_id("alice"),
                participant_id("bob"),
                participant_id("carol"),
            ])
            .unwrap();
        recording
    }

    #[test]
    fn new_recording_starts_as_prepared_without_artifact_slots() {
        let recording = Recording::new("recording-test-01");
        assert_eq!(recording.status(), RecordingStatus::Prepared);
        assert!(recording.artifact_slots().is_empty());
    }

    #[test]
    fn expected_participants_are_fixed_before_recording() {
        let recording = prepared_recording();
        assert_eq!(
            recording
                .expected_participant_ids()
                .map(ParticipantId::value)
                .collect::<Vec<_>>(),
            vec!["alice", "bob", "carol"]
        );
    }

    #[test]
    fn duplicate_expected_participant_is_rejected() {
        let mut recording = Recording::new("recording-test-02");
        assert_eq!(
            recording.set_expected_participants([
                participant_id("alice"),
                participant_id("alice"),
            ]),
            Err(RecordingLifecycleError::DuplicateExpectedParticipant {
                participant_id: participant_id("alice"),
            })
        );
    }

    #[test]
    fn prepared_recording_can_start_only_after_expected_participants_exist() {
        let mut recording = prepared_recording();
        assert_eq!(recording.start(), Ok(()));
        assert_eq!(recording.status(), RecordingStatus::Recording);
    }

    #[test]
    fn recording_stop_persists_stop_time() {
        let mut recording = prepared_recording();
        let stopped_at = UNIX_EPOCH + Duration::from_secs(100);
        recording.start().unwrap();
        recording.stop_at(stopped_at).unwrap();
        assert_eq!(recording.status(), RecordingStatus::Stopped);
        assert_eq!(recording.stopped_at(), Some(stopped_at));
    }

    #[test]
    fn one_artifact_does_not_complete_multi_track_recording() {
        let mut recording = prepared_recording();
        recording.start().unwrap();
        recording.stop_at(UNIX_EPOCH + Duration::from_secs(100)).unwrap();

        assert_eq!(
            recording.complete_for_participant(&participant_id("alice"), artifact_id("alice-1")),
            Ok(true)
        );
        assert_eq!(recording.status(), RecordingStatus::Stopped);
        assert_eq!(
            recording.artifact_for_participant(&participant_id("alice")).map(RecordingArtifactId::value),
            Some("alice-1")
        );
    }

    #[test]
    fn last_artifact_completes_multi_track_recording() {
        let mut recording = prepared_recording();
        recording.start().unwrap();
        recording.stop_at(UNIX_EPOCH + Duration::from_secs(100)).unwrap();

        recording
            .complete_for_participant(&participant_id("alice"), artifact_id("alice-1"))
            .unwrap();
        recording
            .complete_for_participant(&participant_id("bob"), artifact_id("bob-1"))
            .unwrap();
        assert_eq!(
            recording.complete_for_participant(&participant_id("carol"), artifact_id("carol-1")),
            Ok(true)
        );
        assert_eq!(recording.status(), RecordingStatus::Completed);
        assert!(recording.all_artifacts_confirmed());
    }

    #[test]
    fn same_artifact_completion_is_idempotent() {
        let mut recording = prepared_recording();
        recording.start().unwrap();
        recording.stop().unwrap();
        recording
            .complete_for_participant(&participant_id("alice"), artifact_id("alice-1"))
            .unwrap();
        assert_eq!(
            recording.complete_for_participant(&participant_id("alice"), artifact_id("alice-1")),
            Ok(false)
        );
        assert_eq!(recording.status(), RecordingStatus::Stopped);
    }

    #[test]
    fn different_artifact_for_same_participant_is_rejected() {
        let mut recording = prepared_recording();
        recording.start().unwrap();
        recording.stop().unwrap();
        recording
            .complete_for_participant(&participant_id("alice"), artifact_id("alice-1"))
            .unwrap();
        assert_eq!(
            recording.complete_for_participant(&participant_id("alice"), artifact_id("alice-2")),
            Err(RecordingLifecycleError::ArtifactConflict {
                existing: artifact_id("alice-1"),
                requested: artifact_id("alice-2"),
            })
        );
    }

    #[test]
    fn unexpected_participant_cannot_complete_slot() {
        let mut recording = prepared_recording();
        recording.start().unwrap();
        recording.stop().unwrap();
        assert_eq!(
            recording.complete_for_participant(&participant_id("dave"), artifact_id("dave-1")),
            Err(RecordingLifecycleError::ParticipantNotExpected {
                participant_id: participant_id("dave"),
            })
        );
    }

    #[test]
    fn stopped_recording_times_out_only_when_artifacts_are_pending() {
        let mut recording = prepared_recording();
        let stopped_at = UNIX_EPOCH + Duration::from_secs(100);
        recording.start().unwrap();
        recording.stop_at(stopped_at).unwrap();
        assert!(!recording.should_timeout(
            stopped_at + Duration::from_secs(23 * 60 * 60 + 59 * 60),
            Duration::from_secs(24 * 60 * 60),
        ));
        assert!(recording.should_timeout(
            stopped_at + Duration::from_secs(24 * 60 * 60),
            Duration::from_secs(24 * 60 * 60),
        ));
        for participant in ["alice", "bob", "carol"] {
            recording
                .complete_for_participant(
                    &participant_id(participant),
                    artifact_id(format!("{participant}-1")),
                )
                .unwrap();
        }
        assert!(!recording.should_timeout(
            stopped_at + Duration::from_secs(48 * 60 * 60),
            Duration::from_secs(24 * 60 * 60),
        ));
    }
}
