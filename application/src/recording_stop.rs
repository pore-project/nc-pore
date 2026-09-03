use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{
    RecordingArtifactId, RecordingClosingOutcome, RecordingId, RecordingStopCoordinator,
    RecordingStopCoordinatorError, RecordingWorkflow, RecordingWorkflowError,
};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
use recorder::application::{RecorderApplication, RecorderApplicationError};
use recorder::artifact::{RecordingArtifact, RecordingArtifactAssociation};
use recorder::audio::{CaptureProvider, RecordingConfiguration};
use recorder::persistence::PersistenceProvider;

#[derive(Debug, PartialEq, Eq)]
pub enum ExecuteRecordingStopError<E> {
    SessionNotFound,
    Repository(E),
    Session(ProductionSessionError),
    Workflow(RecordingWorkflowError),
    Coordinator(RecordingStopCoordinatorError),
    Recorder(RecorderApplicationError),
}

/// Executes the host-stop sequence defined by ADR-080i.
///
/// The fachliche Core stop is persisted before Closing. Closing is best effort
/// and never blocks technical capture stop or completion. The coordinator is
/// deliberately host-only here; Safety Stop is a separate local path because
/// it must not pretend that a Core stop was persisted.
pub fn execute_recording_stop<R, C, P>(
    repository: &mut R,
    production_id: &ProductionId,
    actor: &ParticipantId,
    recording_id: &RecordingId,
    workflow: &mut RecordingWorkflow,
    recorder: &mut RecorderApplication<C, P>,
    configuration: &RecordingConfiguration,
) -> Result<RecordingArtifact, ExecuteRecordingStopError<R::Error>>
where
    R: ProductionSessionRepository,
    C: CaptureProvider,
    P: PersistenceProvider,
{
    let mut coordinator = RecordingStopCoordinator::new(
        nc_pore_core::recording::RecordingStopMode::Host,
    );
    coordinator
        .persist_core_stop()
        .map_err(ExecuteRecordingStopError::Coordinator)?;

    // Keep the application/domain workflow and the persisted Core session at
    // the same fachlichen stop boundary before any Closing attempt.
    workflow
        .request_stop()
        .map_err(ExecuteRecordingStopError::Workflow)?;

    let mut session = repository
        .get(production_id)
        .map_err(ExecuteRecordingStopError::Repository)?
        .ok_or(ExecuteRecordingStopError::SessionNotFound)?;
    session
        .stop_recording_by(actor, recording_id)
        .map_err(ExecuteRecordingStopError::Session)?;
    repository
        .update(&session)
        .map_err(ExecuteRecordingStopError::Repository)?;

    let closing_outcome = match configuration.signets().closing() {
        Some(closing) if recorder.emit_optional_sync_signet(&closing) => {
            RecordingClosingOutcome::Emitted
        }
        Some(_) => RecordingClosingOutcome::Unavailable,
        None => RecordingClosingOutcome::NotAttempted,
    };
    coordinator
        .record_closing_outcome(closing_outcome)
        .map_err(ExecuteRecordingStopError::Coordinator)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::recording::{RecordingClosingOutcome, RecordingStopCoordinatorStatus, RecordingStopMode};

    #[test]
    fn host_stop_requires_persisted_core_stop_before_closing() {
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
        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::TechnicalStopping
        );
    }

    #[test]
    fn closing_failure_does_not_block_technical_stop() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        coordinator.persist_core_stop().unwrap();
        coordinator
            .record_closing_outcome(RecordingClosingOutcome::Unavailable)
            .unwrap();
        coordinator.begin_technical_stop().unwrap();
        coordinator.complete().unwrap();
        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::Completed
        );
    }
}
