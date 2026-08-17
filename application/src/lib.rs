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
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::audio::{CaptureProvider, CaptureStartError, RecordingConfiguration};
use recorder::persistence::PersistenceProvider;

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
