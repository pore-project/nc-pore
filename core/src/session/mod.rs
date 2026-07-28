use crate::identity::ProductionId;
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

    pub fn complete(&mut self) {
        self.status = ProductionStatus::Completed;
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
        if self
            .participations
            .iter()
            .any(|existing| existing.participant_id == participation.participant_id)
        {
            return Err(ProductionSessionError::ParticipantAlreadyExists);
        }

        self.participations.push(participation);

        Ok(())
    }

    /// Checks whether a specific role exists within this production session.
    ///
    /// Roles describe responsibility within a production,
    /// not the identity of a participant.
    ///
    /// See ADR-031.
    pub fn has_role(&self, role: ParticipantRole) -> bool {
        self.participations
            .iter()
            .any(|participation| participation.role == role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ProductionId;
    use crate::participant::ParticipantId;
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

        assert_eq!(session.status, ProductionStatus::Created);
    }

    #[test]
    fn starting_session_changes_status_to_active() {
        let mut session = create_test_session();

        session.start();

        assert_eq!(session.status, ProductionStatus::Active);
    }

    #[test]
    fn completing_session_changes_status_to_completed() {
        let mut session = create_test_session();

        session.complete();

        assert_eq!(session.status, ProductionStatus::Completed);
    }

    #[test]
    fn participation_can_be_added_to_session() {
        let mut session = create_test_session();

        let participation = create_participation("participant-1", ParticipantRole::Participant);

        assert!(session.add_participation(participation).is_ok());
        assert_eq!(session.participations.len(), 1);
    }

    #[test]
    fn duplicate_participant_cannot_be_added_to_session() {
        let mut session = create_test_session();

        let first = create_participation("participant-1", ParticipantRole::Participant);

        let second = create_participation("participant-1", ParticipantRole::Guest);

        session.add_participation(first).unwrap();

        assert_eq!(
            session.add_participation(second),
            Err(ProductionSessionError::ParticipantAlreadyExists)
        );
    }

    #[test]
    fn session_can_check_existing_roles() {
        let mut session = create_test_session();

        session
            .add_participation(create_participation("owner-1", ParticipantRole::Owner))
            .unwrap();

        assert!(session.has_role(ParticipantRole::Owner));
        assert!(!session.has_role(ParticipantRole::Producer));
    }
}
