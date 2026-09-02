//! Recorder application flow.
//!
//! This module composes recorder workflow and artifact processing.
//!
//! It intentionally does not contain audio implementation logic, artifact
//! creation logic, persistence implementation, or production domain rules.

use crate::artifact::RecordingArtifactAssociation;
use crate::artifact::processing::RecordingArtifactProcessor;
use crate::audio::{
    CaptureProvider, CaptureStatus, RecordingConfiguration, SyncSignet, SyncSignetEmissionError,
    SyncSignetKind,
};
use crate::persistence::PersistenceProvider;
use crate::persistence::PersistenceStoreError;
use crate::session::{RecordingSession, RecordingSessionId};
use crate::workflow::RecorderWorkflow;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecorderApplicationError {
    Capture(String),
    Persistence(PersistenceStoreError),
    SyncSignet(SyncSignetEmissionError),
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

    pub fn ready(&mut self) -> Result<(), crate::session::SessionTransitionError> {
        self.workflow.ready()
    }

    /// Emits a synchronization signet into the active capture.
    ///
    /// Opening is strict because it is the required ADR-068 start barrier.
    /// Closing is optional; emission failure is deliberately swallowed so a
    /// recorder that can no longer hear/capture Closing can still stop and
    /// complete normally.
    pub fn emit_sync_signet(
        &mut self,
        signet: &SyncSignet,
    ) -> Result<(), RecorderApplicationError> {
        match signet.kind() {
            SyncSignetKind::Opening => self
                .workflow
                .emit_sync_signet(signet)
                .map_err(RecorderApplicationError::SyncSignet),
            SyncSignetKind::Closing => {
                let _ = self.workflow.emit_sync_signet(signet);
                Ok(())
            }
        }
    }

    /// Explicitly named best-effort API for optional Closing emission.
    /// Returns whether the technical provider accepted the signet.
    pub fn emit_optional_sync_signet(&mut self, signet: &SyncSignet) -> bool {
        self.workflow.emit_sync_signet(signet).is_ok()
    }

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
    use crate::audio::{CaptureProvider, CaptureResult, SyncSignetKind};
    use crate::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

    struct TestCaptureProvider {
        emitted: Vec<SyncSignetKind>,
        fail_on_closing: bool,
    }

    impl CaptureProvider for TestCaptureProvider {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), crate::audio::CaptureStartError> {
            Ok(())
        }

        fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
            if signet.kind() == SyncSignetKind::Closing && self.fail_on_closing {
                return Err(SyncSignetEmissionError::NotCapturing);
            }
            self.emitted.push(signet.kind());
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

    #[test]
    fn application_processes_recording_flow() {
        let session = RecordingSession::new("session-001");
        let capture = TestCaptureProvider {
            emitted: Vec::new(),
            fail_on_closing: false,
        };
        let processor = RecordingArtifactProcessor::new(ArtifactCoordinator::new(
            InMemoryPersistenceProvider::new(),
        ));
        let mut application = RecorderApplication::new(session, capture, processor);
        let configuration = RecordingConfiguration::default();

        application.start(&configuration).unwrap();
        application.ready().unwrap();
        application
            .emit_sync_signet(&configuration.signets().opening())
            .unwrap();

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

    #[test]
    fn application_stores_processed_artifact() {
        let session = RecordingSession::new("session-002");
        let capture = TestCaptureProvider {
            emitted: Vec::new(),
            fail_on_closing: false,
        };
        let processor = RecordingArtifactProcessor::new(ArtifactCoordinator::new(
            InMemoryPersistenceProvider::new(),
        ));
        let mut application = RecorderApplication::new(session, capture, processor);
        let configuration = RecordingConfiguration::default();
        application.start(&configuration).unwrap();
        application.ready().unwrap();

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
    fn optional_closing_failure_does_not_block_technical_stop() {
        let session = RecordingSession::new("session-closing-fallback");
        let capture = TestCaptureProvider {
            emitted: Vec::new(),
            fail_on_closing: true,
        };
        let processor = RecordingArtifactProcessor::new(ArtifactCoordinator::new(
            InMemoryPersistenceProvider::new(),
        ));
        let mut application = RecorderApplication::new(session, capture, processor);
        let configuration = RecordingConfiguration::default();

        application.start(&configuration).unwrap();
        application.ready().unwrap();
        application
            .emit_sync_signet(&configuration.signets().opening())
            .unwrap();

        assert!(application.emit_optional_sync_signet(&configuration.signets().closing()) == false);
        let artifact = application
            .stop(RecordingArtifactAssociation::new(
                "production-fallback",
                "recording-fallback",
            ))
            .expect("Closing failure must not block technical stop");
        assert_eq!(artifact.id.value(), "application-test-capture");
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
        application.ready().unwrap();

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
