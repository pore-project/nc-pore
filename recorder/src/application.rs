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
use crate::audio::{CaptureProvider, CaptureStatus, RecordingConfiguration};
use crate::persistence::PersistenceProvider;
use crate::persistence::PersistenceStoreError;
use crate::session::{RecordingSession, RecordingSessionId};
use crate::workflow::RecorderWorkflow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecorderApplicationError {
    Capture(String),
    Persistence(PersistenceStoreError),
}

impl From<PersistenceStoreError> for RecorderApplicationError {
    fn from(error: PersistenceStoreError) -> Self {
        Self::Persistence(error)
    }
}

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

    pub fn start(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), crate::audio::CaptureStartError> {
        self.workflow.start(configuration)
    }

    /// Stops the local recording and persists an artifact associated with
    /// the originating domain production and recording.
    pub fn stop(
        &mut self,
        association: RecordingArtifactAssociation,
    ) -> Result<crate::artifact::RecordingArtifact, RecorderApplicationError> {
        let recording_session_id = RecordingSessionId::new(self.workflow.session().id());

        let capture_result = self.workflow.stop();

        if let CaptureStatus::Failed(error) = capture_result.status() {
            return Err(RecorderApplicationError::Capture(error.clone()));
        }

        self.processor
            .process(capture_result, recording_session_id, association)
            .map_err(RecorderApplicationError::from)
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
    use crate::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

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

    struct FailedCaptureProvider;

    impl CaptureProvider for FailedCaptureProvider {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), crate::audio::CaptureStartError> {
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            CaptureResult::failed("application-failed-capture", "input stream failed")
        }
    }

    struct RejectingPersistenceProvider;

    impl crate::persistence::PersistenceProvider for RejectingPersistenceProvider {
        fn store(&mut self, _artifact: crate::artifact::RecordingArtifact) {
            panic!("failed capture must not reach persistence");
        }

        fn load(&self, _id: &str) -> PersistenceLoadResult {
            PersistenceLoadResult::NotFound
        }

        fn list_ids(&self) -> Vec<String> {
            Vec::new()
        }

        fn list(&self) -> Vec<crate::artifact::RecordingArtifact> {
            Vec::new()
        }

        fn remove(&mut self, _id: &str) {}
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

        application.start(&configuration).unwrap();

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

        application.start(&configuration).unwrap();

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

    #[test]
    fn failed_capture_returns_application_error() {
        let session = RecordingSession::new("session-failed");
        let processor =
            RecordingArtifactProcessor::new(ArtifactCoordinator::new(RejectingPersistenceProvider));
        let mut application = RecorderApplication::new(session, FailedCaptureProvider, processor);

        application
            .start(&RecordingConfiguration::default())
            .unwrap();

        let result = application.stop(RecordingArtifactAssociation::new(
            "production-failed",
            "recording-failed",
        ));

        assert!(matches!(
            result,
            Err(RecorderApplicationError::Capture(error)) if error == "input stream failed"
        ));
    }
}
