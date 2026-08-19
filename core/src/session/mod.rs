pub mod repository;

use crate::activity::{ActivityEvent, ActivityResult, ActivityType};
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::{Recording, RecordingArtifactId, RecordingId, RecordingLifecycleError};
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
    InvalidStateTransition,
    Unauthorized,
    RecordingLifecycle(RecordingLifecycleError),
}

#[derive(Debug, Clone)]
pub struct ProductionSession {
    pub id: ProductionId,
    status: ProductionStatus,
    participations: Vec<Participation>,
    recordings: Vec<Recording>,
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

        self.push_activity(
            Some(actor.clone()),
            ActivityType::RecordingStarted,
            Some(recording_id.value().to_owned()),
        );

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::ParticipantRole;

    fn create_test_session() -> ProductionSession {
        ProductionSession::new(ProductionId::new("test-session"))
    }

    fn create_participation(id: &str, role: ParticipantRole) -> Participation {
        Participation::new(ParticipantId::new(id), role)
    }

    fn add_owner(session: &mut ProductionSession) -> ParticipantId {
        let owner = ParticipantId::new("owner-1");
        session
            .add_participation_by(
                &owner,
                create_participation("owner-1", ParticipantRole::Owner),
            )
            .unwrap();
        owner
    }

    #[test]
    fn new_session_starts_as_created() {
        let session = create_test_session();
        assert_eq!(session.status(), ProductionStatus::Created);
    }

    #[test]
    fn session_creation_can_record_actor() {
        let owner = ParticipantId::new("owner-1");
        let session = ProductionSession::new_with_actor(
            ProductionId::new("test-session"),
            Some(owner.clone()),
        );

        assert_eq!(session.activities()[0].actor, Some(owner));
        assert_eq!(session.activities()[0].session_id, session.id);
    }

    #[test]
    fn owner_can_start_session() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);

        session.start_by(&owner).unwrap();

        assert_eq!(session.status(), ProductionStatus::Active);
    }

    #[test]
    fn completed_session_cannot_be_started_again() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();

        assert_eq!(
            session.start_by(&owner),
            Err(ProductionSessionError::InvalidStateTransition)
        );
    }

    #[test]
    fn completing_active_session_without_owner_fails() {
        let mut session = create_test_session();
        let owner = ParticipantId::new("owner-1");

        session
            .add_participation_by(
                &owner,
                create_participation("owner-1", ParticipantRole::Participant),
            )
            .unwrap_err();

        assert_eq!(
            session.complete_by(&owner),
            Err(ProductionSessionError::Unauthorized)
        );
    }

    #[test]
    fn completing_session_with_owner_changes_status_to_completed() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();

        assert!(session.complete_by(&owner).is_ok());
        assert_eq!(session.status(), ProductionStatus::Completed);
    }

    #[test]
    fn duplicate_participant_cannot_be_added_to_session() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);

        session
            .add_participation_by(
                &owner,
                create_participation("participant-1", ParticipantRole::Participant),
            )
            .unwrap();

        assert_eq!(
            session.add_participation_by(
                &owner,
                create_participation("participant-1", ParticipantRole::Guest),
            ),
            Err(ProductionSessionError::ParticipantAlreadyExists)
        );
    }

    #[test]
    fn participant_cannot_manage_other_participants() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();
        let participant = ParticipantId::new("participant-1");
        session
            .add_participation_by(
                &owner,
                create_participation("participant-1", ParticipantRole::Participant),
            )
            .unwrap();

        assert_eq!(
            session.add_participation_by(
                &participant,
                create_participation("participant-2", ParticipantRole::Participant),
            ),
            Err(ProductionSessionError::Unauthorized)
        );
    }

    #[test]
    fn combined_owner_producer_participant_role_has_all_capabilities() {
        let mut session = create_test_session();
        let owner = ParticipantId::new("owner-1");
        let combined = Participation::with_roles(
            owner.clone(),
            [
                ParticipantRole::Owner,
                ParticipantRole::Producer,
                ParticipantRole::Participant,
            ],
        );

        session.add_participation_by(&owner, combined).unwrap();
        session.start_by(&owner).unwrap();

        assert_eq!(session.status(), ProductionStatus::Active);
    }

    #[test]
    fn session_lifecycle_creates_rich_activity_history() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();

        assert_eq!(session.activities().len(), 3);
        assert_eq!(
            session.activities()[0].activity_type,
            ActivityType::SessionCreated
        );
        assert_eq!(
            session.activities()[1].activity_type,
            ActivityType::SessionStarted
        );
        assert_eq!(session.activities()[1].actor, Some(owner.clone()));
        assert_eq!(
            session.activities()[2].activity_type,
            ActivityType::SessionCompleted
        );
        assert_eq!(session.activities()[2].session_id, session.id);
        assert_eq!(session.activities()[2].result, ActivityResult::Success);
    }

    #[test]
    fn completed_session_rejects_participant_mutations() {
        let mut session = create_test_session();
        let owner = add_owner(&mut session);
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();

        assert_eq!(
            session.add_participation_by(
                &owner,
                create_participation("participant-1", ParticipantRole::Participant),
            ),
            Err(ProductionSessionError::InvalidStateTransition)
        );
    }
}
