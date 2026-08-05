//! Recorder application flow.
//!
//! This module composes recorder workflow and artifact processing.
//!
//! It intentionally does not contain:
//! - audio implementation logic
//! - artifact creation logic
//! - persistence implementation
//!
//! See:
//! - ADR-040 Recorder Workflow and Capture Lifecycle Coordination
//! - ADR-051 Recording Artifact Processing Boundary

use crate::artifact::processing::RecordingArtifactProcessor;
use crate::audio::CaptureProvider;
use crate::persistence::PersistenceProvider;
use crate::session::RecordingSession;
use crate::workflow::RecorderWorkflow;

pub struct RecorderApplication<C, P>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    workflow: RecorderWorkflow<C>,
    processor: RecordingArtifactProcessor<P>,
}

impl<C, P> RecorderApplication<C, P>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    pub fn new(
        session: RecordingSession,
        capture: C,
        processor: RecordingArtifactProcessor<P>,
    ) -> Self {
        Self {
            workflow: RecorderWorkflow::new(session, capture),
            processor,
        }
    }

    pub fn start(&mut self) {
        self.workflow.start();
    }

    pub fn stop(&mut self, recording_session_id: impl Into<String>) {
        let capture_result = self.workflow.stop();

        self.processor.process(capture_result, recording_session_id);
    }

    pub fn session(&self) -> &RecordingSession {
        self.workflow.session()
    }
}
