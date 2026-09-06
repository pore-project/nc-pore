use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{RecordingId, RecordingWorkflow, RecordingWorkflowError};

/// Host-neutral application orchestration for a recording session.
///
/// This layer translates host/user intent into Core workflow commands and
/// exposes Core state. It deliberately knows nothing about Nextcloud, Talk,
/// browser capture, or artifact transport.
#[derive(Debug)]
pub struct RecordingCoordinator {
    workflow: RecordingWorkflow,
    production_id: ProductionId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingCoordinatorSnapshot {
    pub production_id: String,
    pub recording_id: String,
    pub status: nc_pore_core::recording::RecordingWorkflowStatus,
    pub participants: Vec<String>,
    pub ready_participants: Vec<String>,
}

impl RecordingCoordinator {
    pub fn new(
        production_id: ProductionId,
        recording_id: RecordingId,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<Self, RecordingWorkflowError> {
        let workflow = RecordingWorkflow::from_recording(
            nc_pore_core::recording::Recording::new(recording_id.value()),
            participants,
        )?;
        Ok(Self { workflow, production_id })
    }

    pub fn begin(&mut self) -> Result<RecordingCoordinatorSnapshot, RecordingWorkflowError> {
        self.workflow.begin_ready_phase()?;
        Ok(self.snapshot())
    }

    pub fn mark_ready(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<RecordingCoordinatorSnapshot, RecordingWorkflowError> {
        self.workflow.mark_ready(participant_id)?;
        Ok(self.snapshot())
    }

    pub fn start(&mut self) -> Result<RecordingCoordinatorSnapshot, RecordingWorkflowError> {
        self.workflow.start_recording()?;
        Ok(self.snapshot())
    }

    pub fn request_stop(&mut self) -> Result<RecordingCoordinatorSnapshot, RecordingWorkflowError> {
        self.workflow.request_stop()?;
        Ok(self.snapshot())
    }

    pub fn acknowledge_stop(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingWorkflowError> {
        self.workflow.acknowledge_stop(participant_id)
    }

    pub fn snapshot(&self) -> RecordingCoordinatorSnapshot {
        RecordingCoordinatorSnapshot {
            production_id: self.production_id.value().to_owned(),
            recording_id: self.workflow.recording().id().value().to_owned(),
            status: self.workflow.status(),
            participants: self
                .workflow
                .coordination()
                .participants()
                .iter()
                .map(|id| id.value().to_owned())
                .collect(),
            ready_participants: self
                .workflow
                .coordination()
                .ready_participants()
                .iter()
                .map(|id| id.value().to_owned())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(id: &str) -> ParticipantId { ParticipantId::new(id) }

    #[test]
    fn coordinator_delegates_lifecycle_to_core() {
        let mut coordinator = RecordingCoordinator::new(
            ProductionId::new("production-001"),
            RecordingId::new("recording-001"),
            [participant("alice"), participant("bob")],
        )
        .unwrap();

        assert_eq!(coordinator.snapshot().status, nc_pore_core::recording::RecordingWorkflowStatus::Preparing);
        coordinator.begin().unwrap();
        coordinator.mark_ready(&participant("alice")).unwrap();
        assert_eq!(coordinator.snapshot().status, nc_pore_core::recording::RecordingWorkflowStatus::WaitingForReady);
        coordinator.mark_ready(&participant("bob")).unwrap();
        assert_eq!(coordinator.snapshot().status, nc_pore_core::recording::RecordingWorkflowStatus::Ready);
        coordinator.start().unwrap();
        assert_eq!(coordinator.snapshot().status, nc_pore_core::recording::RecordingWorkflowStatus::Recording);
        coordinator.request_stop().unwrap();
        assert_eq!(coordinator.snapshot().status, nc_pore_core::recording::RecordingWorkflowStatus::Stopping);
    }

    #[test]
    fn coordinator_rejects_start_until_core_reports_ready() {
        let mut coordinator = RecordingCoordinator::new(
            ProductionId::new("production-001"),
            RecordingId::new("recording-001"),
            [participant("alice"), participant("bob")],
        )
        .unwrap();
        coordinator.begin().unwrap();
        coordinator.mark_ready(&participant("alice")).unwrap();
        assert_eq!(
            coordinator.start(),
            Err(RecordingWorkflowError::InvalidState)
        );
    }
}
