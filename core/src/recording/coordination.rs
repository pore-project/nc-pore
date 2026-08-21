use std::collections::BTreeSet;

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
    InvalidState,
}

/// Coordinates the distributed start of one recording across a fixed set of
/// recording participants. It does not perform audio capture or emit a sync
/// signet; those belong to the technical boundary outside the domain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingCoordination {
    recording_id: RecordingId,
    participants: BTreeSet<ParticipantId>,
    ready: BTreeSet<ParticipantId>,
    status: RecordingCoordinationStatus,
}

impl RecordingCoordination {
    pub fn new(
        recording_id: RecordingId,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<Self, RecordingCoordinationError> {
        let participants: BTreeSet<_> = participants.into_iter().collect();
        if participants.is_empty() {
            return Err(RecordingCoordinationError::NoParticipants);
        }

        Ok(Self {
            recording_id,
            participants,
            ready: BTreeSet::new(),
            status: RecordingCoordinationStatus::Preparing,
        })
    }

    pub fn recording_id(&self) -> &RecordingId {
        &self.recording_id
    }

    pub fn participants(&self) -> &BTreeSet<ParticipantId> {
        &self.participants
    }

    pub fn ready_participants(&self) -> &BTreeSet<ParticipantId> {
        &self.ready
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
        if !self.ready.insert(participant_id.clone()) {
            return Err(RecordingCoordinationError::AlreadyReady);
        }

        if self.ready.len() == self.participants.len() {
            self.status = RecordingCoordinationStatus::Ready;
            return Ok(true);
        }

        Ok(false)
    }

    pub fn is_ready(&self) -> bool {
        self.status == RecordingCoordinationStatus::Ready
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(id: &str) -> ParticipantId {
        ParticipantId::new(id)
    }

    fn coordination() -> RecordingCoordination {
        RecordingCoordination::new(
            RecordingId::new("recording-coordination-01"),
            [participant("participant-a"), participant("participant-b")],
        )
        .unwrap()
    }

    // TEST-01
    #[test]
    fn recording_participant_set_is_frozen_at_creation() {
        let coordination = coordination();

        assert_eq!(coordination.participants().len(), 2);
        assert!(coordination
            .participants()
            .contains(&participant("participant-a")));
        assert!(coordination
            .participants()
            .contains(&participant("participant-b")));
    }

    // TEST-02
    #[test]
    fn empty_recording_participant_set_is_rejected() {
        assert_eq!(
            RecordingCoordination::new(RecordingId::new("recording-coordination-02"), []),
            Err(RecordingCoordinationError::NoParticipants)
        );
    }

    // TEST-03
    #[test]
    fn ready_requires_all_selected_participants() {
        let mut coordination = coordination();
        coordination.begin_waiting_for_ready().unwrap();

        assert_eq!(
            coordination.mark_ready(&participant("participant-a")),
            Ok(false)
        );
        assert_eq!(
            coordination.status(),
            RecordingCoordinationStatus::WaitingForReady
        );
        assert!(!coordination.is_ready());

        assert_eq!(
            coordination.mark_ready(&participant("participant-b")),
            Ok(true)
        );
        assert!(coordination.is_ready());
    }

    // TEST-04
    #[test]
    fn unselected_participant_cannot_report_ready() {
        let mut coordination = coordination();
        coordination.begin_waiting_for_ready().unwrap();

        assert_eq!(
            coordination.mark_ready(&participant("participant-c")),
            Err(RecordingCoordinationError::ParticipantNotSelected)
        );
    }

    // TEST-05
    #[test]
    fn participant_cannot_report_ready_twice() {
        let mut coordination = coordination();
        coordination.begin_waiting_for_ready().unwrap();
        coordination.mark_ready(&participant("participant-a")).unwrap();

        assert_eq!(
            coordination.mark_ready(&participant("participant-a")),
            Err(RecordingCoordinationError::AlreadyReady)
        );
    }

    // TEST-06
    #[test]
    fn ready_cannot_be_reported_before_start_waiting_state() {
        let mut coordination = coordination();

        assert_eq!(
            coordination.mark_ready(&participant("participant-a")),
            Err(RecordingCoordinationError::InvalidState)
        );
    }
}
