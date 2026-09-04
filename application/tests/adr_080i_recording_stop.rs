use std::sync::{Arc, Mutex};

use nc_pore_application::recording::execute_recording;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::{Recording, RecordingId, RecordingStatus};
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSession;
use recorder::application::RecorderApplication;
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::audio::{CaptureProvider, CaptureResult, CaptureStartError, RecordingConfiguration};
use recorder::persistence::InMemoryPersistenceProvider;
use recorder::session::RecordingSession;

#[derive(Default)]
struct StopBoundaryObservation {
    stopped_persisted: bool,
    capture_stop_observed_persisted: bool,
}

struct ObservingSessions {
    session: ProductionSession,
    observation: Arc<Mutex<StopBoundaryObservation>>,
}

impl ProductionSessionRepository for ObservingSessions {
    type Error = &'static str;

    fn store(&mut self, _session: &ProductionSession) -> Result<(), Self::Error> {
        Err("store is not used in this test")
    }

    fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        let stopped = session
            .recordings()
            .first()
            .map(|recording| recording.status() == RecordingStatus::Stopped)
            .unwrap_or(false);
        if stopped {
            self.observation.lock().unwrap().stopped_persisted = true;
        }
        self.session = session.clone();
        Ok(())
    }

    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
        Ok((self.session.id == *id).then(|| self.session.clone()))
    }
}

struct ObservingCaptureProvider {
    observation: Arc<Mutex<StopBoundaryObservation>>,
}

impl CaptureProvider for ObservingCaptureProvider {
    fn start_capture(
        &mut self,
        _configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError> {
        Ok(())
    }

    fn stop_capture(&mut self) -> CaptureResult {
        let persisted = self.observation.lock().unwrap().stopped_persisted;
        self.observation
            .lock()
            .unwrap()
            .capture_stop_observed_persisted = persisted;
        CaptureResult::new("adr-080i-stop-boundary-artifact")
    }
}

fn owner() -> ParticipantId {
    ParticipantId::new("owner-080i")
}

fn session_with_recording() -> (ProductionSession, ProductionId, ParticipantId, RecordingId) {
    let production_id = ProductionId::new("production-080i");
    let actor = owner();
    let recording_id = RecordingId::new("recording-080i");
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

    (session, production_id, actor, recording_id)
}

#[test]
fn fachlicher_stop_is_persisted_before_technical_capture_stop() {
    let (session, production_id, actor, recording_id) = session_with_recording();
    let observation = Arc::new(Mutex::new(StopBoundaryObservation::default()));
    let mut repository = ObservingSessions {
        session,
        observation: Arc::clone(&observation),
    };

    let persistence = InMemoryPersistenceProvider::new();
    let coordinator = ArtifactCoordinator::new(persistence);
    let processor = RecordingArtifactProcessor::new(coordinator);
    let capture = ObservingCaptureProvider {
        observation: Arc::clone(&observation),
    };
    let mut recorder =
        RecorderApplication::new(RecordingSession::new("recording-080i"), capture, processor);

    execute_recording(
        &mut repository,
        &production_id,
        &actor,
        &recording_id,
        &mut recorder,
        &RecordingConfiguration::default(),
    )
    .unwrap();

    let observation = observation.lock().unwrap();
    assert!(observation.stopped_persisted);
    assert!(observation.capture_stop_observed_persisted);
}
