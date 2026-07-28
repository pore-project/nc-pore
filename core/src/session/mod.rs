use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
    Active,
    Completed,
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

    /// Starts the production session.
    ///
    /// A session becomes active when production work begins.
    pub fn start(&mut self) {
        self.status = ProductionStatus::Active;
    }

    /// Completes the production session.
    ///
    /// A completed session is no longer actively produced.
    pub fn complete(&mut self) {
        self.status = ProductionStatus::Completed;
    }

    /// Adds a participation to the production session.
    ///
    /// A participant can only participate once within the same session.
    ///
    /// See ADR-019 and ADR-031.
    pub fn add_participation(&mut self, participation: Participation) -> Result<(), String> {
        if self.has_participant(&participation.participant_id) {
            return Err(String::from(
                "Participant already exists in this production session",
            ));
        }

        self.participations.push(participation);

        Ok(())
    }

    /// Returns all participations of this production session.
    ///
    /// The session owns the participation collection.
    pub fn participations(&self) -> &[Participation] {
        &self.participations
    }

    /// Checks whether a participant is already assigned to this session.
    ///
    /// A participant can only have one participation per production session.
    pub fn has_participant(&self, participant_id: &ParticipantId) -> bool {
        self.participations
            .iter()
            .any(|existing| &existing.participant_id == participant_id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::ProductionId;
    use crate::participant::ParticipantId;
    use crate::participation::Participation;
    use crate::role::ParticipantRole;

    fn create_test_session() -> ProductionSession {
        ProductionSession::new(ProductionId::new("test-production"))
    }

    fn create_test_participation() -> Participation {
        Participation {
            participant_id: ParticipantId::new("participant-1"),
            role: ParticipantRole::Guest,
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

        session.start();
        session.complete();

        assert_eq!(session.status, ProductionStatus::Completed);
    }

    #[test]
    fn participation_can_be_added_to_session() {
        let mut session = create_test_session();
        let participation = create_test_participation();

        let result = session.add_participation(participation);

        assert!(result.is_ok());
        assert_eq!(session.participations().len(), 1);
    }

    #[test]
    fn duplicate_participant_cannot_be_added_to_session() {
        let mut session = create_test_session();

        let first = create_test_participation();
        let second = create_test_participation();

        assert!(session.add_participation(first).is_ok());
        assert!(session.add_participation(second).is_err());

        assert_eq!(session.participations().len(), 1);
    }
}
