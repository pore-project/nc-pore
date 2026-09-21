pub mod repository;

use crate::activity::{ActivityEvent, ActivityResult, ActivityType};
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::{
    Recording, RecordingArtifactId, RecordingCoordination, RecordingCoordinationError, RecordingId,
    RecordingLifecycleError, RecordingStatus,
};
use crate::role::ProductionAction;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
    Active,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionCompletionReason {
    AllRecordingsCompleted,
    ArtifactCompletionTimeout,
    HostForced,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionSessionError {
    ParticipantAlreadyExists,
    MissingOwner,
    RecordingAlreadyExists,
    RecordingNotFound,
    RecordingCoordinationNotFound,
    RecordingCoordinationAlreadyActive,
    InvalidStateTransition,
    Unauthorized,
    RecordingLifecycle(RecordingLifecycleError),
    RecordingCoordination(RecordingCoordinationError),
}

#[derive(Debug, Clone)]
pub struct ProductionSession {
    pub id: ProductionId,
    status: ProductionStatus,
    participations: Vec<Participation>,
    recordings: Vec<Recording>,
    recording_coordination: Option<RecordingCoordination>,
    activities: Vec<ActivityEvent>,
    completion_reason: Option<ProductionCompletionReason>,
}

impl ProductionSession {
    pub fn new(id: ProductionId) -> Self {
        Self::new_with_actor(id, None)
    }

    pub fn new_with_actor(id: ProductionId, actor: Option<ParticipantId>) -> Self {
        let activity = ActivityEvent::new(
            ActivityType::SessionCreated,
            id.clone(),
            actor,
            None,
            ActivityResult::Success,
        );
        Self {
            id,
            status: ProductionStatus::Created,
            participations: Vec::new(),
            recordings: Vec::new(),
            recording_coordination: None,
            activities: vec![activity],
            completion_reason: None,
        }
    }

    fn authorize(
        &self,
        actor: &ParticipantId,
        action: ProductionAction,
    ) -> Result<(), ProductionSessionError> {
        self.participations
            .iter()
            .find(|participation| &participation.participant_id == actor)
            .filter(|participation| participation.allows(action))
            .map(|_| ())
            .ok_or(ProductionSessionError::Unauthorized)
    }

    fn push_activity(
        &mut self,
        actor: Option<ParticipantId>,
        activity_type: ActivityType,
        target: Option<String>,
    ) {
        self.activities.push(ActivityEvent::new(
            activity_type,
            self.id.clone(),
            actor,
            target,
            ActivityResult::Success,
        ));
    }

    pub fn start_by(&mut self, actor: &ParticipantId) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::StartSession)?;
        if self.status != ProductionStatus::Created {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        self.status = ProductionStatus::Active;
        self.push_activity(Some(actor.clone()), ActivityType::SessionStarted, None);
        Ok(())
    }

    pub fn status(&self) -> ProductionStatus {
        self.status
    }

    pub fn completion_reason(&self) -> Option<ProductionCompletionReason> {
        self.completion_reason
    }

    pub fn participations(&self) -> &[Participation] {
        &self.participations
    }

    pub fn recordings(&self) -> &[Recording] {
        &self.recordings
    }

    pub fn recording_coordination(&self) -> Option<&RecordingCoordination> {
        self.recording_coordination.as_ref()
    }

    pub fn activities(&self) -> &[ActivityEvent] {
        &self.activities
    }

    pub fn participant_count(&self) -> usize {
        self.participations.len()
    }

    pub fn complete_by(&mut self, actor: &ParticipantId) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::CompleteSession)?;
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        if !self.has_owner() {
            return Err(ProductionSessionError::MissingOwner);
        }
        if self
            .recordings
            .iter()
            .any(|recording| recording.status() == RecordingStatus::Recording)
        {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        self.close_production(ProductionCompletionReason::HostForced, Some(actor.clone()));
        Ok(())
    }

    pub fn complete_due_to_artifact_timeout(
        &mut self,
        now: SystemTime,
        timeout: Duration,
    ) -> Result<bool, ProductionSessionError> {
        if self.status != ProductionStatus::Active {
            return Ok(false);
        }

        if self
            .recordings
            .iter()
            .any(|recording| recording.should_timeout(now, timeout))
        {
            self.close_production(ProductionCompletionReason::ArtifactCompletionTimeout, None);
            return Ok(true);
        }

        Ok(false)
    }

    fn close_production(
        &mut self,
        reason: ProductionCompletionReason,
        actor: Option<ParticipantId>,
    ) {
        self.status = ProductionStatus::Completed;
        self.completion_reason = Some(reason);
        self.push_activity(
            actor,
            ActivityType::SessionCompleted,
            Some(
                match reason {
                    ProductionCompletionReason::AllRecordingsCompleted => {
                        "all_recordings_completed"
                    }
                    ProductionCompletionReason::ArtifactCompletionTimeout => {
                        "artifact_completion_timeout"
                    }
                    ProductionCompletionReason::HostForced => "host_forced",
                }
                .to_owned(),
            ),
        );
    }

    pub fn add_participation_by(
        &mut self,
        actor: &ParticipantId,
        participation: Participation,
    ) -> Result<(), ProductionSessionError> {
        if self.has_participant(&participation.participant_id) {
            return Err(ProductionSessionError::ParticipantAlreadyExists);
        }
        if self.participations.is_empty() {
            if !participation.is_owner() || actor != &participation.participant_id {
                return Err(ProductionSessionError::Unauthorized);
            }
        } else {
            self.authorize(actor, ProductionAction::ManageParticipants)?;
        }
        if self.status == ProductionStatus::Completed {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let target = participation.participant_id.value().to_owned();
        self.participations.push(participation);
        self.push_activity(
            Some(actor.clone()),
            ActivityType::ParticipantAdded,
            Some(target),
        );
        Ok(())
    }

    pub fn add_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording: Recording,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ManageRecordings)?;
        if self.status == ProductionStatus::Completed {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        if self
            .recordings
            .iter()
            .any(|existing| existing.id() == recording.id())
        {
            return Err(ProductionSessionError::RecordingAlreadyExists);
        }
        let target = recording.id().value().to_owned();
        self.recordings.push(recording);
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingAdded,
            Some(target),
        );
        Ok(())
    }

    pub fn ensure_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        if self.status == ProductionStatus::Completed {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        if self
            .recordings
            .iter()
            .any(|recording| recording.id() == recording_id)
        {
            self.authorize(actor, ProductionAction::ParticipateInRecording)?;
            return Ok(());
        }
        self.authorize(actor, ProductionAction::ManageRecordings)?;
        self.add_recording_by(actor, Recording::new(recording_id.value()))
    }

    pub fn begin_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ManageRecordings)?;
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        if self.recording_coordination.is_some() {
            return Err(ProductionSessionError::RecordingCoordinationAlreadyActive);
        }
        let participants: Vec<_> = participants.into_iter().collect();
        for participant in &participants {
            self.participations
                .iter()
                .find(|participation| &participation.participant_id == participant)
                .filter(|participation| {
                    participation.allows(ProductionAction::ParticipateInRecording)
                })
                .ok_or(ProductionSessionError::Unauthorized)?;
        }
        let recording = self
            .recordings
            .iter_mut()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;
        recording
            .set_expected_participants(participants.clone())
            .map_err(ProductionSessionError::RecordingLifecycle)?;

        let mut coordination = RecordingCoordination::new(recording_id.clone(), participants)
            .map_err(ProductionSessionError::RecordingCoordination)?;
        coordination
            .begin_waiting_for_ready()
            .map_err(ProductionSessionError::RecordingCoordination)?;
        recording
            .start()
            .map_err(ProductionSessionError::RecordingLifecycle)?;
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingStarted,
            Some(recording_id.value().to_owned()),
        );
        self.recording_coordination = Some(coordination);
        Ok(())
    }

    pub fn mark_recording_ready_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<bool, ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;
        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;
        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }
        coordination
            .mark_ready(actor)
            .map_err(ProductionSessionError::RecordingCoordination)
    }

    pub fn trigger_recording_opening_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ManageRecordings)?;
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let recording = self
            .recordings
            .iter()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;
        if recording.status() != RecordingStatus::Recording {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;
        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }
        coordination
            .trigger_opening()
            .map_err(ProductionSessionError::RecordingCoordination)
    }

    pub fn confirm_recording_opening_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;
        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;
        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }
        coordination
            .confirm_opening(actor)
            .map_err(ProductionSessionError::RecordingCoordination)
    }

    pub fn start_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ManageRecordings)?;
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let coordination = self
            .recording_coordination
            .as_ref()
            .ok_or(ProductionSessionError::InvalidStateTransition)?;
        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let recording = self
            .recordings
            .iter_mut()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;
        if recording.status() == RecordingStatus::Recording {
            return Ok(());
        }
        recording
            .start()
            .map_err(ProductionSessionError::RecordingLifecycle)?;
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingStarted,
            Some(recording_id.value().to_owned()),
        );
        Ok(())
    }

    /// Persists the fachliche recording stop boundary before technical capture stop.
    pub fn stop_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }
        let recording = self
            .recordings
            .iter_mut()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;
        recording
            .stop()
            .map_err(ProductionSessionError::RecordingLifecycle)?;
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingStopped,
            Some(recording_id.value().to_owned()),
        );
        Ok(())
    }

    /// Records a technical stop acknowledgement without making it a completion barrier.
    pub fn acknowledge_recording_stop_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::ParticipateInRecording)?;
        let coordination = self
            .recording_coordination
            .as_mut()
            .ok_or(ProductionSessionError::RecordingCoordinationNotFound)?;
        if coordination.recording_id() != recording_id {
            return Err(ProductionSessionError::RecordingCoordinationNotFound);
        }
        coordination
            .acknowledge_stop(actor)
            .map_err(ProductionSessionError::RecordingCoordination)
    }

    pub fn complete_recording_by(
        &mut self,
        actor: &ParticipantId,
        recording_id: &RecordingId,
        artifact_id: RecordingArtifactId,
    ) -> Result<(), ProductionSessionError> {
        self.authorize(actor, ProductionAction::CompleteRecordingArtifact)?;
        if self.status == ProductionStatus::Created {
            return Err(ProductionSessionError::InvalidStateTransition);
        }

        let (changed, recording_completed) = {
            let recording = self
                .recordings
                .iter_mut()
                .find(|recording| recording.id() == recording_id)
                .ok_or(ProductionSessionError::RecordingNotFound)?;
            let was_completed = recording.status() == RecordingStatus::Completed;
            let changed = recording
                .complete_for_participant(actor, artifact_id)
                .map_err(ProductionSessionError::RecordingLifecycle)?;
            (
                changed,
                !was_completed && recording.status() == RecordingStatus::Completed,
            )
        };

        if changed {
            self.push_activity(
                Some(actor.clone()),
                ActivityType::RecordingArtifactCompleted,
                Some(recording_id.value().to_owned()),
            );
        }

        if recording_completed {
            self.push_activity(
                Some(actor.clone()),
                ActivityType::RecordingCompleted,
                Some(recording_id.value().to_owned()),
            );
        }

        if self.status == ProductionStatus::Active
            && recording_completed
            && !self.recordings.is_empty()
            && self
                .recordings
                .iter()
                .all(|recording| recording.status() == RecordingStatus::Completed)
        {
            self.close_production(
                ProductionCompletionReason::AllRecordingsCompleted,
                Some(actor.clone()),
            );
        }

        Ok(())
    }

    pub fn has_participant(&self, participant_id: &ParticipantId) -> bool {
        self.participations
            .iter()
            .any(|participation| &participation.participant_id == participant_id)
    }

    pub fn has_owner(&self) -> bool {
        self.participations
            .iter()
            .any(|participation| participation.is_owner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::ParticipantRole;

    fn active_session() -> (ProductionSession, ParticipantId) {
        let owner = ParticipantId::new("owner-1");
        let mut session = ProductionSession::new_with_actor(
            ProductionId::new("production-1"),
            Some(owner.clone()),
        );
        session
            .add_participation_by(
                &owner,
                Participation::with_roles(
                    owner.clone(),
                    [ParticipantRole::Owner, ParticipantRole::Producer],
                ),
            )
            .unwrap();
        session.start_by(&owner).unwrap();
        (session, owner)
    }

    #[test]
    fn recording_start_requires_coordination() {
        let (mut session, owner) = active_session();
        let recording_id = RecordingId::new("recording-1");
        session
            .add_recording_by(&owner, Recording::new(recording_id.value()))
            .unwrap();

        assert_eq!(
            session.start_recording_by(&owner, &recording_id),
            Err(ProductionSessionError::InvalidStateTransition)
        );
    }

    #[test]
    fn recording_starts_before_ready_and_opening_is_a_separate_barrier() {
        let (mut session, owner) = active_session();
        let bob = ParticipantId::new("participant-1");
        session
            .add_participation_by(
                &owner,
                Participation::new(bob.clone(), ParticipantRole::Participant),
            )
            .unwrap();
        let recording_id = RecordingId::new("recording-1");
        session
            .add_recording_by(&owner, Recording::new(recording_id.value()))
            .unwrap();
        session
            .begin_recording_by(&owner, &recording_id, [owner.clone(), bob.clone()])
            .unwrap();

        assert_eq!(
            session.recordings()[0].status(),
            RecordingStatus::Recording
        );
        assert_eq!(session.start_recording_by(&owner, &recording_id), Ok(()));

        session
            .mark_recording_ready_by(&owner, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();

        assert_eq!(
            session.confirm_recording_opening_by(&owner, &recording_id),
            Err(ProductionSessionError::RecordingCoordination(
                RecordingCoordinationError::InvalidState
            ))
        );

        session
            .trigger_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();

        assert_eq!(session.start_recording_by(&owner, &recording_id), Ok(()));
    }

    #[test]
    fn production_rejects_duplicate_recording_ids() {
        let (mut session, owner) = active_session();
        let recording_id = RecordingId::new("recording-1");

        session
            .add_recording_by(&owner, Recording::new(recording_id.value()))
            .unwrap();
        let result = session.add_recording_by(&owner, Recording::new(recording_id.value()));

        assert_eq!(result, Err(ProductionSessionError::RecordingAlreadyExists));
        assert_eq!(session.recordings().len(), 1);
    }

    #[test]
    fn ensure_recording_is_idempotent_for_the_same_recording_id() {
        let (mut session, owner) = active_session();
        let recording_id = RecordingId::new("recording-1");

        session.ensure_recording_by(&owner, &recording_id).unwrap();
        session.ensure_recording_by(&owner, &recording_id).unwrap();

        assert_eq!(session.recordings().len(), 1);
        assert_eq!(session.recordings()[0].id(), &recording_id);
        assert_eq!(
            session
                .activities()
                .iter()
                .filter(|activity| activity.activity_type == ActivityType::RecordingAdded)
                .count(),
            1
        );
    }

    #[test]
    fn participant_can_ensure_an_existing_recording_without_mutating_it() {
        let (mut session, owner) = active_session();
        let participant = ParticipantId::new("participant-1");
        let recording_id = RecordingId::new("recording-1");

        session
            .add_participation_by(
                &owner,
                Participation::new(participant.clone(), ParticipantRole::Participant),
            )
            .unwrap();
        session.ensure_recording_by(&owner, &recording_id).unwrap();

        let activity_count = session.activities().len();
        session
            .ensure_recording_by(&participant, &recording_id)
            .unwrap();

        assert_eq!(session.recordings().len(), 1);
        assert_eq!(session.activities().len(), activity_count);
    }

    #[test]
    fn participant_artifacts_complete_recording_independently() {
        let (mut session, owner) = active_session();
        let bob = ParticipantId::new("bob");
        session
            .add_participation_by(
                &owner,
                Participation::new(bob.clone(), ParticipantRole::Participant),
            )
            .unwrap();

        let recording_id = RecordingId::new("recording-1");
        session.ensure_recording_by(&owner, &recording_id).unwrap();
        session
            .begin_recording_by(&owner, &recording_id, [owner.clone(), bob.clone()])
            .unwrap();
        session
            .mark_recording_ready_by(&owner, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&owner, &recording_id).unwrap();
        session.stop_recording_by(&owner, &recording_id).unwrap();

        session
            .complete_recording_by(
                &owner,
                &recording_id,
                RecordingArtifactId::new("alice-artifact"),
            )
            .unwrap();

        let recording = &session.recordings()[0];
        assert_eq!(recording.status(), RecordingStatus::Stopped);
        assert_eq!(
            recording.artifact_for_participant(&owner).unwrap().value(),
            "alice-artifact"
        );
        assert_eq!(session.status(), ProductionStatus::Active);

        session
            .complete_recording_by(
                &bob,
                &recording_id,
                RecordingArtifactId::new("bob-artifact"),
            )
            .unwrap();

        assert_eq!(session.recordings()[0].status(), RecordingStatus::Completed);
        assert_eq!(session.status(), ProductionStatus::Completed);
        assert_eq!(
            session.completion_reason(),
            Some(ProductionCompletionReason::AllRecordingsCompleted)
        );
    }

    #[test]
    fn late_artifact_completion_does_not_reopen_completed_production() {
        let (mut session, owner) = active_session();
        let bob = ParticipantId::new("bob");
        session
            .add_participation_by(
                &owner,
                Participation::new(bob.clone(), ParticipantRole::Participant),
            )
            .unwrap();

        let recording_id = RecordingId::new("recording-1");
        session.ensure_recording_by(&owner, &recording_id).unwrap();
        session
            .begin_recording_by(&owner, &recording_id, [owner.clone(), bob.clone()])
            .unwrap();
        session
            .mark_recording_ready_by(&owner, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&owner, &recording_id).unwrap();
        session.stop_recording_by(&owner, &recording_id).unwrap();

        session
            .complete_recording_by(
                &owner,
                &recording_id,
                RecordingArtifactId::new("alice-artifact"),
            )
            .unwrap();
        session.complete_by(&owner).unwrap();

        assert_eq!(session.status(), ProductionStatus::Completed);
        assert_eq!(
            session.completion_reason(),
            Some(ProductionCompletionReason::HostForced)
        );

        session
            .complete_recording_by(
                &bob,
                &recording_id,
                RecordingArtifactId::new("bob-artifact"),
            )
            .unwrap();

        assert_eq!(session.recordings()[0].status(), RecordingStatus::Completed);
        assert_eq!(session.status(), ProductionStatus::Completed);
        assert_eq!(
            session.completion_reason(),
            Some(ProductionCompletionReason::HostForced)
        );
    }

    #[test]
    fn timeout_closes_production_without_invalidating_pending_artifact() {
        let (mut session, owner) = active_session();
        let bob = ParticipantId::new("bob");
        session
            .add_participation_by(
                &owner,
                Participation::new(bob.clone(), ParticipantRole::Participant),
            )
            .unwrap();

        let recording_id = RecordingId::new("recording-1");
        session.ensure_recording_by(&owner, &recording_id).unwrap();
        session
            .begin_recording_by(&owner, &recording_id, [owner.clone(), bob.clone()])
            .unwrap();
        session
            .mark_recording_ready_by(&owner, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&owner, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&owner, &recording_id).unwrap();

        let stopped_at = std::time::UNIX_EPOCH + Duration::from_secs(100);
        {
            let recording = session
                .recordings
                .iter_mut()
                .find(|recording| recording.id() == &recording_id)
                .unwrap();
            recording.stop_at(stopped_at).unwrap();
        }

        session
            .complete_recording_by(
                &owner,
                &recording_id,
                RecordingArtifactId::new("alice-artifact"),
            )
            .unwrap();

        let changed = session
            .complete_due_to_artifact_timeout(
                stopped_at + Duration::from_secs(24 * 60 * 60),
                Duration::from_secs(24 * 60 * 60),
            )
            .unwrap();

        assert!(changed);
        assert_eq!(session.status(), ProductionStatus::Completed);
        assert_eq!(
            session.completion_reason(),
            Some(ProductionCompletionReason::ArtifactCompletionTimeout)
        );
        assert!(session.recordings()[0].has_pending_artifacts());

        session
            .complete_recording_by(
                &bob,
                &recording_id,
                RecordingArtifactId::new("bob-artifact"),
            )
            .unwrap();

        assert_eq!(session.recordings()[0].status(), RecordingStatus::Completed);
        assert_eq!(session.status(), ProductionStatus::Completed);
    }

    #[test]
    fn participant_cannot_ensure_a_missing_recording() {
        let (mut session, owner) = active_session();
        let participant = ParticipantId::new("participant-1");
        let recording_id = RecordingId::new("recording-1");

        session
            .add_participation_by(
                &owner,
                Participation::new(participant.clone(), ParticipantRole::Participant),
            )
            .unwrap();

        let result = session.ensure_recording_by(&participant, &recording_id);

        assert_eq!(result, Err(ProductionSessionError::Unauthorized));
        assert!(session.recordings().is_empty());
    }
}
