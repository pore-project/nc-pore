use crate::activity::{ActivityEvent, ActivityType};
use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::Recording;

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
    InvalidStateTransition,
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

    /// Completes a production session.
    ///
    /// A production session requires an owner before completion.
    ///
    /// See ADR-031.
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
    /// Adds a participation to the production session.
    ///
    /// A participant can only participate once within the same session.
    ///
    /// See ADR-019 and ADR-031.
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

    /// Checks whether a participant is already part of this production session.
    ///
    /// See ADR-031.
    pub fn has_participant(&self, participant_id: &ParticipantId) -> bool {
        self.participations
            .iter()
            .any(|participation| &participation.participant_id == participant_id)
    }

    /// Checks whether this production session has an owner.
    ///
    /// A production session requires ownership responsibility.
    ///
    /// See ADR-031.
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

    // TEST-01
    // Verify: A new production session starts in Created state.
    #[test]
    fn new_session_starts_as_created() {
        let session = create_test_session();

        assert_eq!(session.status(), ProductionStatus::Created);
    }

    // TEST-02
    // Verify: Starting a created session changes the state to Active.
    #[test]
    fn starting_session_changes_status_to_active() {
        let mut session = create_test_session();

        session.start().unwrap();

        assert_eq!(session.status(), ProductionStatus::Active);
    }

    // TEST-03
    // Verify: A completed session cannot transition back into Active state.
    //
    // Lifecycle:
    // Created -> Active -> Completed
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

    // TEST-04
    // Verify: An active session cannot be completed without an owner.
    #[test]
    fn completing_active_session_without_owner_fails() {
        let mut session = create_test_session();

        session.start().unwrap();

        assert_eq!(
            session.complete(),
            Err(ProductionSessionError::MissingOwner)
        );
    }

    // TEST-05
    // Verify: An active session with an owner can be completed.
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

    // TEST-06
    // Verify: A participation can be added to a production session.
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

    // TEST-07
    // Verify: A participant cannot be added twice to the same session.
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

    // TEST-08
    // Verify: Owner responsibility can be detected inside a session.
    #[test]
    fn session_can_check_owner() {
        let mut session = create_test_session();

        assert!(!session.has_owner());

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        assert!(session.has_owner());
    }

    // TEST-09
    // Verify: The public participation accessor exposes stored participations.
    //
    // This protects the read-only access boundary.
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
    // TEST-10
    // Verify: Lifecycle transitions create activity history entries.
    //
    // Lifecycle:
    // Created -> Active -> Completed
    //
    // Protects ADR-032 and ADR-035.
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
}
