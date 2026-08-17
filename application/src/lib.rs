//! Application layer for NC-PoRe.
//!
//! The application layer orchestrates domain aggregates and technical
//! boundaries. It decides the order in which domain state and technical
//! recorder operations are coordinated; it does not contain domain rules
//! and it does not implement audio capture.
//!
//! Dependencies point inward:
//!
//! ```text
//! application ──► core
//!      │
//!      └──────► recorder boundary
//! ```

use nc_pore_core::identity::ProductionId;
use nc_pore_core::recording::{RecordingArtifactId, RecordingId};
use nc_pore_core::session::{repository::ProductionSessionRepository, ProductionSessionError};
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::audio::{CaptureProvider, CaptureStartError, RecordingConfiguration};
use recorder::persistence::{PersistenceProvider, PersistenceRecoveryLookup};

/// Technical recorder boundary used by recording lifecycle use cases.
pub trait RecorderPort<C> {
    type Error;

    fn start(&mut self, configuration: &C) -> Result<(), Self::Error>;

    fn complete(
        &mut self,
        production_id: &ProductionId,
        recording_id: &RecordingId,
    ) -> Result<RecordingArtifactId, Self::Error>;
}

/// Adapter from the concrete technical RecorderApplication to the application
/// boundary. Technical ArtifactId is converted to the opaque domain reference
/// here, never in Core.
pub struct RecorderApplicationAdapter<C, P>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    recorder: RecorderApplication<C, P>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecorderBoundaryError {
    Start(String),
    Recorder(RecorderApplicationError),
}

impl<C, P> RecorderApplicationAdapter<C, P>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    pub fn new(recorder: RecorderApplication<C, P>) -> Self {
        Self { recorder }
    }
}

