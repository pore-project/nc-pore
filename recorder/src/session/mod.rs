//! Recorder session model.
//!
//! This module contains the application-level recording session state.
//!
//! It intentionally does not contain:
//! - production domain logic
//! - audio backend access
//! - file handling
//! - synchronization logic
//!
//! The recorder layer represents the local recording workflow.
//!
//! See:
//! - ADR-038 Core Implementation Structure and Module Organization
//! - ADR-039 Recording Architecture and Capture Boundary (future)

#[derive(Debug, PartialEq, Eq)]
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
    status: SessionStatus,
}

impl RecordingSession {
    /// Creates a new recording session.
    ///
    /// A new recorder session always starts in Created state.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: SessionStatus::Created,
        }
    }

    /// Returns the current session status.
    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    /// Starts the recording process.
    pub fn start(&mut self) {
        self.status = SessionStatus::Recording;
    }

    /// Stops the recording process.
    pub fn stop(&mut self) {
        self.status = SessionStatus::Stopped;
    }

    /// Marks the recording session as stored.
    pub fn store(&mut self) {
        self.status = SessionStatus::Stored;
    }

    /// Marks the recording session as failed.
    pub fn fail(&mut self) {
        self.status = SessionStatus::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01
    // Verify: A new recording session starts in Created state.
    #[test]
    fn new_session_starts_as_created() {
        let session = RecordingSession::new("test-session");

        assert_eq!(session.status(), &SessionStatus::Created);
    }

    // TEST-02
    // Verify: Starting a recording session changes the state to Recording.
    #[test]
    fn starting_session_changes_status_to_recording() {
        let mut session = RecordingSession::new("test-session");

        session.start();

        assert_eq!(session.status(), &SessionStatus::Recording);
    }

    // TEST-03
    // Verify: A recording session can be stopped.
    //
    // Lifecycle:
    // Created -> Recording -> Stopped
    #[test]
    fn stopping_session_changes_status_to_stopped() {
        let mut session = RecordingSession::new("test-session");

        session.start();
        session.stop();

        assert_eq!(session.status(), &SessionStatus::Stopped);
    }

    // TEST-04
    // Verify: A stopped recording session can be stored.
    //
    // Lifecycle:
    // Created -> Recording -> Stopped -> Stored
    #[test]
    fn storing_session_changes_status_to_stored() {
        let mut session = RecordingSession::new("test-session");

        session.start();
        session.stop();
        session.store();

        assert_eq!(session.status(), &SessionStatus::Stored);
    }

    // TEST-05
    // Verify: A recording session can enter failed state.
    #[test]
    fn failing_session_changes_status_to_failed() {
        let mut session = RecordingSession::new("test-session");

        session.fail();

        assert_eq!(session.status(), &SessionStatus::Failed);
    }
}
