#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParticipantId(String);

impl ParticipantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone)]
pub struct Participant {
    pub id: ParticipantId,
    pub name: String,
}

impl Participant {
    pub fn new(id: ParticipantId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
        }
    }
}
