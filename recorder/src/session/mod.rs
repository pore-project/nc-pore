#[derive(Debug)]
pub enum SessionStatus {
    Created,
    Recording,
    Stopped,
    Stored,
    Failed,
}

#[derive(Debug)]
pub struct RecordingSession {
    pub id: String,
    pub status: SessionStatus,
}
