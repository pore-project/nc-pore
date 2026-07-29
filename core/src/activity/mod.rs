#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActivityType {
    SessionCreated,
    SessionStarted,
    SessionCompleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivityEvent {
    pub activity_type: ActivityType,
}

impl ActivityEvent {
    pub fn new(activity_type: ActivityType) -> Self {
        Self { activity_type }
    }
}
