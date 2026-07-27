use crate::identity::ProductionId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
}

#[derive(Debug, Clone)]
pub struct ProductionSession {
    pub id: ProductionId,
    pub status: ProductionStatus,
}

impl ProductionSession {
    pub fn new(id: ProductionId) -> Self {
        Self {
            id,
            status: ProductionStatus::Created,
        }
    }
}
