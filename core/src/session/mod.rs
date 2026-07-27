#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStatus {
    Created,
}

#[derive(Debug, Clone)]
pub struct ProductionSession {
    pub id: String,
    pub status: ProductionStatus,
}

impl ProductionSession {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: ProductionStatus::Created,
        }
    }
}
