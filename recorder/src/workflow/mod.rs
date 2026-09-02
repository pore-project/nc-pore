//! Recorder workflow coordination.
//!
//! This module coordinates the local recording workflow.
//! It connects RecordingSession, CaptureProvider and the ADR-068 start/stop
//! coordinators so that the local execution order cannot bypass the sync
//! signet boundaries.

pub mod recording_start;
pub mod recording_stop;

use crate::audio::{
    CaptureProvider, CaptureResult, RecordingConfiguration, SyncSignet, SyncSignetEmissionError,
};
use crate::session::{RecordingSession, SessionStatus};
use recording_start::{RecordingParticipantId, RecordingStartCoordinator, RecordingStartError};
use recording_stop::RecordingStopCoordinator;

pub struct RecorderWorkflow<C>
where
    C: CaptureProvider,
{
    session: RecordingSession,
    capture: C,
}

#[derive(Debug, PartialEq, Eq)]
pub enum WorkflowCoordinationError {
    InvalidSessionState,
    RecordingStart(RecordingStartError),
    SignetEmission(SyncSignetEmissionError),
}

impl<C> RecorderWorkflow<C>
where
    C: CaptureProvider,
{
    pub fn new(session: RecordingSession, capture: C) -> Self {
        Self { session, capture }
    }

    /// Starts local technical capture. The local session remains
    /// WaitingForReady until the READY barrier is completed.
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

    /// Confirms local READY and, when this is the final participant, emits the
    /// required Opening Signet into the already-running capture. The local
    /// technical session enters Recording only after that emission succeeds.
    pub fn ready_and_maybe_opening_signet(
        &mut self,
        coordinator: &mut RecordingStartCoordinator,
        participant: &RecordingParticipantId,
    ) -> Result<Option<SyncSignet>, WorkflowCoordinationError> {
        if self.session.status() != &SessionStatus::WaitingForReady {
            return Err(WorkflowCoordinationError::InvalidSessionState);
        }

        coordinator
            .confirm_ready(participant)
            .map_err(WorkflowCoordinationError::RecordingStart)?;

        if !coordinator.all_ready() {
            return Ok(None);
        }

        let signet = coordinator
            .opening_sync_signet()
            .ok_or(WorkflowCoordinationError::InvalidSessionState)?;

        self.capture
            .emit_sync_signet(&signet)
            .map_err(WorkflowCoordinationError::SignetEmission)?;

        self.session
            .ready()
            .map_err(|_| WorkflowCoordinationError::InvalidSessionState)?;
        Ok(Some(signet))
    }

    /// Legacy/local-only READY transition. The coordinated ADR-068 path should
    /// use `ready_and_maybe_opening_signet`.
    pub fn ready(&mut self) -> Result<(), crate::session::SessionTransitionError> {
        self.session.ready()
    }

    pub fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
        self.capture.emit_sync_signet(signet)
    }

    /// Stops local capture without coordinating a Closing Signet.
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

    /// Attempts the optional Closing Signet while capture is active and then
    /// always proceeds to technical stop. Failure to emit Closing is a valid
    /// v1 outcome: Bob may have disconnected, stopped capturing already, or
    /// simply may not support the configured signet. None of those cases may
    /// block completion or cause a replay after capture has ended.
    pub fn stop_with_coordinator(
        &mut self,
        coordinator: &mut RecordingStopCoordinator,
    ) -> Result<(Option<SyncSignet>, CaptureResult), WorkflowCoordinationError> {
        if self.session.status() != &SessionStatus::Recording {
            return Err(WorkflowCoordinationError::InvalidSessionState);
        }

        let signet = coordinator.closing_sync_signet();

        if let Some(ref signet) = signet {
            // Best effort only. We deliberately discard emission failure and
            // continue with technical stop.
            let _ = self.capture.emit_sync_signet(signet);
        }

        Ok((signet, self.stop()))
    }

    /// Lower-level helper for callers that explicitly own the optional Closing
    /// Signet decision. It never delays technical stop.
    pub fn stop_after_closing_signet<F>(
        &mut self,
        closing_signet: SyncSignet,
        emit: F,
    ) -> CaptureResult
    where
        F: FnOnce(SyncSignet),
    {
        emit(closing_signet);
        self.stop()
    }

    pub fn session(&self) -> &RecordingSession {
        &self.session
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.session.status(), SessionStatus::Recording)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestCapture {
        active: bool,
        fail_on_start: bool,
        fail_on_stop: bool,
        fail_on_opening_signet: bool,
        fail_on_closing_signet: bool,
        events: Rc<RefCell<Vec<&'static str>>>,
    }

    impl TestCapture {
        fn new() -> Self {
            Self {
                active: false,
                fail_on_start: false,
                fail_on_stop: false,
                fail_on_opening_signet: false,
                fail_on_closing_signet: false,
                events: Rc::new(RefCell::new(Vec::new())),
            }
        }

        fn failing_start() -> Self {
            Self {
                fail_on_start: true,
                ..Self::new()
            }
        }

        fn failing_stop() -> Self {
            Self {
                fail_on_stop: true,
                ..Self::new()
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

        fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
            match signet.kind() {
                crate::audio::SyncSignetKind::Opening if self.fail_on_opening_signet => {
                    Err(SyncSignetEmissionError::Unsupported)
                }
                crate::audio::SyncSignetKind::Closing if self.fail_on_closing_signet => {
                    Err(SyncSignetEmissionError::Unsupported)
                }
                _ => {
                    self.events.borrow_mut().push(match signet.kind() {
                        crate::audio::SyncSignetKind::Opening => "opening",
                        crate::audio::SyncSignetKind::Closing => "closing",
                    });
                    Ok(())
                }
            }
        }

        fn stop_capture(&mut self) -> CaptureResult {
            self.events.borrow_mut().push("stop");
            self.active = false;
            if self.fail_on_stop {
                CaptureResult::failed("workflow-test-capture", "stop failed")
            } else {
                CaptureResult::new("workflow-test-capture")
            }
        }
    }

    fn participant(id: &str) -> RecordingParticipantId {
        RecordingParticipantId::new(id)
    }

    #[test]
    fn workflow_can_be_created_with_session_and_capture() {
        let workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::new(),
        );
        assert_eq!(workflow.session().status(), &SessionStatus::Prepared);
    }

    #[test]
    fn coordinated_ready_emits_opening_only_at_barrier() {
        let first = participant("p1");
        let second = participant("p2");
        let mut coordinator = RecordingStartCoordinator::new([first.clone(), second.clone()]);
        let capture = TestCapture::new();
        let events = Rc::clone(&capture.events);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            capture,
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();

        assert_eq!(
            workflow.ready_and_maybe_opening_signet(&mut coordinator, &first),
            Ok(None)
        );
        assert_eq!(workflow.session().status(), &SessionStatus::WaitingForReady);
        assert_eq!(
            workflow.ready_and_maybe_opening_signet(&mut coordinator, &second),
            Ok(Some(SyncSignet::opening()))
        );
        assert_eq!(workflow.session().status(), &SessionStatus::Recording);
        assert_eq!(&*events.borrow(), &["opening"]);
    }

    #[test]
    fn non_recording_participant_cannot_complete_local_ready() {
        let recording = participant("p1");
        let outsider = participant("p2");
        let mut coordinator = RecordingStartCoordinator::new([recording]);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::new(),
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();

        assert_eq!(
            workflow.ready_and_maybe_opening_signet(&mut coordinator, &outsider),
            Err(WorkflowCoordinationError::RecordingStart(
                RecordingStartError::NotRecordingParticipant
            ))
        );
        assert_eq!(workflow.session().status(), &SessionStatus::WaitingForReady);
    }

    #[test]
    fn opening_signet_emission_failure_does_not_enter_recording_state() {
        let p = participant("p1");
        let mut coordinator = RecordingStartCoordinator::new([p.clone()]);
        let mut capture = TestCapture::new();
        capture.fail_on_opening_signet = true;
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            capture,
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();

        assert!(matches!(
            workflow.ready_and_maybe_opening_signet(&mut coordinator, &p),
            Err(WorkflowCoordinationError::SignetEmission(_))
        ));
        assert_eq!(workflow.session().status(), &SessionStatus::WaitingForReady);
    }

    #[test]
    fn coordinated_stop_emits_closing_before_technical_stop_when_supported() {
        let p = participant("p1");
        let mut start = RecordingStartCoordinator::new([p.clone()]);
        let mut stop = RecordingStopCoordinator::new([p.clone()]);
        let capture = TestCapture::new();
        let events = Rc::clone(&capture.events);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            capture,
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();
        workflow
            .ready_and_maybe_opening_signet(&mut start, &p)
            .unwrap();
        let (signet, result) = workflow.stop_with_coordinator(&mut stop).unwrap();

        assert_eq!(signet, Some(SyncSignet::closing()));
        assert_eq!(result.id(), "workflow-test-capture");
        assert_eq!(&*events.borrow(), &["opening", "closing", "stop"]);
        assert_eq!(workflow.session().status(), &SessionStatus::Completed);
    }

    #[test]
    fn coordinated_stop_continues_when_closing_signet_is_unavailable() {
        let p = participant("p1");
        let mut start = RecordingStartCoordinator::new([p.clone()]);
        let mut stop = RecordingStopCoordinator::new([p.clone()]);
        let mut capture = TestCapture::new();
        capture.fail_on_closing_signet = true;
        let events = Rc::clone(&capture.events);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            capture,
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();
        workflow
            .ready_and_maybe_opening_signet(&mut start, &p)
            .unwrap();

        let (signet, result) = workflow.stop_with_coordinator(&mut stop).unwrap();

        assert_eq!(signet, Some(SyncSignet::closing()));
        assert!(matches!(
            result.status(),
            crate::audio::CaptureStatus::Completed
        ));
        assert_eq!(&*events.borrow(), &["opening", "stop"]);
        assert_eq!(workflow.session().status(), &SessionStatus::Completed);
    }

    #[test]
    fn coordinated_stop_can_complete_without_closing_signet() {
        let p = participant("p1");
        let mut start = RecordingStartCoordinator::new([p.clone()]);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::new(),
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();
        workflow
            .ready_and_maybe_opening_signet(&mut start, &p)
            .unwrap();

        let result = workflow.stop().status();
        assert!(matches!(result, crate::audio::CaptureStatus::Completed));
        assert_eq!(workflow.session().status(), &SessionStatus::Completed);
    }

    #[test]
    fn stop_rejects_non_recording_state_before_consuming_closing_signet() {
        let p = participant("p1");
        let mut stop = RecordingStopCoordinator::new([p]);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::new(),
        );
        assert_eq!(
            workflow.stop_with_coordinator(&mut stop),
            Err(WorkflowCoordinationError::InvalidSessionState)
        );
        assert!(stop.closing_sync_signet().is_some());
    }

    #[test]
    fn failed_capture_start_marks_session_as_failed() {
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::failing_start(),
        );
        let result = workflow.start(&RecordingConfiguration::default());
        assert_eq!(
            result,
            Err(crate::audio::CaptureStartError::DeviceUnavailable)
        );
        assert_eq!(workflow.session().status(), &SessionStatus::Failed);
    }

    #[test]
    fn failed_capture_stop_marks_session_as_failed() {
        let p = participant("p1");
        let mut start = RecordingStartCoordinator::new([p.clone()]);
        let mut stop = RecordingStopCoordinator::new([p.clone()]);
        let mut workflow = RecorderWorkflow::new(
            RecordingSession::new("workflow-test"),
            TestCapture::failing_stop(),
        );
        workflow.start(&RecordingConfiguration::default()).unwrap();
        workflow
            .ready_and_maybe_opening_signet(&mut start, &p)
            .unwrap();
        let result = workflow.stop_with_coordinator(&mut stop).unwrap().1;
        assert!(matches!(
            result.status(),
            crate::audio::CaptureStatus::Failed(_)
        ));
        assert_eq!(workflow.session().status(), &SessionStatus::Failed);
    }
}
