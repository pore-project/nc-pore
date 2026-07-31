//! Recorder workflow coordination.
//!
//! This module coordinates the local recording workflow.
//!
//! It connects:
//! - RecordingSession
//! - CaptureProvider
//!
//! It intentionally does not contain:
//! - domain production rules
//! - audio backend implementations
//! - persistent storage logic
//!
//! See:
//! - ADR-040 Recorder Workflow and Capture Lifecycle Coordination

use crate::audio::CaptureProvider;
use crate::session::RecordingSession;

/// Coordinates the local recorder workflow.
///
/// The workflow layer connects session state management
/// with technical capture capabilities.
///
/// The workflow does not own:
/// - audio implementation details
/// - production domain rules
/// - storage decisions
pub struct RecorderWorkflow<C>
where
    C: CaptureProvider,
{
    session: RecordingSession,
    capture: C,
}

impl<C> RecorderWorkflow<C>
where
    C: CaptureProvider,
{
    /// Creates a new recorder workflow.
    pub fn new(session: RecordingSession, capture: C) -> Self {
        Self { session, capture }
    }

    /// Starts the recording workflow.
    ///
    /// The workflow coordinates:
    /// - recorder session state transition
    /// - capture provider activation
    pub fn start(&mut self) {
        self.session.start();
        self.capture.start_capture();
    }

    /// Stops the recording workflow.
    ///
    /// The workflow coordinates:
    /// - capture provider shutdown
    /// - recorder session state transition
    pub fn stop(&mut self) {
        self.capture.stop_capture();
        self.session.stop();
    }

    /// Provides read-only access to the recorder session.
    pub fn session(&self) -> &RecordingSession {
        &self.session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCapture {
        active: bool,
    }

    impl TestCapture {
        fn new() -> Self {
            Self { active: false }
        }
    }

    impl CaptureProvider for TestCapture {
        fn start_capture(&mut self) {
            self.active = true;
        }

        fn stop_capture(&mut self) {
            self.active = false;
        }
    }

    // TEST-01
    // Verify: A workflow can be created with a recording session
    // and a capture provider.
    //
    // Protects ADR-040:
    // The workflow layer coordinates recorder session state
    // and capture implementation without coupling them.
    #[test]
    fn workflow_can_be_created_with_session_and_capture() {
        let session = RecordingSession::new("workflow-test");

        let capture = TestCapture::new();

        let workflow = RecorderWorkflow::new(session, capture);

        assert_eq!(
            workflow.session().status(),
            &crate::session::SessionStatus::Created
        );
    }

    // TEST-02
    // Verify: Workflow start and stop operations coordinate
    // session lifecycle and capture lifecycle.
    //
    // Protects ADR-040:
    // Recorder workflow remains responsible for coordination,
    // while capture implementation remains behind its boundary.
    #[test]
    fn workflow_coordinates_session_and_capture() {
        let session = RecordingSession::new("workflow-test");

        let capture = TestCapture::new();

        let mut workflow = RecorderWorkflow::new(session, capture);

        workflow.start();

        assert_eq!(
            workflow.session().status(),
            &crate::session::SessionStatus::Recording
        );

        workflow.stop();

        assert_eq!(
            workflow.session().status(),
            &crate::session::SessionStatus::Stopped
        );
    }
}
