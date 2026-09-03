use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{RecordingId, RecordingSyncSignet, RecordingWorkflowError};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::artifact::RecordingArtifact;
use recorder::audio::{CaptureProvider, CaptureStartError, RecordingConfiguration};
use recorder::persistence::PersistenceProvider;

use crate::distributed_recording::{
    begin_distributed_recording, mark_distributed_recording_ready, DistributedRecording,
    DistributedRecordingError,
};
use crate::recording_stop::{execute_recording_stop, ExecuteRecordingStopError};

#[derive(Debug, PartialEq, Eq)]
pub enum ExecuteRecordingError<E> {
    SessionNotFound,
    RecordingNotFound,
    Repository(E),
    Session(ProductionSessionError),
    Workflow(RecordingWorkflowError),
    RecorderStart(CaptureStartError),
    Recorder(RecorderApplicationError),
    CoordinationDiverged,
    Stop(ExecuteRecordingStopError<E>),
}

fn map_distributed_error<E>(error: DistributedRecordingError<E>) -> ExecuteRecordingError<E> {
    match error {
        DistributedRecordingError::SessionNotFound => ExecuteRecordingError::SessionNotFound,
        DistributedRecordingError::RecordingNotFound => ExecuteRecordingError::RecordingNotFound,
        DistributedRecordingError::Repository(error) => ExecuteRecordingError::Repository(error),
        DistributedRecordingError::Session(error) => ExecuteRecordingError::Session(error),
        DistributedRecordingError::Workflow(error) => ExecuteRecordingError::Workflow(error),
        DistributedRecordingError::RecorderStart(error) => {
            ExecuteRecordingError::RecorderStart(error)
        }
        DistributedRecordingError::Recorder(error) => ExecuteRecordingError::Recorder(error),
        DistributedRecordingError::CoordinationDiverged => {
            ExecuteRecordingError::CoordinationDiverged
        }
    }
}

