use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{
    RecordingArtifactId, RecordingId, RecordingWorkflow, RecordingWorkflowError,
};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::{ProductionSession, ProductionSessionError};
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::artifact::{RecordingArtifact, RecordingArtifactAssociation};
use recorder::audio::{CaptureProvider, CaptureStartError, RecordingConfiguration};
use recorder::persistence::PersistenceProvider;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecuteRecordingError<E> {
    SessionNotFound,
    RecordingNotFound,
    Repository(E),
    Session(ProductionSessionError),
    Workflow(RecordingWorkflowError),
    RecorderStart(CaptureStartError),
    Recorder(RecorderApplicationError),
}

/// Orchestrates one complete production recording lifecycle.
///
/// The domain workflow owns the recording state machine while the recorder
/// owns capture and artifact processing. The application layer coordinates
/// the two without exposing either implementation detail to the other boundary.
pub fn execute_recording<R, C, P>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    recorder: &mut RecorderApplication<C, P>,
    configuration: &RecordingConfiguration,
) -> Result<RecordingArtifact, ExecuteRecordingError<R::Error>>
where
    R: ProductionSessionRepository,
    C: CaptureProvider,
    P: PersistenceProvider,
{
    let mut session = repository
        .get(production_id)
        .map_err(ExecuteRecordingError::Repository)?
        .ok_or(ExecuteRecordingError::SessionNotFound)?;

    let recording = session
        .recordings()
        .iter()
        .find(|recording| recording.id() == recording_id)
        .cloned()
        .ok_or(ExecuteRecordingError::RecordingNotFound)?;

    let mut workflow = RecordingWorkflow::from_recording(recording, [actor.clone()])
        .map_err(ExecuteRecordingError::Workflow)?;
    workflow
        .begin_ready_phase()
        .map_err(ExecuteRecordingError::Workflow)?;

    recorder
        .start(configuration)
        .map_err(ExecuteRecordingError::RecorderStart)?;

    workflow
        .mark_ready(actor)
        .map_err(ExecuteRecordingError::Workflow)?;
    workflow
        .start_recording()
        .map_err(ExecuteRecordingError::Workflow)?;

    let artifact = recorder
        .stop(RecordingArtifactAssociation::new(
            production_id.value(),
            recording_id.value(),
        ))
        .map_err(ExecuteRecordingError::Recorder)?;

    workflow
        .request_stop()
        .map_err(ExecuteRecordingError::Workflow)?;
    workflow
        .acknowledge_stop(actor)
        .map_err(ExecuteRecordingError::Workflow)?;
    workflow
        .complete(RecordingArtifactId::new(artifact.id.value()))
        .map_err(ExecuteRecordingError::Workflow)?;

    session
        .start_recording_by(actor, recording_id)
        .map_err(ExecuteRecordingError::Session)?;
    session
        .complete_recording_by(
            actor,
            recording_id,
            RecordingArtifactId::new(artifact.id.value()),
        )
        .map_err(ExecuteRecordingError::Session)?;

    repository
        .update(&session)
        .map_err(ExecuteRecordingError::Repository)?;

    Ok(artifact)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::repository::ProductionSessionRepository;
    use recorder::artifact::coordination::ArtifactCoordinator;
    use recorder::artifact::processing::RecordingArtifactProcessor;
    use recorder::audio::{CaptureProvider, CaptureResult, CpalCaptureProvider};
    use recorder::persistence::InMemoryPersistenceProvider;
    use recorder::session::RecordingSession;

    struct InMemorySessions {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemorySessions {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            if self
                .sessions
                .iter()
                .any(|existing| existing.id == session.id)
            {
                return Err("session already exists");
            }
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .map(|existing| *existing = session.clone())
                .ok_or("session not found")
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

    struct TestCaptureProvider;

    impl CaptureProvider for TestCaptureProvider {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), CaptureStartError> {
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            CaptureResult::new("vertical-slice-artifact")
        }
    }

    struct TimedCpalCaptureProvider {
        provider: CpalCaptureProvider,
        duration: std::time::Duration,
    }

    impl CaptureProvider for TimedCpalCaptureProvider {
        fn start_capture(
            &mut self,
            configuration: &RecordingConfiguration,
        ) -> Result<(), CaptureStartError> {
            self.provider.start_capture(configuration)?;
            std::thread::sleep(self.duration);
            Ok(())
        }

        fn stop_capture(&mut self) -> CaptureResult {
            self.provider.stop_capture()
        }
    }

    fn owner() -> ParticipantId {
        ParticipantId::new("owner-1")
    }

    fn repository_with_recording() -> (InMemorySessions, ProductionId, ParticipantId, RecordingId) {
        let mut repository = InMemorySessions {
            sessions: Vec::new(),
        };
        let production_id = ProductionId::new("production-001");
        let actor = owner();
        let recording_id = RecordingId::new("recording-001");
        let mut session =
            ProductionSession::new_with_actor(production_id.clone(), Some(actor.clone()));

        session
            .add_participation_by(
                &actor,
                Participation::with_roles(
                    actor.clone(),
                    [
                        ParticipantRole::Owner,
                        ParticipantRole::Producer,
                        ParticipantRole::Participant,
                    ],
                ),
            )
            .unwrap();
        session.start_by(&actor).unwrap();
        session
            .add_recording_by(&actor, Recording::new(recording_id.value()))
            .unwrap();
        repository.store(&session).unwrap();

        (repository, production_id, actor, recording_id)
    }

    fn recorder_application(
    ) -> RecorderApplication<TestCaptureProvider, InMemoryPersistenceProvider> {
        let session = RecordingSession::new("recording-001");
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let processor = RecordingArtifactProcessor::new(coordinator);
        RecorderApplication::new(session, TestCaptureProvider, processor)
    }

    // TEST-01
    //
    // Verify: The application layer drives the domain workflow through
    // ready-gating, recording, stop acknowledgement, and completion while
    // the recorder remains responsible for technical capture and artifacts.
    #[test]
    fn execute_recording_completes_domain_and_technical_flow() {
        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let mut recorder = recorder_application();

        let artifact = execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &RecordingConfiguration::default(),
        )
        .unwrap();

        let session = repository.get(&production_id).unwrap().unwrap();
        let recording = &session.recordings()[0];

        assert_eq!(
            recording.status(),
            nc_pore_core::recording::RecordingStatus::Completed
        );
        assert_eq!(
            recording.artifact_id().unwrap().value(),
            artifact.id.value()
        );
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-001"));
        assert_eq!(artifact.id.value(), "vertical-slice-artifact");
    }

    // TEST-02
    //
    // Verify: A recorder start failure does not persist a partially advanced
    // domain session because the repository update occurs only after capture
    // and workflow completion succeed.
    #[test]
    fn execute_recording_does_not_persist_failed_start() {
        struct FailingCaptureProvider;

        impl CaptureProvider for FailingCaptureProvider {
            fn start_capture(
                &mut self,
                _configuration: &RecordingConfiguration,
            ) -> Result<(), CaptureStartError> {
                Err(CaptureStartError::DeviceUnavailable)
            }

            fn stop_capture(&mut self) -> CaptureResult {
                unreachable!("capture must not stop after a failed start")
            }
        }

        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let processor = RecordingArtifactProcessor::new(coordinator);
        let mut recorder = RecorderApplication::new(
            RecordingSession::new("recording-001"),
            FailingCaptureProvider,
            processor,
        );

        let result = execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &RecordingConfiguration::default(),
        );

        assert!(matches!(
            result,
            Err(ExecuteRecordingError::RecorderStart(
                CaptureStartError::DeviceUnavailable
            ))
        ));
        let session = repository.get(&production_id).unwrap().unwrap();
        assert_eq!(
            session.recordings()[0].status(),
            nc_pore_core::recording::RecordingStatus::Prepared
        );
    }

    // TEST-03
    //
    // Verify: The existing CPAL provider can drive the complete application
    // recording path and produce a persisted artifact with real payload data.
    //
    // This test is intentionally ignored in CI because it requires a physical
    // default input device. Run it explicitly on a machine with a microphone.
    #[test]
    #[ignore = "requires a physical default input device"]
    fn execute_recording_with_real_cpal_capture_produces_payload() {
        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let session = RecordingSession::new("recording-001");
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let processor = RecordingArtifactProcessor::new(coordinator);
        let capture = TimedCpalCaptureProvider {
            provider: CpalCaptureProvider::new(),
            duration: std::time::Duration::from_secs(1),
        };
        let mut recorder = RecorderApplication::new(session, capture, processor);

        let artifact = execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &RecordingConfiguration::default(),
        )
        .expect("real CPAL recording should complete successfully");

        let session = repository.get(&production_id).unwrap().unwrap();
        let recording = &session.recordings()[0];

        assert_eq!(
            recording.status(),
            nc_pore_core::recording::RecordingStatus::Completed
        );
        assert_eq!(
            recording.artifact_id().unwrap().value(),
            artifact.id.value()
        );
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-001"));
        assert_eq!(
            artifact.status(),
            &recorder::artifact::ArtifactStatus::Stored
        );
        assert!(
            !artifact.tracks().is_empty(),
            "real capture must produce a track"
        );

        let track = &artifact.tracks()[0];
        assert_eq!(
            track.configuration(),
            Some(RecordingConfiguration::default())
        );
        assert!(
            !track.chunks().is_empty(),
            "real capture must produce a chunk"
        );
        assert!(
            track
                .chunks()
                .iter()
                .all(|chunk| !chunk.payload().data().is_empty()),
            "real capture chunks must contain payload bytes"
        );
        assert_eq!(track.chunks()[0].sequence, 1);
    }
}
