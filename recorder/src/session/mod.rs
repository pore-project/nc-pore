#![allow(dead_code)]

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
//! - ADR-068 Recording Start and Audio Synchronization Signet

pub mod id;

pub use id::RecordingSessionId;

/// Local lifecycle of one concrete recording attempt.
///
/// The state machine deliberately models the recorder/client lifecycle,
/// not the Production Session domain lifecycle. In particular, a session
/// membership does not imply that this recorder is participating in a
/// recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionStatus {
    /// No concrete recording has been started for this local session.
    Prepared,
    /// The host has started this recording and local capture is being started.
    Starting,
    /// Local capture is active and the client is waiting for the coordinated
    /// opening of the recording.
    WaitingForReady,
    /// The local recorder is active and has reported READY for this recording.
    Recording,
    /// The coordinated stop has begun. The closing sync signet is emitted
    /// before the technical recorder is finally stopped.
    Stopping,
    /// Local capture has actually stopped and the recording is complete.
    Completed,
    /// A local technical failure prevents the recording from continuing.
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionTransitionError {
    InvalidTransition {
        from: SessionStatus,
        action: &'static str,
    },
}

#[derive(Debug)]
pub struct RecordingSession {
    id: RecordingSessionId,
    status: SessionStatus,
}

impl RecordingSession {
    /// Creates a new local recording session.
    ///
    /// A new recorder session starts in Prepared state. Merely joining a
    /// production session must never start local audio capture.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingSessionId::new(id),
            status: SessionStatus::Prepared,
        }
    }

    /// Returns the current session status.
    pub fn status(&self) -> &SessionStatus {
        &self.status
    }

    /// Returns the session identifier.
    pub fn id(&self) -> &str {
        self.id.value()
    }

    /// Begins a concrete recording attempt.
    pub fn begin(&mut self) -> Result<(), SessionTransitionError> {
        self.transition(SessionStatus::Starting, "begin")
    }

    /// Records that local audio capture is actually running.
    pub fn capture_started(&mut self) -> Result<(), SessionTransitionError> {
        self.transition(SessionStatus::WaitingForReady, "capture_started")
    }

    /// Records the client's READY confirmation for this recording attempt.
    ///
    /// The coordination layer is responsible for ensuring that this READY
    /// belongs to the current recording participant set and recording start.
    pub fn ready(&mut self) -> Result<(), SessionTransitionError> {
        self.transition(SessionStatus::Recording, "ready")
    }

    /// Begins the coordinated stop sequence.
    ///
    /// The closing sync signet must be emitted before the technical capture
    /// is stopped. This state represents that interval.
    pub fn begin_stop(&mut self) -> Result<(), SessionTransitionError> {
        self.transition(SessionStatus::Stopping, "begin_stop")
    }

    /// Confirms that local technical capture has actually ended.
    pub fn complete(&mut self) -> Result<(), SessionTransitionError> {
        self.transition(SessionStatus::Completed, "complete")
    }

    /// Marks the local recording attempt as failed.
    pub fn fail(&mut self) -> Result<(), SessionTransitionError> {
        if self.status == SessionStatus::Completed {
            return Err(SessionTransitionError::InvalidTransition {
                from: self.status,
                action: "fail",
            });
        }

        self.status = SessionStatus::Failed;
        Ok(())
    }

    fn transition(
        &mut self,
        next: SessionStatus,
        action: &'static str,
    ) -> Result<(), SessionTransitionError> {
        let valid = matches!(
            (self.status, next),
            (SessionStatus::Prepared, SessionStatus::Starting)
                | (SessionStatus::Starting, SessionStatus::WaitingForReady)
                | (SessionStatus::WaitingForReady, SessionStatus::Recording)
                | (SessionStatus::Recording, SessionStatus::Stopping)
                | (SessionStatus::Stopping, SessionStatus::Completed)
        );

        if !valid {
            return Err(SessionTransitionError::InvalidTransition {
                from: self.status,
                action,
            });
        }

        self.status = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01 / CUE30
    // Verify the complete happy-path lifecycle defined by ADR-068:
    // Prepared -> Starting -> WaitingForReady -> Recording -> Stopping -> Completed.
    #[test]
    fn recording_lifecycle_follows_adr_068() {
        let mut session = RecordingSession::new("test-session");

        session.begin().unwrap();
        session.capture_started().unwrap();
        session.ready().unwrap();
        session.begin_stop().unwrap();
        session.complete().unwrap();

        assert_eq!(session.status(), &SessionStatus::Completed);
    }

    // TEST-02 / CUE30
    // Verify that joining/preparing a session does not itself start recording.
    #[test]
    fn new_session_is_only_prepared() {
        let session = RecordingSession::new("test-session");

        assert_eq!(session.status(), &SessionStatus::Prepared);
    }

    // TEST-03 / CUE30
    // Verify that an invalid lifecycle transition is rejected.
    #[test]
    fn invalid_transition_is_rejected() {
        let mut session = RecordingSession::new("test-session");

        let result = session.ready();

        assert_eq!(
            result,
            Err(SessionTransitionError::InvalidTransition {
                from: SessionStatus::Prepared,
                action: "ready",
            })
        );
    }

    // TEST-04 / CUE30
    // Verify that failure is available during an active lifecycle but cannot
    // resurrect or overwrite a technically completed recording.
    #[test]
    fn failure_is_terminal_after_completion() {
        let mut session = RecordingSession::new("test-session");

        session.begin().unwrap();
        session.capture_started().unwrap();
        session.ready().unwrap();
        session.begin_stop().unwrap();
        session.complete().unwrap();

        let result = session.fail();

        assert_eq!(
            result,
            Err(SessionTransitionError::InvalidTransition {
                from: SessionStatus::Completed,
                action: "fail",
            })
        );
        assert_eq!(session.status(), &SessionStatus::Completed);
    }

    // TEST-05 / CUE30
    // Verify that a technical failure can be surfaced without inventing a
    // domain state in Core.
    #[test]
    fn technical_failure_moves_session_to_failed() {
        let mut session = RecordingSession::new("test-session");

        session.begin().unwrap();
        session.fail().unwrap();

        assert_eq!(session.status(), &SessionStatus::Failed);
    }
}
