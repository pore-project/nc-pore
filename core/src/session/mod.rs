use crate::identity::ProductionId;
use crate::participation::Participation;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
    Active,
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

    /// Adds a participation to the production session.
    ///
    /// A participant can only participate once within the same session.
    ///
    /// See ADR-019 and ADR-031.
    pub fn add_participation(&mut self, participation: Participation) -> Result<(), String> {
        if self
            .participations
            .iter()
            .any(|existing| existing.participant_id == participation.participant_id)
        {
            return Err(String::from(
                "Participant already exists in this production session",
            ));
        }

        self.participations.push(participation);

        Ok(())
    }
}
