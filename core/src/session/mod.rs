pub mod repository;

use crate::activity::{ActivityEvent, ActivityResult, ActivityType};
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::{
    Recording, RecordingArtifactId, RecordingCoordination, RecordingCoordinationError, RecordingId,
    RecordingLifecycleError,
};
use crate::role::ProductionAction;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
    Active,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionSessionError {
    ParticipantAlreadyExists,
    MissingOwner,
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
        self.status = ProductionStatus::Completed;
        self.push_activity(Some(actor.clone()), ActivityType::SessionCompleted, None);
        Ok(())
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
        let target = recording.id().value().to_owned();
        self.recordings.push(recording);
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingAdded,
            Some(target),
        );
        Ok(())
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
        let mut coordination = RecordingCoordination::new(recording_id.clone(), participants)
            .map_err(ProductionSessionError::RecordingCoordination)?;
        coordination
            .begin_waiting_for_ready()
            .map_err(ProductionSessionError::RecordingCoordination)?;
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
            .start()
            .map_err(ProductionSessionError::RecordingLifecycle)?;
        recording.assign_participant(actor.clone());
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
            .complete(artifact_id)
            .map_err(ProductionSessionError::RecordingLifecycle)?;
        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingCompleted,
            Some(recording_id.value().to_owned()),
        );
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
