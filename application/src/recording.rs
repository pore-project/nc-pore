use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{
    RecordingArtifactId, RecordingId, RecordingSyncSignet, RecordingWorkflow,
    RecordingWorkflowError,
};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
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
///
/// For ADR-068 the Opening Sync Signet is obtained from the domain workflow
/// after the READY barrier and is then emitted into the already-running local
/// capture. In a future multi-recorder host/client path the same signet value
/// is the transport-neutral value to distribute to every participating client.
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

    session
        .start_recording_by(actor, recording_id)
        .map_err(ExecuteRecordingError::Session)?;

    recorder
        .start(configuration)
        .map_err(ExecuteRecordingError::RecorderStart)?;

    workflow
        .mark_ready(actor)
        .map_err(ExecuteRecordingError::Workflow)?;
    recorder.ready().map_err(|error| {
        ExecuteRecordingError::Recorder(RecorderApplicationError::Capture(format!(
            "recorder ready transition failed: {error:?}"
        )))
    })?;

    let opening_signet = workflow
        .start_recording_with_signet()
        .map_err(ExecuteRecordingError::Workflow)?;

    emit_opening_signet(recorder, opening_signet)?;

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

fn emit_opening_signet<C, P>(
    recorder: &mut RecorderApplication<C, P>,
    signet: RecordingSyncSignet,
) -> Result<(), ExecuteRecordingError<impl std::fmt::Debug>>
where
    C: CaptureProvider,
    P: PersistenceProvider,
{
    recorder.emit_sync_signet(signet).map_err(|error| {
        ExecuteRecordingError::Recorder(RecorderApplicationError::Capture(format!(
            "opening sync signet emission failed: {error:?}"
        )))
    })
}
