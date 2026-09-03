use super::{
    Recording, RecordingArtifactId, RecordingCoordination, RecordingCoordinationError,
    RecordingLifecycleError, RecordingStatus, RecordingSyncSignet,
};
use crate::participant::ParticipantId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingWorkflowStatus {
    Preparing,
    WaitingForReady,
    Ready,
    Opening,
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
    OpeningNotConfirmed,
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

/// Orchestrates the domain-level ADR-068 recording sequence without owning
/// audio capture, clock handling, or sync-signet generation.
///
/// The Opening Signet is a hard synchronization barrier: READY makes the
/// workflow eligible to trigger Opening, but the workflow does not enter the
/// stable Recording state until every selected recording client has confirmed
/// that it received/captured Opening.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingWorkflow {
    recording: Recording,
    coordination: RecordingCoordination,
    status: RecordingWorkflowStatus,
    opening_confirmed: Vec<ParticipantId>,
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
            opening_confirmed: Vec::new(),
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

    pub fn start_recording_with_signet(
        &mut self,
    ) -> Result<RecordingSyncSignet, RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Ready {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.status = RecordingWorkflowStatus::Opening;
        Ok(RecordingSyncSignet::Opening)
    }

    /// Confirms Opening for one selected recording client.
    ///
    /// Stable Recording is reached only after every selected client has
    /// confirmed Opening. This is the domain barrier corresponding to the
    /// ADR-071i requirement that all recording clients confirm the Opening
    /// Signet before Recording becomes authoritative.
    pub fn confirm_opening(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Opening {
            return Err(RecordingWorkflowError::InvalidState);
        }
        if !self.coordination.participants().contains(participant_id) {
            return Err(RecordingWorkflowError::ParticipantNotSelected);
        }
        if self.opening_confirmed.contains(participant_id) {
            return Err(RecordingWorkflowError::AlreadyAcknowledged);
        }

        self.opening_confirmed.push(participant_id.clone());
        if self.opening_confirmed.len() != self.coordination.participants().len() {
            return Ok(false);
        }

        self.recording.start()?;
        self.status = RecordingWorkflowStatus::Recording;
        Ok(true)
    }

    pub fn start_recording(&mut self) -> Result<(), RecordingWorkflowError> {
        self.start_recording_with_signet().map(|_| ())
    }

    /// Validates that the workflow is at the stable Recording boundary and
    /// accepts a fachliche stop request without mutating local lifecycle state.
    pub fn request_stop(&self) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Recording
            || self.recording.status() != RecordingStatus::Recording
        {
            return Err(RecordingWorkflowError::InvalidState);
        }
        Ok(())
    }

    /// Applies the local/domain stop transition after the authoritative Core
    /// stop has been successfully persisted.
    pub fn confirm_core_stop_persisted(&mut self) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Recording
            || self.recording.status() != RecordingStatus::Recording
        {
            return Err(RecordingWorkflowError::InvalidState);
        }
        self.recording.stop()?;
        self.status = RecordingWorkflowStatus::Stopping;
        Ok(())
    }

    /// Records technical stop completion for a selected participant.
    ///
    /// Completion acknowledgements are idempotent: a duplicate confirmation
    /// from the same participant simply reports the current aggregate state.
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
            return Ok(self.acknowledged.len() == self.coordination.participants().len());
        }
        self.acknowledged.push(participant_id.clone());
        Ok(self.acknowledged.len() == self.coordination.participants().len())
    }

    pub fn complete(
        &mut self,
        artifact_id: RecordingArtifactId,
    ) -> Result<(), RecordingWorkflowError> {
        if self.status != RecordingWorkflowStatus::Stopping
            || self.acknowledged.len() != self.coordination.participants().len()
        {
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

    fn reach_opening(workflow: &mut RecordingWorkflow) {
        workflow.begin_ready_phase().unwrap();
        workflow.mark_ready(&participant("participant-a")).unwrap();
        workflow.mark_ready(&participant("participant-b")).unwrap();
    }

    fn reach_recording(workflow: &mut RecordingWorkflow) {
        reach_opening(workflow);
        workflow.start_recording_with_signet().unwrap();
        workflow
            .confirm_opening(&participant("participant-a"))
            .unwrap();
        workflow
            .confirm_opening(&participant("participant-b"))
            .unwrap();
    }

    #[test]
    fn workflow_requires_all_ready_before_opening() {
        let mut workflow = workflow();
        workflow.begin_ready_phase().unwrap();
        assert!(!workflow.mark_ready(&participant("participant-a")).unwrap());
        assert_eq!(workflow.status(), RecordingWorkflowStatus::WaitingForReady);
        assert_eq!(
            workflow.start_recording_with_signet(),
            Err(RecordingWorkflowError::InvalidState)
        );
        assert!(workflow.mark_ready(&participant("participant-b")).unwrap());
        assert_eq!(
            workflow.start_recording_with_signet(),
            Ok(RecordingSyncSignet::Opening)
        );
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Opening);
    }

    #[test]
    fn workflow_requires_ready_phase_before_ready_reports() {
        let mut workflow = workflow();
        assert_eq!(
            workflow.mark_ready(&participant("participant-a")),
            Err(RecordingWorkflowError::InvalidState)
        );
    }

    #[test]
    fn opening_must_be_confirmed_by_every_participant_before_stable_recording() {
        let mut workflow = workflow();
        reach_opening(&mut workflow);
        let signet = workflow.start_recording_with_signet().unwrap();

        assert_eq!(signet, RecordingSyncSignet::Opening);
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Opening);
        assert_eq!(
            workflow.request_stop(),
            Err(RecordingWorkflowError::InvalidState)
        );

        assert!(!workflow
            .confirm_opening(&participant("participant-a"))
            .unwrap());
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Opening);
        assert_eq!(
            workflow.confirm_opening(&participant("participant-a")),
            Err(RecordingWorkflowError::AlreadyAcknowledged)
        );

        assert!(workflow
            .confirm_opening(&participant("participant-b"))
            .unwrap());
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Recording);
    }

    #[test]
    fn opening_cannot_be_confirmed_before_it_is_triggered() {
        let mut workflow = workflow();
        reach_opening(&mut workflow);
        assert_eq!(
            workflow.confirm_opening(&participant("participant-a")),
            Err(RecordingWorkflowError::InvalidState)
        );
    }

    #[test]
    fn opening_confirmation_rejects_unselected_participant() {
        let mut workflow = workflow();
        reach_opening(&mut workflow);
        workflow.start_recording_with_signet().unwrap();
        assert_eq!(
            workflow.confirm_opening(&participant("participant-c")),
            Err(RecordingWorkflowError::ParticipantNotSelected)
        );
    }

    #[test]
    fn workflow_requires_recording_before_stop() {
        let mut workflow = workflow();
        reach_opening(&mut workflow);
        workflow.start_recording_with_signet().unwrap();
        assert_eq!(
            workflow.request_stop(),
            Err(RecordingWorkflowError::InvalidState)
        );
        workflow
            .confirm_opening(&participant("participant-a"))
            .unwrap();
        assert_eq!(
            workflow.request_stop(),
            Err(RecordingWorkflowError::InvalidState)
        );
        workflow
            .confirm_opening(&participant("participant-b"))
            .unwrap();
        workflow.request_stop().unwrap();
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Recording);
        workflow.confirm_core_stop_persisted().unwrap();
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Stopping);
        assert_eq!(workflow.recording().status(), RecordingStatus::Stopped);
    }

    #[test]
    fn workflow_requires_core_stop_persistence_before_local_stop() {
        let mut workflow = workflow();
        reach_recording(&mut workflow);
        workflow.request_stop().unwrap();

        assert_eq!(workflow.status(), RecordingWorkflowStatus::Recording);
        assert_eq!(workflow.recording().status(), RecordingStatus::Recording);

        workflow.confirm_core_stop_persisted().unwrap();
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Stopping);
        assert_eq!(workflow.recording().status(), RecordingStatus::Stopped);
    }

    #[test]
    fn workflow_requires_all_stop_acknowledgements_before_completion() {
        let mut workflow = workflow();
        reach_recording(&mut workflow);
        workflow.request_stop().unwrap();
        workflow.confirm_core_stop_persisted().unwrap();
        assert!(!workflow
            .acknowledge_stop(&participant("participant-a"))
            .unwrap());
        assert_eq!(workflow.status(), RecordingWorkflowStatus::Stopping);
        assert_eq!(
            workflow.complete(RecordingArtifactId::new("artifact-workflow-01")),
            Err(RecordingWorkflowError::InvalidState)
        );
        assert!(workflow
            .acknowledge_stop(&participant("participant-b"))
            .unwrap());
        workflow
            .complete(RecordingArtifactId::new("artifact-workflow-01"))
            .unwrap();
        assert!(workflow.is_complete());
    }

    #[test]
    fn workflow_rejects_unselected_stop_acknowledgement() {
        let mut workflow = workflow();
        reach_recording(&mut workflow);
        workflow.request_stop().unwrap();
        workflow.confirm_core_stop_persisted().unwrap();
        assert_eq!(
            workflow.acknowledge_stop(&participant("participant-c")),
            Err(RecordingWorkflowError::ParticipantNotSelected)
        );
    }

    #[test]
    fn workflow_treats_duplicate_stop_acknowledgement_as_idempotent() {
        let mut workflow = workflow();
        reach_recording(&mut workflow);
        workflow.request_stop().unwrap();
        workflow.confirm_core_stop_persisted().unwrap();

        assert!(!workflow
            .acknowledge_stop(&participant("participant-a"))
            .unwrap());
        assert!(!workflow
            .acknowledge_stop(&participant("participant-a"))
            .unwrap());
        assert!(workflow
            .acknowledge_stop(&participant("participant-b"))
            .unwrap());
        assert!(workflow
            .acknowledge_stop(&participant("participant-b"))
            .unwrap());
    }

    #[test]
    fn workflow_can_reconstitute_and_return_existing_recording_state() {
        let mut recording = Recording::new("recording-workflow-02");
        recording.assign_participant(participant("participant-a"));
        let mut workflow =
            RecordingWorkflow::from_recording(recording.clone(), [participant("participant-a")])
                .unwrap();

        workflow.begin_ready_phase().unwrap();
        assert!(workflow.mark_ready(&participant("participant-a")).unwrap());
        workflow.start_recording_with_signet().unwrap();
        assert!(workflow
            .confirm_opening(&participant("participant-a"))
            .unwrap());
        workflow.request_stop().unwrap();
        workflow.confirm_core_stop_persisted().unwrap();
        assert!(workflow
            .acknowledge_stop(&participant("participant-a"))
            .unwrap());
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
