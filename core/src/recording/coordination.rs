use crate::participant::ParticipantId;

use super::RecordingId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingCoordinationStatus {
    Preparing,
    WaitingForReady,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingCoordinationError {
    NoParticipants,
    ParticipantNotSelected,
    AlreadyReady,
    AlreadyOpeningConfirmed,
    AlreadyStopAcknowledged,
    InvalidState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingCoordination {
    recording_id: RecordingId,
    participants: Vec<ParticipantId>,
    ready: Vec<ParticipantId>,
    opening_confirmed: Vec<ParticipantId>,
    stop_acknowledged: Vec<ParticipantId>,
    status: RecordingCoordinationStatus,
}

impl RecordingCoordination {
    pub fn new(
        recording_id: RecordingId,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<Self, RecordingCoordinationError> {
        let participants: Vec<_> = participants.into_iter().collect();
        if participants.is_empty() {
            return Err(RecordingCoordinationError::NoParticipants);
        }
        if participants.iter().enumerate().any(|(index, participant)| {
            participants[..index].contains(participant)
        }) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }

        Ok(Self {
            recording_id,
            participants,
            ready: Vec::new(),
            opening_confirmed: Vec::new(),
            stop_acknowledged: Vec::new(),
            status: RecordingCoordinationStatus::Preparing,
        })
    }

    pub fn reconstitute(
        recording_id: RecordingId,
        participants: Vec<ParticipantId>,
        ready: Vec<ParticipantId>,
        opening_confirmed: Vec<ParticipantId>,
        stop_acknowledged: Vec<ParticipantId>,
        status: RecordingCoordinationStatus,
    ) -> Self {
        Self {
            recording_id,
            participants,
            ready,
            opening_confirmed,
            stop_acknowledged,
            status,
        }
    }

    pub fn recording_id(&self) -> &RecordingId {
        &self.recording_id
    }

    pub fn participants(&self) -> &[ParticipantId] {
        &self.participants
    }

    pub fn ready_participants(&self) -> &[ParticipantId] {
        &self.ready
    }

    pub fn opening_confirmed_participants(&self) -> &[ParticipantId] {
        &self.opening_confirmed
    }

    pub fn stop_acknowledged_participants(&self) -> &[ParticipantId] {
        &self.stop_acknowledged
    }

    pub fn status(&self) -> RecordingCoordinationStatus {
        self.status
    }

    pub fn begin_waiting_for_ready(&mut self) -> Result<(), RecordingCoordinationError> {
        if self.status != RecordingCoordinationStatus::Preparing {
            return Err(RecordingCoordinationError::InvalidState);
        }
        self.status = RecordingCoordinationStatus::WaitingForReady;
        Ok(())
    }

    pub fn mark_ready(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingCoordinationError> {
        if self.status != RecordingCoordinationStatus::WaitingForReady {
            return Err(RecordingCoordinationError::InvalidState);
        }
        if !self.participants.contains(participant_id) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }
        if self.ready.contains(participant_id) {
            return Err(RecordingCoordinationError::AlreadyReady);
        }
        self.ready.push(participant_id.clone());

        if self.ready.len() == self.participants.len() {
            self.status = RecordingCoordinationStatus::Ready;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn confirm_opening(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<(), RecordingCoordinationError> {
        if !self.participants.contains(participant_id) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }
        if self.opening_confirmed.contains(participant_id) {
            return Err(RecordingCoordinationError::AlreadyOpeningConfirmed);
        }
        self.opening_confirmed.push(participant_id.clone());
        Ok(())
    }

    pub fn acknowledge_stop(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<(), RecordingCoordinationError> {
        if !self.participants.contains(participant_id) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }
        if self.stop_acknowledged.contains(participant_id) {
            return Err(RecordingCoordinationError::AlreadyStopAcknowledged);
        }
        self.stop_acknowledged.push(participant_id.clone());
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.status == RecordingCoordinationStatus::Ready
    }
}
