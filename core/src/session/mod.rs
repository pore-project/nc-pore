pub mod repository;

use crate::activity::{ActivityEvent, ActivityType};
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::{Recording, RecordingArtifactId, RecordingId, RecordingLifecycleError};

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
        Self {
            id,
            status: ProductionStatus::Created,
            participations: Vec::new(),
            recordings: Vec::new(),
            activities: vec![ActivityEvent::new(ActivityType::SessionCreated)],
        }
    }

    pub fn start(&mut self) -> Result<(), ProductionSessionError> {
        if self.status != ProductionStatus::Created {
            return Err(ProductionSessionError::InvalidStateTransition);
        }

        self.status = ProductionStatus::Active;

        self.activities
            .push(ActivityEvent::new(ActivityType::SessionStarted));

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

    pub fn complete(&mut self) -> Result<(), ProductionSessionError> {
        if self.status != ProductionStatus::Active {
            return Err(ProductionSessionError::InvalidStateTransition);
        }

        if !self.has_owner() {
            return Err(ProductionSessionError::MissingOwner);
        }

        self.status = ProductionStatus::Completed;

        self.activities
            .push(ActivityEvent::new(ActivityType::SessionCompleted));

        Ok(())
    }

    pub fn add_participation(
        &mut self,
        participation: Participation,
    ) -> Result<(), ProductionSessionError> {
        if self.has_participant(&participation.participant_id) {
            return Err(ProductionSessionError::ParticipantAlreadyExists);
        }

        self.participations.push(participation);

        Ok(())
    }

    /// Adds a recording to the production session.
    ///
    /// The production session owns the relationship between
    /// production and recordings.
    pub fn add_recording(&mut self, recording: Recording) {
        self.recordings.push(recording);
    }

    /// Starts the domain recording owned by this production session.
    ///
    /// The aggregate delegates the recording lifecycle transition to the
    /// owned Recording rather than exposing mutable recording state.
    pub fn start_recording(
        &mut self,
        recording_id: &RecordingId,
    ) -> Result<(), ProductionSessionError> {
        let recording = self
            .recordings
            .iter_mut()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;

        recording
            .start()
            .map_err(ProductionSessionError::RecordingLifecycle)
    }

    /// Completes the domain recording owned by this production session and
    /// associates it with the opaque technical artifact identity.
    pub fn complete_recording(
        &mut self,
        recording_id: &RecordingId,
        artifact_id: RecordingArtifactId,
    ) -> Result<(), ProductionSessionError> {
        let recording = self
            .recordings
            .iter_mut()
            .find(|recording| recording.id() == recording_id)
            .ok_or(ProductionSessionError::RecordingNotFound)?;

        recording
            .complete(artifact_id)
            .map_err(ProductionSessionError::RecordingLifecycle)
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
        Participation {
            participant_id: ParticipantId::new(id),
            role,
        }
    }

    #[test]
    fn new_session_starts_as_created() {
        let session = create_test_session();

        assert_eq!(session.status(), ProductionStatus::Created);
    }

    #[test]
    fn starting_session_changes_status_to_active() {
        let mut session = create_test_session();

        session.start().unwrap();

        assert_eq!(session.status(), ProductionStatus::Active);
    }

    #[test]
    fn completed_session_cannot_be_started_again() {
        let mut session = create_test_session();

        session.start().unwrap();

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        session.complete().unwrap();

        assert_eq!(
            session.start(),
            Err(ProductionSessionError::InvalidStateTransition)
        );
    }

    #[test]
    fn completing_active_session_without_owner_fails() {
        let mut session = create_test_session();

        session.start().unwrap();

        assert_eq!(
            session.complete(),
            Err(ProductionSessionError::MissingOwner)
        );
    }

    #[test]
    fn completing_session_with_owner_changes_status_to_completed() {
        let mut session = create_test_session();

        session.start().unwrap();

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        assert!(session.complete().is_ok());
        assert_eq!(session.status(), ProductionStatus::Completed);
    }

    #[test]
    fn participation_can_be_added_to_session() {
        let mut session = create_test_session();

        assert!(
            session
                .add_participation(create_participation(
                    "participant-1",
                    ParticipantRole::Participant,
                ))
                .is_ok()
        );

        assert_eq!(session.participant_count(), 1);
    }

    #[test]
    fn duplicate_participant_cannot_be_added_to_session() {
        let mut session = create_test_session();

        session
            .add_participation(create_participation(
                "participant-1",
                ParticipantRole::Participant,
            ))
            .unwrap();

        assert_eq!(
            session.add_participation(create_participation(
                "participant-1",
                ParticipantRole::Guest,
            )),
            Err(ProductionSessionError::ParticipantAlreadyExists)
        );
    }

    #[test]
    fn session_can_check_owner() {
        let mut session = create_test_session();

        assert!(!session.has_owner());

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        assert!(session.has_owner());
    }

    #[test]
    fn session_exposes_participations_read_only() {
        let mut session = create_test_session();

        session
            .add_participation(create_participation(
                "participant-1",
                ParticipantRole::Participant,
            ))
            .unwrap();

        assert_eq!(session.participations().len(), 1);
        assert_eq!(
            session.participations()[0].participant_id,
            ParticipantId::new("participant-1")
        );
    }

    #[test]
    fn recording_can_be_added_to_session() {
        let mut session = create_test_session();

        session.add_recording(Recording::new("recording-session-test"));

        assert_eq!(session.recordings().len(), 1);
    }

    #[test]
    fn session_lifecycle_creates_activity_history() {
        let mut session = create_test_session();

        assert_eq!(session.activities().len(), 1);
        assert_eq!(
            session.activities()[0].activity_type,
            ActivityType::SessionCreated
        );

        session.start().unwrap();

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        session.complete().unwrap();

        assert_eq!(session.activities().len(), 3);

        assert_eq!(
            session.activities()[1].activity_type,
            ActivityType::SessionStarted
        );

        assert_eq!(
            session.activities()[2].activity_type,
            ActivityType::SessionCompleted
        );
    }

    #[test]
    fn session_can_start_owned_recording() {
        let mut session = create_test_session();
        let recording_id = RecordingId::new("recording-001");
        session.add_recording(Recording::new(recording_id.value()));

        session.start_recording(&recording_id).unwrap();

        assert_eq!(session.recordings()[0].status(), crate::recording::RecordingStatus::Recording);
    }

    #[test]
    fn session_can_complete_owned_recording_with_artifact() {
        let mut session = create_test_session();
        let recording_id = RecordingId::new("recording-001");
        let artifact_id = RecordingArtifactId::new("artifact-001");
        session.add_recording(Recording::new(recording_id.value()));

        session.start_recording(&recording_id).unwrap();
        session
            .complete_recording(&recording_id, artifact_id.clone())
            .unwrap();

        assert_eq!(
            session.recordings()[0].artifact_id(),
            Some(&artifact_id)
        );
    }

    #[test]
    fn session_rejects_unknown_recording_for_lifecycle_operations() {
        let mut session = create_test_session();
        let recording_id = RecordingId::new("missing");

        assert_eq!(
            session.start_recording(&recording_id),
            Err(ProductionSessionError::RecordingNotFound)
        );
    }
}
