//! Recording Session identity.
//!
//! RecordingSessionId represents the technical identity
//! of a recorder-side recording session.
//!
//! It is intentionally independent from:
//! - persistence identifiers
//! - filenames
//! - storage locations
//! - synchronization mechanisms
//!
//! See:
//! - ADR-040 Recorder Workflow and Capture Lifecycle Coordination

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingSessionId(String);

impl RecordingSessionId {
    /// Creates a new recording session identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}
