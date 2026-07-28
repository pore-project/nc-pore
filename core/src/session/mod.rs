use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::role::ParticipantRole;

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
}

#[derive(Debug, Clone)]
pub struct ProductionSession {
    pub id: ProductionId,
    pub status: ProductionStatus,
    pub participations: Vec<Participation>,
}

impl ProductionSession {
    pub fn new(id: ProductionId) -> Self {
        Self {
            id,
            status: ProductionStatus::Created,
            participations: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.status = ProductionStatus::Active;
    }

    /// Completes a production session.
    ///
    /// A production session requires an owner before completion.
    ///
    /// See ADR-031.
    pub fn complete(&mut self) -> Result<(), ProductionSessionError> {
        if !self.has_owner() {
            return Err(ProductionSessionError::MissingOwner);
        }

        self.status = ProductionStatus::Completed;

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
            .any(|participation| participation.role == ParticipantRole::Owner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        assert_eq!(session.status, ProductionStatus::Created);
    }

    #[test]
    fn starting_session_changes_status_to_active() {
        let mut session = create_test_session();

        session.start();

        assert_eq!(session.status, ProductionStatus::Active);
    }

    #[test]
    fn completing_session_without_owner_fails() {
        let mut session = create_test_session();

        assert_eq!(
            session.complete(),
            Err(ProductionSessionError::MissingOwner)
        );
    }

    #[test]
    fn completing_session_with_owner_changes_status_to_completed() {
        let mut session = create_test_session();

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        assert!(session.complete().is_ok());
        assert_eq!(session.status, ProductionStatus::Completed);
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

        assert_eq!(session.participations.len(), 1);
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
}
