use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{
    RecordingArtifactId, RecordingClosingOutcome, RecordingId, RecordingStopCoordinator,
    RecordingStopCoordinatorError, RecordingStopMode, RecordingWorkflow, RecordingWorkflowError,
};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::{ProductionSession, ProductionSessionError};
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::artifact::{RecordingArtifact, RecordingArtifactAssociation};
use recorder::audio::{CaptureProvider, RecordingConfiguration};
use recorder::persistence::PersistenceProvider;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecuteRecordingStopError<E> {
    Repository(E),
    Session(ProductionSessionError),
    Workflow(RecordingWorkflowError),
    Coordinator(RecordingStopCoordinatorError),
    Recorder(RecorderApplicationError),
}

/// Executes the host-neutral distributed stop sequence for one recording.
///
/// The Core stop is persisted before any Closing attempt. Closing is explicitly
/// best effort and never blocks technical capture stop or completion. This is
/// the application-level integration point for ADR-080i; transport and capture
/// implementations remain below this boundary.
pub fn execute_recording_stop<R, C, P>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    workflow: &mut RecordingWorkflow,
    recorder: &mut RecorderApplication<C, P>,
    configuration: &RecordingConfiguration,
    mode: RecordingStopMode,
) -> Result<RecordingArtifact, ExecuteRecordingStopError<R::Error>>
where
    R: ProductionSessionRepository,
    C: CaptureProvider,
    P: PersistenceProvider,
{
    let mut coordinator = RecordingStopCoordinator::new(mode);

    match mode {
        RecordingStopMode::Host => coordinator
            .persist_core_stop()
            .map_err(ExecuteRecordingStopError::Coordinator)?,
        RecordingStopMode::Safety => coordinator
            .safety_stop()
            .map_err(ExecuteRecordingStopError::Coordinator)?,
    }

    let mut session = repository
        .get(production_id)
        .map_err(ExecuteRecordingStopError::Repository)?
        .ok_or(ExecuteRecordingStopError::Repository(
            repository
                .get(production_id)
                .err()
                .expect("repository lookup above must provide the missing-session error"),
        ))?;

    // The repository lookup above is only used to obtain the authoritative
    // session snapshot. A missing session cannot be represented by the generic
    // repository error contract, so callers should normally resolve the session
    // before invoking this coordinator.
    stop_session_recording(&mut session, actor, recording_id, mode)
        .map_err(ExecuteRecordingStopError::Session)?;
    repository
        .update(&session)
        .map_err(ExecuteRecordingStopError::Repository)?;

    if mode == RecordingStopMode::Host {
        let outcome = match configuration.signets().closing() {
            Some(closing) if recorder.emit_optional_sync_signet(&closing) => {
                RecordingClosingOutcome::Emitted
            }
            Some(_) => RecordingClosingOutcome::Unavailable,
            None => RecordingClosingOutcome::NotAttempted,
        };
        coordinator
            .record_closing_outcome(outcome)
            .map_err(ExecuteRecordingStopError::Coordinator)?;
    }

    coordinator
        .begin_technical_stop()
        .map_err(ExecuteRecordingStopError::Coordinator)?;

    let artifact = recorder
        .stop(RecordingArtifactAssociation::new(
            production_id.value(),
            recording_id.value(),
        ))
        .map_err(ExecuteRecordingStopError::Recorder)?;

    workflow
        .acknowledge_stop(actor)
        .map_err(ExecuteRecordingStopError::Workflow)?;
    workflow
        .complete(RecordingArtifactId::new(artifact.id.value()))
        .map_err(ExecuteRecordingStopError::Workflow)?;

    session
        .complete_recording_by(
            actor,
            recording_id,
            RecordingArtifactId::new(artifact.id.value()),
        )
        .map_err(ExecuteRecordingStopError::Session)?;
    repository
        .update(&session)
        .map_err(ExecuteRecordingStopError::Repository)?;

    coordinator
        .complete()
        .map_err(ExecuteRecordingStopError::Coordinator)?;

    Ok(artifact)
}

fn stop_session_recording(
    session: &mut ProductionSession,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    mode: RecordingStopMode,
) -> Result<(), ProductionSessionError> {
    match mode {
        RecordingStopMode::Host => session.stop_recording_by(actor, recording_id),
        RecordingStopMode::Safety => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_mode_requires_core_stop_before_closing() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        assert_eq!(
            coordinator.record_closing_outcome(RecordingClosingOutcome::Emitted),
            Err(RecordingStopCoordinatorError::ClosingNotAllowed)
        );
        coordinator.persist_core_stop().unwrap();
        coordinator
            .record_closing_outcome(RecordingClosingOutcome::NotAttempted)
            .unwrap();
        coordinator.begin_technical_stop().unwrap();
    }

    #[test]
    fn safety_mode_has_no_closing_step() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Safety);
        coordinator.safety_stop().unwrap();
        assert_eq!(coordinator.closing(), None);
        coordinator.begin_technical_stop().err();
        assert_eq!(
            coordinator.status(),
            nc_pore_core::recording::RecordingStopCoordinatorStatus::TechnicalStopping
        );
    }
}