/// Orchestrates one complete production recording lifecycle.
///
/// The distributed recording coordinator is the single application-level
/// start path. Core freezes the complete recording participant set before
/// local capture begins; each participant must then complete local technical
/// READY before that participant is marked READY in Core. Opening can only be
/// triggered after the complete frozen set is READY.
///
/// ADR-068 signet semantics come from the core domain, while the concrete
/// signet description is supplied by the technical recording configuration.
/// The Opening Signet is emitted while local capture is active and before the
/// workflow enters the stable Recording state.
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
    let mut distributed =
        begin_distributed_recording(repository, production_id, actor, recording_id)
            .map_err(map_distributed_error)?;

    // Local technical readiness must be established before this participant
    // can contribute READY to Core's distributed barrier.
    distributed
        .prepare_local_recorder(recorder, configuration)
        .map_err(|error| match error {
            DistributedRecordingError::RecorderStart(error) => {
                ExecuteRecordingError::RecorderStart(error)
            }
            DistributedRecordingError::Recorder(error) => ExecuteRecordingError::Recorder(error),
            _ => unreachable!("local recorder preparation cannot return this error"),
        })?;

    let all_ready = mark_distributed_recording_ready(repository, &mut distributed, actor)
        .map_err(map_distributed_error)?;
    debug_assert!(all_ready);

    // For this synchronous application entry point, the actor is the only
    // locally available recorder. The distributed coordinator nevertheless
    // derives the participant set from Core and refuses Opening unless all
    // frozen participants are READY. A future remote-client path calls the
    // same mark_distributed_recording_ready operation for each remote recorder.
    let opening = distributed
        .trigger_opening()
        .map_err(ExecuteRecordingError::Workflow)?;
    debug_assert_eq!(opening, RecordingSyncSignet::Opening);

    distributed
        .confirm_opening(repository, recorder, &configuration.signets().opening())
        .map_err(map_distributed_error)?;

    // ADR-080i: once the fachliche Recording state is persisted, all stop
    // ordering is delegated to the dedicated host-stop coordinator. In
    // particular, Core STOP is persisted before Closing is attempted.
    execute_recording_stop(
        repository,
        production_id,
        actor,
        recording_id,
        distributed.workflow_mut(),
        recorder,
        configuration,
    )
    .map_err(ExecuteRecordingError::Stop)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::repository::ProductionSessionRepository;
    use nc_pore_core::session::ProductionSession;
    use recorder::artifact::coordination::ArtifactCoordinator;
    use recorder::artifact::processing::RecordingArtifactProcessor;
    use recorder::audio::{
        CaptureProvider, CaptureResult, CpalCaptureProvider, SignetEvent, SyncSignet,
        SyncSignetConfiguration, SyncSignetKind,
    };
    use recorder::persistence::InMemoryPersistenceProvider;
    use recorder::session::RecordingSession;
    use std::cell::RefCell;
    use std::rc::Rc;

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

    struct TestCaptureProvider {
        emitted: Rc<RefCell<Vec<SyncSignet>>>,
    }

    impl CaptureProvider for TestCaptureProvider {
        fn start_capture(
            &mut self,
            _configuration: &RecordingConfiguration,
        ) -> Result<(), CaptureStartError> {
            Ok(())
        }

        fn emit_sync_signet(
            &mut self,
            signet: &SyncSignet,
        ) -> Result<(), recorder::audio::SyncSignetEmissionError> {
            self.emitted.borrow_mut().push(*signet);
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

        fn emit_sync_signet(
            &mut self,
            signet: &SyncSignet,
        ) -> Result<(), recorder::audio::SyncSignetEmissionError> {
            self.provider.emit_sync_signet(signet)
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
        emitted: Rc<RefCell<Vec<SyncSignet>>>,
    ) -> RecorderApplication<TestCaptureProvider, InMemoryPersistenceProvider> {
        let session = RecordingSession::new("recording-001");
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let processor = RecordingArtifactProcessor::new(coordinator);
        RecorderApplication::new(session, TestCaptureProvider { emitted }, processor)
    }

    // TEST-01
    // Verify: the application layer completes the domain and technical flow,
    // with Opening emitted before the stable Recording state is reached.
    #[test]
    fn execute_recording_completes_domain_and_technical_flow_with_configured_signets() {
        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let mut recorder = recorder_application(Rc::clone(&emitted));
        let opening = SyncSignet::new(
            SyncSignetKind::Opening,
            [
                SignetEvent::new(0, 10),
                SignetEvent::new(50, 10),
                SignetEvent::new(100, 10),
            ],
            0.05,
            42,
        );
        let configuration = RecordingConfiguration::with_signets(
            48_000,
            1,
            recorder::audio::SampleFormat::Pcm24,
            recorder::audio::RecordingChunkDuration::OneMinute,
            SyncSignetConfiguration::new(opening, None),
        );

        let artifact = execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &configuration,
        )
        .unwrap();

        assert_eq!(emitted.borrow().as_slice(), &[opening]);
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
    // Verify: Closing remains optional while Opening is always emitted.
    #[test]
    fn execute_recording_uses_default_signets_when_requested() {
        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let mut recorder = recorder_application(Rc::clone(&emitted));

        execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &RecordingConfiguration::default(),
        )
        .unwrap();

        assert_eq!(
            emitted
                .borrow()
                .iter()
                .map(|signet| signet.kind())
                .collect::<Vec<_>>(),
            vec![SyncSignetKind::Opening, SyncSignetKind::Closing]
        );
    }

    // TEST-03
    // Verify: A recorder start failure does not mark the participant READY in
    // Core because local technical readiness must precede the Core READY call.
    #[test]
    fn execute_recording_does_not_mark_core_ready_after_failed_start() {
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
        let coordination = session.recording_coordination().unwrap();
        assert!(!coordination.ready_participants().contains(&actor));
    }

    // TEST-04
    // Verify: A single-participant recording uses the same distributed path as
    // a multi-participant recording rather than inventing an actor-only workflow.
    #[test]
    fn execute_recording_persists_distributed_coordination_for_single_participant() {
        let (mut repository, production_id, actor, recording_id) = repository_with_recording();
        let emitted = Rc::new(RefCell::new(Vec::new()));
        let mut recorder = recorder_application(emitted);

        execute_recording(
            &mut repository,
            &production_id,
            &actor,
            &recording_id,
            &mut recorder,
            &RecordingConfiguration::default(),
        )
        .unwrap();

        let session = repository.get(&production_id).unwrap().unwrap();
        let coordination = session.recording_coordination().unwrap();
        assert_eq!(coordination.participants(), &[actor.clone()]);
        assert_eq!(coordination.ready_participants(), &[actor]);
    }

    // TEST-05
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
