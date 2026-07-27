#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionId(String);

impl ProductionId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
