use super::{
    Recording, RecordingArtifactId, RecordingCoordination, RecordingCoordinationError,
    RecordingLifecycleError, RecordingStatus,
};
use crate::participant::ParticipantId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingWorkflowStatus {
    Preparing,
    WaitingForReady,
    Ready,
    Recording,
    Stopping,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingWorkflowError {
    Coordination(RecordingCoordinationError),
    Lifecycle(RecordingLifecycleError),
    InvalidState,
    ParticipantNotSelected,
    AlreadyReady,
    AlreadyAcknowledged,
}

impl From<RecordingCoordinationError> for RecordingWorkflowError {
    fn from(error: RecordingCoordinationError) -> Self {
        Self::Coordination(error)
    }
}

impl From<RecordingLifecycleError> for RecordingWorkflowError {
    fn from(error: RecordingLifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}

/// Orchestrates the domain-level recording sequence without owning audio
/// capture, clock handling, or sync-signet generation.
///
/// The fachliche stop is part of the Core recording lifecycle. Stop
/// acknowledgements remain technical coordination facts and never form a
/// completion barrier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingWorkflow {
    recording: Recording,
    coordination: RecordingCoordination,
    status: RecordingWorkflowStatus,
    acknowledged: Vec<ParticipantId>,
}

impl RecordingWorkflow {
    pub fn new(
        recording_id: impl Into<String>,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<Self, RecordingWorkflowError> {
        Self::from_recording(Recording::new(recording_id), participants)
    }

    pub fn from_recording(
        recording: Recording,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<Self, RecordingWorkflowError> {
        let coordination = RecordingCoordination::new(recording.id().clone(), participants)?;

        Ok(Self {
            recording,
            coordination,
            status: RecordingWorkflowStatus::Preparing,
            acknowledged: Vec::new(),
        })
    }

    pub fn recording(&self) -> &Recording {
        &self.recording
    }

    pub fn into_recording(self) -> Recording {
        self.recording
    }

    pub fn coordination(&self) -> &RecordingCoordination {
        &self.coordination
    }

    pub fn status(&self) -> RecordingWorkflowStatus {
        self.status
    }

    pub fn begin_ready_phase(&mut self) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Preparing {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.coordination.begin_waiting_for_ready()?;
        self.status = RecordingWorkflowStatus::WaitingForReady;
        Ok(())
    }

    pub fn mark_ready(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::WaitingForReady {
            return Err(RecordingWorkflowError::InvalidState);
        }
        let ready = self.coordination.mark_ready(participant_id)?;
        if ready {
            self.status = RecordingWorkflowStatus::Ready;
        }
        Ok(ready)
    }

    pub fn start_recording(&mut self) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Ready {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.recording.start()?;
        self.status = RecordingWorkflowStatus::Recording;
        Ok(())
    }

    pub fn request_stop(&mut self) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Recording {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.recording.stop()?;
        self.status = RecordingWorkflowStatus::Stopping;
        Ok(())
    }

    pub fn acknowledge_stop(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Stopping {
            return Err(RecordingWorkflowError::InvalidState);
        }
        if !self.coordination.participants().contains(participant_id) {
            return Err(RecordingWorkflowError::ParticipantNotSelected);
        }
        if self.acknowledged.contains(participant_id) {
            return Err(RecordingWorkflowError::AlreadyAcknowledged);
        }
        self.acknowledged.push(participant_id.clone());
        Ok(self.acknowledged.len() == self.coordination.participants().len())
    }

    pub fn complete(
        &mut self,
        artifact_id: RecordingArtifactId,
    ) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Stopping {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.recording.complete(artifact_id)?;
        self.status = RecordingWorkflowStatus::Completed;
        Ok(())
    }

    pub fn is_complete(&self) -> bool {
        self.status == RecordingWorkflowStatus::Completed
            && self.recording.status() == RecordingStatus::Completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(id: &str) -> ParticipantId {
        ParticipantId::new(id)
    }

    fn workflow() -> RecordingWorkflow {
        RecordingWorkflow::new(
            "recording-workflow-01",
            [participant("participant-a"), participant("participant-b")],
        )
        .unwrap()
    }

    // TEST-01
    #[test]
    fn workflow_requires_all_ready_before_recording() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        assert!(!workflow.mark_ready(&participant("participant-a")).unwrap());
        assert_eq!(workflow.status(), RecordingWorkflowStatus::WaitingForReady);
        assert_eq!(
            workflow.start_recording(),
            Err(RecordingWorkflowError::InvalidState)
        );
        assert!(workflow.mark_ready(&participant("participant-b")).unwrap());
        workflow.start_recording().unwrap();
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Recording);
    }

    // TEST-02
    #[test]
    fn workflow_requires_ready_phase_before_ready_reports() {
        let mut workflow = workflow();
        assert_eq!(
            workflow.mark_ready(&participant("participant-a")),
            Err(RecordingWorkflowError::InvalidState)
        );
    }

    // TEST-03
    #[test]
    fn workflow_requires_recording_before_stop() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        workflow.mark_ready(&participant("participant-a")).unwrap();
        workflow.mark_ready(&participant("participant-b")).unwrap();
        assert_eq!(
            workflow.request_stop(),
            Err(RecordingWorkflowError::InvalidState)
        );
        workflow.start_recording().unwrap();
        workflow.request_stop().unwrap();
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Stopping);
        assert_eq!(workflow.recording().status(), RecordingStatus::Stopped);
    }

    // TEST-04
    #[test]
    fn workflow_stop_acknowledgements_do_not_block_completion() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        workflow.mark_ready(&participant("participant-a")).unwrap();
        workflow.mark_ready(&participant("participant-b")).unwrap();
        workflow.start_recording().unwrap();
        workflow.request_stop().unwrap();

        workflow
            .complete(RecordingArtifactId::new("artifact-workflow-01"))
            .unwrap();
        assert!(workflow.is_complete());
    }

    // TEST-05
    #[test]
    fn workflow_rejects_unselected_stop_acknowledgement() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        workflow.mark_ready(&participant("participant-a")).unwrap();
        workflow.mark_ready(&participant("participant-b")).unwrap();
        workflow.start_recording().unwrap();
        workflow.request_stop().unwrap();
        assert_eq!(
            workflow.acknowledge_stop(&participant("participant-c")),
            Err(RecordingWorkflowError::ParticipantNotSelected)
        );
    }

    // TEST-06
    #[test]
    fn workflow_rejects_duplicate_stop_acknowledgement() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        workflow.mark_ready(&participant("participant-a")).unwrap();
        workflow.mark_ready(&participant("participant-b")).unwrap();
        workflow.start_recording().unwrap();
        workflow.request_stop().unwrap();
        workflow
            .acknowledge_stop(&participant("participant-a"))
            .unwrap();
        assert_eq!(
            workflow.acknowledge_stop(&participant("participant-a")),
            Err(RecordingWorkflowError::AlreadyAcknowledged)
        );
    }

    // TEST-07
    #[test]
    fn workflow_can_reconstitute_and_return_existing_recording_state() {
        let mut recording = Recording::new("recording-workflow-02");
        recording.assign_participant(participant("participant-a"));
        let mut workflow =
            RecordingWorkflow::from_recording(recording.clone(), [participant("participant-a")])
                .unwrap();

        workflow.begin_ready_phase().unwrap();
        assert!(workflow.mark_ready(&participant("participant-a")).unwrap());
        workflow.start_recording().unwrap();
        workflow.request_stop().unwrap();
        workflow
            .complete(RecordingArtifactId::new("artifact-workflow-02"))
            .unwrap();

        let result = workflow.into_recording();
        assert_eq!(result.id(), recording.id());
        assert_eq!(result.participant_id(), recording.participant_id());
        assert_eq!(result.status(), RecordingStatus::Completed);
        assert_eq!(
            result.artifact_id().unwrap().value(),
            "artifact-workflow-02"
        );
    }
}
