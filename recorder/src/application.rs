//! Recorder application flow.
//!
//! This module composes recorder workflow and artifact processing.
//!
//! It intentionally does not contain:
//! - audio implementation logic
//! - artifact creation logic
//! - persistence implementation
//! - production domain rules
//!
//! The application boundary receives the originating domain identifiers as
//! opaque values and passes them into artifact processing. This keeps the
//! recorder crate independent from the core crate while preserving the
//! relationship between a domain Recording and its persisted artifact.
//!
//! See:
//! - ADR-040 Recorder Workflow and Capture Lifecycle Coordination
//! - ADR-051 Recording Artifact Processing Boundary
//! - ADR-061 Configurable Recording Configuration

use crate::artifact::RecordingArtifactAssociation;
use crate::artifact::processing::RecordingArtifactProcessor;
use crate::audio::{CaptureProvider, RecordingConfiguration};
use crate::persistence::PersistenceProvider;
use crate::persistence::PersistenceStoreError;
use crate::session::{RecordingSession, RecordingSessionId};
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

    pub fn start(&mut self, configuration: &RecordingConfiguration) {
        self.workflow.start(configuration);
    }

    /// Stops the local recording and persists an artifact associated with
    /// the originating domain production and recording.
    pub fn stop(
        &mut self,
        association: RecordingArtifactAssociation,
    ) -> Result<crate::artifact::RecordingArtifact, PersistenceStoreError> {
        let recording_session_id = RecordingSessionId::new(self.workflow.session().id());

        let capture_result = self.workflow.stop();

        self.processor
            .process(capture_result, recording_session_id, association)
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
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), crate::audio::CaptureStartError> {
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            CaptureResult::new("application-test-capture")
        }
    }

    // TEST-24
    //
    // Verify: Application flow uses the recording session
    // as source for artifact session association and preserves
    // the originating domain recording association.
    #[test]
    fn application_processes_recording_flow() {
        let session = RecordingSession::new("session-001");

        let capture = TestCaptureProvider;

        let persistence = InMemoryPersistenceProvider::new();

        let coordinator = ArtifactCoordinator::new(persistence);

        let processor = RecordingArtifactProcessor::new(coordinator);

        let mut application = RecorderApplication::new(session, capture, processor);
        let configuration = RecordingConfiguration::default();

        application.start(&configuration);

        let artifact = application
            .stop(RecordingArtifactAssociation::new(
                "production-001",
                "recording-017",
            ))
            .expect("application stop should persist artifact");

        assert_eq!(artifact.id.value(), "application-test-capture");
        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
    }

    // TEST-25
    //
    // Verify: Complete application flow creates and stores
    // a recording artifact without losing the originating domain IDs.
    #[test]
    fn application_stores_processed_artifact() {
        let session = RecordingSession::new("session-002");

        let capture = TestCaptureProvider;

        let persistence = InMemoryPersistenceProvider::new();

        let coordinator = ArtifactCoordinator::new(persistence);

        let processor = RecordingArtifactProcessor::new(coordinator);

        let mut application = RecorderApplication::new(session, capture, processor);
        let configuration = RecordingConfiguration::default();

        application.start(&configuration);

        let artifact = application
            .stop(RecordingArtifactAssociation::new(
                "production-002",
                "recording-018",
            ))
            .expect("application stop should persist artifact");

        assert_eq!(artifact.id.value(), "application-test-capture");
        assert_eq!(artifact.recording_session_id.value(), "session-002");
        assert_eq!(artifact.production_id(), Some("production-002"));
        assert_eq!(artifact.recording_id(), Some("recording-018"));
    }
}
