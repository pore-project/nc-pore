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

    pub fn add_participation(&mut self, participation: Participation) {
        self.participations.push(participation);
    }
}
