//! Recorder workflow coordination.
//!
//! This module coordinates the local recording workflow.
//!
//! It connects:
//! - RecordingSession
//! - CaptureProvider
//! It also provides the participant READY barrier required by ADR-068.
//!
//! It intentionally does not contain:
//! - domain production rules
//! - audio backend implementations
//! - persistent storage logic
//!
//! See:
//! - ADR-040 Recorder Workflow and Capture Lifecycle Coordination
//! - ADR-061 Configurable Recording Configuration
//! - ADR-068 Recording Start and Audio Synchronization Signet

pub mod recording_start;

use crate::audio::{CaptureProvider, CaptureResult, RecordingConfiguration};
use crate::session::{RecordingSession, SessionStatus};

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

    /// Starts one concrete recording attempt.
    ///
    /// The lifecycle is deliberately split into explicit local states:
    ///
    /// Prepared -> Starting -> WaitingForReady
    ///
    /// `READY` is intentionally not generated here. The higher-level session
    /// coordinator must confirm that this recording participant is ready
    /// before the Opening Sync Signet may be emitted.
    pub fn start(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), crate::audio::CaptureStartError> {
        if let Err(_error) = self.session.begin() {
            self.session.fail().ok();
            return Err(crate::audio::CaptureStartError::DeviceUnavailable);
        }

        match self.capture.start_capture(configuration) {
            Ok(()) => {
                self.session.capture_started().ok();
                Ok(())
            }
            Err(error) => {
                self.session.fail().ok();
                Err(error)
            }
        }
    }

    /// Confirms that this local recording participant has reported READY.
    pub fn ready(&mut self) -> Result<(), crate::session::SessionTransitionError> {
        self.session.ready()
    }

    /// Stops the recording workflow.
    ///
    /// The workflow coordinates the ADR-068 stop order:
    ///
    /// 1. enter Stopping
    /// 2. the higher-level coordinator emits the Closing Sync Signet
    /// 3. technical capture is stopped
    /// 4. the local session is completed only after capture has actually ended
    ///
    /// The completed CaptureResult is returned to the caller, allowing
    /// downstream processing to remain outside the workflow.
    pub fn stop(&mut self) -> CaptureResult {
        if self.session.begin_stop().is_err() {
            return CaptureResult::failed(
                self.session.id(),
                "invalid recording lifecycle transition",
            );
        }

        let capture_result = self.capture.stop_capture();

        if matches!(
            capture_result.status(),
            crate::audio::CaptureStatus::Failed(_)
        ) {
            self.session.fail().ok();
        } else {
            self.session.complete().ok();
        }

        capture_result
    }

    /// Provides read-only access to the recorder session.
    pub fn session(&self) -> &RecordingSession {
        self.session.as_ref()
    }

    /// Returns whether the local recorder is actively recording.
    pub fn is_recording(&self) -> bool {
        matches!(self.session.status(), SessionStatus::Recording)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCapture {
        active: bool,
        fail_on_start: bool,
        fail_on_stop: bool,
    }

    impl TestCapture {
        fn new() -> Self {
            Self {
                active: false,
                fail_on_start: false,
                fail_on_stop: false,
            }
        }

        fn failing_start() -> Self {
            Self {
                active: false,
                fail_on_start: true,
                fail_on_stop: false,
            }
        }

        fn failing_stop() -> Self {
            Self {
                active: false,
                fail_on_start: false,
                fail_on_stop: true,
            }
        }
    }

    impl CaptureProvider for TestCapture {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), crate::audio::CaptureStartError> {
            if self.fail_on_start {
                return Err(crate::audio::CaptureStartError::DeviceUnavailable);
            }

            self.active = true;
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            self.active = false;

            if self.fail_on_stop {
                CaptureResult::failed("workflow-test-capture", "stop failed")
            } else {
                CaptureResult::new("workflow-test-capture")
            }
        }
    }

    // TEST-01 / CUE30
    // Verify: A workflow can be created with a recording session
    // and a capture provider.
    #[test]
    fn workflow_can_be_created_with_session_and_capture() {
        let session = RecordingSession::new("workflow-test");
        let capture = TestCapture::new();
        let workflow = RecorderWorkflow::new(session, capture);

        assert_eq!(workflow.session().status(), &SessionStatus::Prepared);
    }

    // TEST-02 / CUE30
    // Verify: Workflow start leaves the local recorder at WaitingForReady
    // until the higher-level coordinator confirms READY.
    #[test]
    fn workflow_waits_for_ready_after_capture_start() {
        let session = RecordingSession::new("workflow-test");
        let capture = TestCapture::new();
        let mut workflow = RecorderWorkflow::new(session, capture);
        let configuration = RecordingConfiguration::default();

        workflow.start(&configuration).unwrap();

        assert_eq!(workflow.session().status(), &SessionStatus::WaitingForReady);
        assert!(!workflow.is_recording());

        workflow.ready().unwrap();

        assert_eq!(workflow.session().status(), &SessionStatus::Recording);
        assert!(workflow.is_recording());

        let result = workflow.stop();

        assert_eq!(result.id(), "workflow-test-capture");
        assert_eq!(workflow.session().status(), &SessionStatus::Completed);
    }

    // TEST-03 / CUE30
    // Verify: A failed capture start does not enter Recording state.
    #[test]
    fn failed_capture_start_marks_session_as_failed() {
        let session = RecordingSession::new("workflow-test");
        let capture = TestCapture::failing_start();
        let mut workflow = RecorderWorkflow::new(session, capture);
        let configuration = RecordingConfiguration::default();

        let result = workflow.start(&configuration);

        assert_eq!(
            result,
            Err(crate::audio::CaptureStartError::DeviceUnavailable)
        );
        assert_eq!(workflow.session().status(), &SessionStatus::Failed);
    }

    // TEST-04 / CUE30
    // Verify: A technical failure while stopping does not falsely report
    // a successfully completed local recording.
    #[test]
    fn failed_capture_stop_marks_session_as_failed() {
        let session = RecordingSession::new("workflow-test");
        let capture = TestCapture::failing_stop();
        let mut workflow = RecorderWorkflow::new(session, capture);
        let configuration = RecordingConfiguration::default();

        workflow.start(&configuration).unwrap();
        workflow.ready().unwrap();
        let result = workflow.stop();

        assert!(matches!(
            result.status(),
            crate::audio::CaptureStatus::Failed(_)
        ));
        assert_eq!(workflow.session().status(), &SessionStatus::Failed);
    }
}
