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

    pub fn stop(&mut self) -> crate::artifact::RecordingArtifact {
        let recording_session_id = self.workflow.session().id().to_string();

        let capture_result = self.workflow.stop();

        self.processor.process(capture_result, recording_session_id)
    }

    pub fn session(&self) -> &RecordingSession {
        self.workflow.session()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::coordination::ArtifactCoordinator;
    use crate::audio::{CaptureProvider, CaptureResult};
    use crate::persistence::InMemoryPersistenceProvider;

    struct TestCaptureProvider;

    impl CaptureProvider for TestCaptureProvider {
        fn start_capture(&mut self) {}

        fn stop_capture(&mut self) -> CaptureResult {
            CaptureResult::new("application-test-capture")
        }
    }

    // TEST-24
    //
    // Verify: Application flow uses the recording session
    // as source for artifact session association.
    #[test]
    fn application_processes_recording_flow() {
        let session = RecordingSession::new("session-001");

        let capture = TestCaptureProvider;

        let persistence = InMemoryPersistenceProvider::new();

        let coordinator = ArtifactCoordinator::new(persistence);

        let processor = RecordingArtifactProcessor::new(coordinator);

        let mut application = RecorderApplication::new(session, capture, processor);

        application.start();

        let artifact = application.stop();

        assert_eq!(artifact.id, "application-test-capture");
        assert_eq!(artifact.recording_session_id, "session-001");
    }

    // TEST-25
    //
    // Verify: Complete application flow creates and stores
    // a recording artifact.
    #[test]
    fn application_stores_processed_artifact() {
        let session = RecordingSession::new("session-002");

        let capture = TestCaptureProvider;

        let persistence = InMemoryPersistenceProvider::new();

        let coordinator = ArtifactCoordinator::new(persistence);

        let processor = RecordingArtifactProcessor::new(coordinator);

        let mut application = RecorderApplication::new(session, capture, processor);

        application.start();

        let artifact = application.stop();

        assert_eq!(artifact.id, "application-test-capture");
        assert_eq!(artifact.recording_session_id, "session-002");
    }
}
