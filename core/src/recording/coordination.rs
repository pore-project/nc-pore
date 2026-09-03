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

/// Coordinates the distributed start and stop barriers of one recording
/// across a fixed set of recording participants. It does not perform audio
/// capture or emit a sync signet; those belong to the technical boundary
/// outside the domain.
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
        if participants
            .iter()
            .enumerate()
            .any(|(index, participant)| participants[..index].contains(participant))
        {
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

    /// Records the persisted Opening confirmation of one selected client.
    /// Opening is only confirmable after the complete READY barrier.
    pub fn confirm_opening(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingCoordinationError> {
        if self.status != RecordingCoordinationStatus::Ready {
            return Err(RecordingCoordinationError::InvalidState);
        }
        if !self.participants.contains(participant_id) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }
        if self.opening_confirmed.contains(participant_id) {
            return Err(RecordingCoordinationError::AlreadyOpeningConfirmed);
        }
        self.opening_confirmed.push(participant_id.clone());
        Ok(self.opening_confirmed.len() == self.participants.len())
    }

    /// Records the persisted technical stop acknowledgement of one selected
    /// client. The Opening barrier must have completed before stopping can be
    /// acknowledged.
    pub fn acknowledge_stop(
        &mut self,
        participant_id: &ParticipantId,
    ) -> Result<bool, RecordingCoordinationError> {
        if self.opening_confirmed.len() != self.participants.len() {
            return Err(RecordingCoordinationError::InvalidState);
        }
        if !self.participants.contains(participant_id) {
            return Err(RecordingCoordinationError::ParticipantNotSelected);
        }
        if self.stop_acknowledged.contains(participant_id) {
            return Ok(self.stop_acknowledged.len() == self.participants.len());
        }
        self.stop_acknowledged.push(participant_id.clone());
        Ok(self.stop_acknowledged.len() == self.participants.len())
    }

    pub fn is_ready(&self) -> bool {
        self.status == RecordingCoordinationStatus::Ready
    }

    pub fn is_opening_confirmed(&self) -> bool {
        self.opening_confirmed.len() == self.participants.len()
    }

    pub fn is_stop_acknowledged(&self) -> bool {
        self.stop_acknowledged.len() == self.participants.len()
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

    fn reach_ready(coordination: &mut RecordingCoordination) {
        coordination.begin_waiting_for_ready().unwrap();
        coordination
            .mark_ready(&participant("participant-a"))
            .unwrap();
        coordination
            .mark_ready(&participant("participant-b"))
            .unwrap();
    }

    fn reach_opening_confirmed(coordination: &mut RecordingCoordination) {
        reach_ready(coordination);
        coordination
            .confirm_opening(&participant("participant-a"))
            .unwrap();
        coordination
            .confirm_opening(&participant("participant-b"))
            .unwrap();
    }

    // TEST-01
    #[test]
    fn recording_participant_set_is_frozen_at_creation() {
        let coordination = coordination();

        assert_eq!(coordination.participants().len(), 2);
        assert!(
            coordination
                .participants()
                .contains(&participant("participant-a"))
        );
        assert!(
            coordination
                .participants()
                .contains(&participant("participant-b"))
        );
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
        coordination
            .mark_ready(&participant("participant-a"))
            .unwrap();

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

    // TEST-07
    #[test]
    fn duplicate_recording_participant_is_rejected() {
        assert_eq!(
            RecordingCoordination::new(
                RecordingId::new("recording-coordination-03"),
                [participant("participant-a"), participant("participant-a")],
            ),
            Err(RecordingCoordinationError::ParticipantNotSelected)
        );
    }

    // TEST-08
    #[test]
    fn opening_confirmation_is_persisted_and_requires_all_participants() {
        let mut coordination = coordination();
        reach_ready(&mut coordination);

        assert!(
            !coordination
                .confirm_opening(&participant("participant-a"))
                .unwrap()
        );
        assert_eq!(
            coordination.opening_confirmed_participants(),
            &[participant("participant-a")]
        );
        assert!(!coordination.is_opening_confirmed());

        assert!(
            coordination
                .confirm_opening(&participant("participant-b"))
                .unwrap()
        );
        assert!(coordination.is_opening_confirmed());
    }

    // TEST-09
    #[test]
    fn opening_confirmation_cannot_precede_ready_barrier() {
        let mut coordination = coordination();

        assert_eq!(
            coordination.confirm_opening(&participant("participant-a")),
            Err(RecordingCoordinationError::InvalidState)
        );
    }

    // TEST-10
    #[test]
    fn duplicate_opening_confirmation_is_rejected() {
        let mut coordination = coordination();
        reach_ready(&mut coordination);
        coordination
            .confirm_opening(&participant("participant-a"))
            .unwrap();

        assert_eq!(
            coordination.confirm_opening(&participant("participant-a")),
            Err(RecordingCoordinationError::AlreadyOpeningConfirmed)
        );
    }

    // TEST-11
    #[test]
    fn stop_acknowledgement_requires_opening_and_allows_idempotent_repeats() {
        let mut coordination = coordination();
        reach_opening_confirmed(&mut coordination);

        assert!(
            !coordination
                .acknowledge_stop(&participant("participant-a"))
                .unwrap()
        );
        assert!(!coordination.is_stop_acknowledged());
        assert!(
            !coordination
                .acknowledge_stop(&participant("participant-a"))
                .unwrap()
        );
        assert!(
            coordination
                .acknowledge_stop(&participant("participant-b"))
                .unwrap()
        );
        assert!(coordination.is_stop_acknowledged());
        assert!(
            coordination
                .acknowledge_stop(&participant("participant-b"))
                .unwrap()
        );
    }

    // TEST-12
    #[test]
    fn stop_acknowledgement_rejects_unselected_participant() {
        let mut coordination = coordination();
        reach_opening_confirmed(&mut coordination);

        assert_eq!(
            coordination.acknowledge_stop(&participant("participant-c")),
            Err(RecordingCoordinationError::ParticipantNotSelected)
        );
    }
}