impl<C, P> RecorderPort<RecordingConfiguration> for RecorderApplicationAdapter<C, P>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    type Error = RecorderBoundaryError;

    fn start(&mut self, configuration: &RecordingConfiguration) -> Result<(), Self::Error> {
        self.recorder
            .start(configuration)
            .map_err(|error: CaptureStartError| RecorderBoundaryError::Start(format!("{error:?}")))
    }

    fn complete(
        &mut self,
        production_id: &ProductionId,
        recording_id: &RecordingId,
    ) -> Result<RecordingArtifactId, Self::Error> {
        let artifact = self
            .recorder
            .stop(recorder::artifact::RecordingArtifactAssociation::new(
                production_id.value(),
                recording_id.value(),
            ))
            .map_err(RecorderBoundaryError::Recorder)?;

        Ok(RecordingArtifactId::new(artifact.id.value()))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum StartRecordingError<RE, TE> {
    Repository(RE),
    Domain(ProductionSessionError),
    Recorder(TE),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompleteRecordingError<RE, TE> {
    Repository(RE),
    Domain(ProductionSessionError),
    Recorder(TE),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingRecoveryOutcome {
    Recovered { artifact_id: RecordingArtifactId },
    AlreadyCompleted { artifact_id: RecordingArtifactId },
    Incomplete,
    Inconsistent { artifact_id: String },
    Conflict { artifact_ids: Vec<String> },
    NotFound,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RecoverRecordingError<RE> {
    Repository(RE),
    Domain(ProductionSessionError),
}

/// Application use case for recovering one concrete domain Recording.
///
/// Recovery is explicitly requested for the production/recording pair. The
/// persistence boundary supplies technical evidence; the domain aggregate
/// remains the authority for whether that evidence can complete the recording.
/// The recorder itself is not involved in the recovery decision.
pub struct RecoverRecordingUseCase<'a, R, P>
where
    R: ProductionSessionRepository,
    P: PersistenceProvider,
{
    repository: &'a mut R,
    persistence: &'a P,
}

impl<'a, R, P> RecoverRecordingUseCase<'a, R, P>
where
    R: ProductionSessionRepository,
    P: PersistenceProvider,
{
    pub fn new(repository: &'a mut R, persistence: &'a P) -> Self {
        Self {
            repository,
            persistence,
        }
    }

    pub fn execute(
        &mut self,
        production_id: &ProductionId,
        recording_id: &RecordingId,
    ) -> Result<RecordingRecoveryOutcome, RecoverRecordingError<R::Error>> {
        let mut session = self
            .repository
            .get(production_id)
            .map_err(RecoverRecordingError::Repository)?
            .ok_or(RecoverRecordingError::Domain(
                ProductionSessionError::RecordingNotFound,
            ))?;

        if !session
            .recordings()
            .iter()
            .any(|recording| recording.id() == recording_id)
        {
            return Err(RecoverRecordingError::Domain(
                ProductionSessionError::RecordingNotFound,
            ));
        }

        match self
            .persistence
            .find_for_recording(production_id.value(), recording_id.value())
        {
            PersistenceRecoveryLookup::Valid(artifact) => {
                let artifact_id = RecordingArtifactId::new(artifact.id.value());
                let already_completed = session
                    .recordings()
                    .iter()
                    .find(|recording| recording.id() == recording_id)
                    .and_then(|recording| recording.artifact_id())
                    == Some(&artifact_id);

                session
                    .complete_recording(recording_id, artifact_id.clone())
                    .map_err(RecoverRecordingError::Domain)?;

                self.repository
                    .update(&session)
                    .map_err(RecoverRecordingError::Repository)?;

                if already_completed {
                    Ok(RecordingRecoveryOutcome::AlreadyCompleted { artifact_id })
                } else {
                    Ok(RecordingRecoveryOutcome::Recovered { artifact_id })
                }
            }
            PersistenceRecoveryLookup::Incomplete { artifact_id } => {
                Ok(RecordingRecoveryOutcome::Incomplete)
            }
            PersistenceRecoveryLookup::Inconsistent { artifact_id } => {
                Ok(RecordingRecoveryOutcome::Inconsistent { artifact_id })
            }
            PersistenceRecoveryLookup::NotFound => Ok(RecordingRecoveryOutcome::NotFound),
            PersistenceRecoveryLookup::Conflict { artifact_ids } => {
                Ok(RecordingRecoveryOutcome::Conflict { artifact_ids })
            }
        }
    }
}

/// Application use case for starting a domain Recording.
///
/// The domain transition is persisted before the technical recorder starts.
/// If the technical start subsequently fails, the persisted Recording remains
/// in `Recording` state with no artifact association. This is an intentional
/// interrupted-recording state and is handled by #71.
pub struct StartRecordingUseCase<'a, R, T, C>
where
    R: ProductionSessionRepository,
    T: RecorderPort<C>,
{
    repository: &'a mut R,
    recorder: &'a mut T,
    _configuration: std::marker::PhantomData<C>,
}

impl<'a, R, T, C> StartRecordingUseCase<'a, R, T, C>
where
    R: ProductionSessionRepository,
    T: RecorderPort<C>,
{
    pub fn new(repository: &'a mut R, recorder: &'a mut T) -> Self {
        Self {
            repository,
            recorder,
            _configuration: std::marker::PhantomData,
        }
    }

    pub fn execute(
        &mut self,
        production_id: &ProductionId,
        recording_id: &RecordingId,
        configuration: &C,
    ) -> Result<(), StartRecordingError<R::Error, T::Error>> {
        let mut session = self
            .repository
            .get(production_id)
            .map_err(StartRecordingError::Repository)?
            .ok_or(StartRecordingError::Domain(
                ProductionSessionError::RecordingNotFound,
            ))?;

        session
            .start_recording(recording_id)
            .map_err(StartRecordingError::Domain)?;

        self.repository
            .update(&session)
            .map_err(StartRecordingError::Repository)?;

        self.recorder
            .start(configuration)
            .map_err(StartRecordingError::Recorder)
    }
}

/// Application use case for completing a domain Recording.
///
/// The technical recorder is completed first because its result supplies the
/// `RecordingArtifactId` required by the domain completion transition. The
/// domain aggregate is persisted only after that transition succeeds.
pub struct CompleteRecordingUseCase<'a, R, T, C>
where
    R: ProductionSessionRepository,
    T: RecorderPort<C>,
{
    repository: &'a mut R,
    recorder: &'a mut T,
    _configuration: std::marker::PhantomData<C>,
}

impl<'a, R, T, C> CompleteRecordingUseCase<'a, R, T, C>
where
    R: ProductionSessionRepository,
    T: RecorderPort<C>,
{
    pub fn new(repository: &'a mut R, recorder: &'a mut T) -> Self {
        Self {
            repository,
            recorder,
            _configuration: std::marker::PhantomData,
        }
    }

    pub fn execute(
        &mut self,
        production_id: &ProductionId,
        recording_id: &RecordingId,
    ) -> Result<(), CompleteRecordingError<R::Error, T::Error>> {
        let mut session = self
            .repository
            .get(production_id)
            .map_err(CompleteRecordingError::Repository)?
            .ok_or(CompleteRecordingError::Domain(
                ProductionSessionError::RecordingNotFound,
            ))?;

        let artifact_id = self
            .recorder
            .complete(&session.id, recording_id)
            .map_err(CompleteRecordingError::Recorder)?;

        session
            .complete_recording(recording_id, artifact_id)
            .map_err(CompleteRecordingError::Domain)?;

        self.repository
            .update(&session)
            .map_err(CompleteRecordingError::Repository)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::session::ProductionSession;
    use recorder::artifact::RecordingArtifact;
    use recorder::session::RecordingSessionId;

    struct InMemoryRepository {
        session: Option<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemoryRepository {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.session = Some(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.session = Some(session.clone());
            Ok(())
        }

        fn get(&self, _id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self.session.clone())
        }
    }

    struct TestPersistence {
        outcome: PersistenceRecoveryLookup,
    }

    impl PersistenceProvider for TestPersistence {
        fn store(&mut self, _artifact: RecordingArtifact) {}

        fn load(&self, _id: &str) -> recorder::persistence::PersistenceLoadResult {
            recorder::persistence::PersistenceLoadResult::NotFound
        }

        fn list_ids(&self) -> Vec<String> {
            Vec::new()
        }

        fn list(&self) -> Vec<RecordingArtifact> {
            Vec::new()
        }

        fn remove(&mut self, _id: &str) {}

        fn find_for_recording(
            &self,
            _production_id: &str,
            _recording_id: &str,
        ) -> PersistenceRecoveryLookup {
            self.outcome.clone()
        }
    }

    struct TestRecorder {
        events: Vec<&'static str>,
        artifact_id: RecordingArtifactId,
    }

    impl RecorderPort<()> for TestRecorder {
        type Error = &'static str;

        fn start(&mut self, _configuration: &()) -> Result<(), Self::Error> {
            self.events.push("recorder.start");
            Ok(())
        }

        fn complete(
            &mut self,
            _production_id: &ProductionId,
            _recording_id: &RecordingId,
        ) -> Result<RecordingArtifactId, Self::Error> {
            self.events.push("recorder.complete");
            Ok(self.artifact_id.clone())
        }
    }

    fn repository_with_recording() -> (InMemoryRepository, ProductionId, RecordingId) {
        let production_id = ProductionId::new("production-001");
        let recording_id = RecordingId::new("recording-001");
        let mut session = ProductionSession::new(production_id.clone());
        session.add_recording(Recording::new(recording_id.value()));

        (
            InMemoryRepository {
                session: Some(session),
            },
            production_id,
            recording_id,
        )
    }

    fn recovery_artifact(production_id: &str, recording_id: &str) -> RecordingArtifact {
        RecordingArtifact::new("artifact-recovery-001", RecordingSessionId::new("session-001"))
            .with_association(production_id, recording_id)
    }

    #[test]
    fn recovery_completes_recording_from_valid_artifact() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        repository
            .session
            .as_mut()
            .unwrap()
            .start_recording(&recording_id)
            .unwrap();
        let artifact = recovery_artifact(production_id.value(), recording_id.value());
        let artifact_id = RecordingArtifactId::new(artifact.id.value());
        let mut persistence = TestPersistence {
            outcome: PersistenceRecoveryLookup::Valid(artifact),
        };

        let mut use_case = RecoverRecordingUseCase::new(&mut repository, &persistence);
        let result = use_case.execute(&production_id, &recording_id).unwrap();

        assert_eq!(
            result,
            RecordingRecoveryOutcome::Recovered {
                artifact_id: artifact_id.clone()
            }
        );
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].artifact_id(),
            Some(&artifact_id)
        );
        persistence.outcome = PersistenceRecoveryLookup::NotFound;
    }

    #[test]
    fn recovery_is_idempotent_for_already_completed_recording() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        let artifact = recovery_artifact(production_id.value(), recording_id.value());
        let artifact_id = RecordingArtifactId::new(artifact.id.value());
        repository
            .session
            .as_mut()
            .unwrap()
            .start_recording(&recording_id)
            .unwrap();
        repository
            .session
            .as_mut()
            .unwrap()
            .complete_recording(&recording_id, artifact_id.clone())
            .unwrap();

        let persistence = TestPersistence {
            outcome: PersistenceRecoveryLookup::Valid(artifact),
        };
        let mut use_case = RecoverRecordingUseCase::new(&mut repository, &persistence);

        assert_eq!(
            use_case.execute(&production_id, &recording_id).unwrap(),
            RecordingRecoveryOutcome::AlreadyCompleted { artifact_id }
        );
    }

    #[test]
    fn recovery_does_not_change_domain_without_artifact() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        repository
            .session
            .as_mut()
            .unwrap()
            .start_recording(&recording_id)
            .unwrap();
        let persistence = TestPersistence {
            outcome: PersistenceRecoveryLookup::NotFound,
        };
        let mut use_case = RecoverRecordingUseCase::new(&mut repository, &persistence);

        assert_eq!(
            use_case.execute(&production_id, &recording_id).unwrap(),
            RecordingRecoveryOutcome::NotFound
        );
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].artifact_id(),
            None
        );
    }

    #[test]
    fn recovery_does_not_change_domain_for_conflicting_evidence() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        repository
            .session
            .as_mut()
            .unwrap()
            .start_recording(&recording_id)
            .unwrap();
        let persistence = TestPersistence {
            outcome: PersistenceRecoveryLookup::Conflict {
                artifact_ids: vec!["artifact-a".to_owned(), "artifact-b".to_owned()],
            },
        };
        let mut use_case = RecoverRecordingUseCase::new(&mut repository, &persistence);

        assert_eq!(
            use_case.execute(&production_id, &recording_id).unwrap(),
            RecordingRecoveryOutcome::Conflict {
                artifact_ids: vec!["artifact-a".to_owned(), "artifact-b".to_owned()]
            }
        );
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].artifact_id(),
            None
        );
    }

    #[test]
    fn start_use_case_persists_domain_transition_before_starting_recorder() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        let mut recorder = TestRecorder {
            events: Vec::new(),
            artifact_id: RecordingArtifactId::new("artifact-001"),
        };

        let mut use_case = StartRecordingUseCase::<_, _, ()>::new(&mut repository, &mut recorder);
        use_case
            .execute(&production_id, &recording_id, &())
            .unwrap();

        assert_eq!(recorder.events, vec!["recorder.start"]);
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].status(),
            nc_pore_core::recording::RecordingStatus::Recording
        );
    }

    #[test]
    fn complete_use_case_associates_recorder_result_with_domain_recording() {
        let (mut repository, production_id, recording_id) = repository_with_recording();
        repository
            .session
            .as_mut()
            .unwrap()
            .start_recording(&recording_id)
            .unwrap();

        let artifact_id = RecordingArtifactId::new("artifact-001");
        let mut recorder = TestRecorder {
            events: Vec::new(),
            artifact_id: artifact_id.clone(),
        };

        let mut use_case =
            CompleteRecordingUseCase::<_, _, ()>::new(&mut repository, &mut recorder);
        use_case.execute(&production_id, &recording_id).unwrap();

        assert_eq!(recorder.events, vec!["recorder.complete"]);
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].artifact_id(),
            Some(&artifact_id)
        );
    }

    #[test]
    fn recorder_failure_does_not_create_a_failed_domain_state() {
        struct FailingRecorder;

        impl RecorderPort<()> for FailingRecorder {
            type Error = &'static str;

            fn start(&mut self, _configuration: &()) -> Result<(), Self::Error> {
                Err("capture start failed")
            }

            fn complete(
                &mut self,
                _production_id: &ProductionId,
                _recording_id: &RecordingId,
            ) -> Result<RecordingArtifactId, Self::Error> {
                unreachable!()
            }
        }

        let (mut repository, production_id, recording_id) = repository_with_recording();
        let mut recorder = FailingRecorder;
        let mut use_case = StartRecordingUseCase::<_, _, ()>::new(&mut repository, &mut recorder);

        let result = use_case.execute(&production_id, &recording_id, &());

        assert_eq!(
            result,
            Err(StartRecordingError::Recorder("capture start failed"))
        );
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].status(),
            nc_pore_core::recording::RecordingStatus::Recording
        );
        assert_eq!(
            repository.session.as_ref().unwrap().recordings()[0].artifact_id(),
            None
        );
    }
}
