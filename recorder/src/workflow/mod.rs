//! Local recorder workflow.
//!
//! Distributed recording lifecycle coordination belongs to the Core/application
//! boundary. This module owns only the technical lifecycle of one recorder:
//! capture startup, local READY, Opening emission/confirmation and capture stop.

use crate::audio::{
    CaptureProvider, CaptureResult, RecordingConfiguration, SyncSignet, SyncSignetEmissionError,
};
use crate::session::{RecordingSession, SessionStatus};

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
    SignetEmission(SyncSignetEmissionError),
}

impl<C> RecorderWorkflow<C>
where
    C: CaptureProvider,
{
    pub fn new(session: RecordingSession, capture: C) -> Self {
        Self { session, capture }
    }

    /// Starts local technical capture. It does not enter stable Recording.
    pub fn start(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), crate::audio::CaptureStartError> {
        if self.session.begin().is_err() {
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

    /// Records local READY. The distributed READY barrier is not owned here.
    pub fn ready(&mut self) -> Result<(), crate::session::SessionTransitionError> {
        self.session.ready()
    }

    /// Emits a sync signet into the local capture.
    ///
    /// Opening is strict: it is legal only in the local Opening phase and a
    /// successful emission confirms the local Opening barrier. Closing is
    /// optional and does not alter the lifecycle.
    pub fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
        if signet.kind() == crate::audio::SyncSignetKind::Opening
            && self.session.status() != &SessionStatus::Opening
        {
            return Err(SyncSignetEmissionError::NotCapturing);
        }

        self.capture.emit_sync_signet(signet)?;

        if signet.kind() == crate::audio::SyncSignetKind::Opening {
            self.session
                .confirm_opening()
                .map_err(|_| SyncSignetEmissionError::NotCapturing)?;
        }

        Ok(())
    }

    /// Stops local technical capture. Closing, if desired, must be attempted
    /// by the application while capture is still active; its failure must not
    /// prevent this technical stop.
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

    /// Best-effort Closing helper. The emit attempt happens before technical
    /// stop and is never replayed after stop, regardless of its outcome.
    pub fn stop_after_optional_closing(&mut self, closing: &SyncSignet) -> CaptureResult {
        if self.session.status() != &SessionStatus::Recording {
            return CaptureResult::failed(
                self.session.id(),
                "invalid recording lifecycle transition",
            );
        }

        let _ = self.capture.emit_sync_signet(closing);
        self.stop()
    }

    pub fn session(&self) -> &RecordingSession {
        &self.session
    }

    pub fn is_recording(&self) -> bool {
        matches!(self.session.status(), SessionStatus::Recording)
    }
}
