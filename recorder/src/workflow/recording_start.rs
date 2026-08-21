use std::collections::BTreeSet;

/// Identifies a participant selected for one concrete recording start.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordingParticipantId(String);

impl RecordingParticipantId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Result of a participant READY confirmation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadyStatus {
    WaitingForParticipants,
    AllParticipantsReady,
}

/// Coordinates the participant set and READY barrier for one recording start.
///
/// The participant set is frozen when the coordinator is created. Session
/// membership changes after that point do not affect this recording start.
/// The coordinator does not start or stop audio and does not emit the actual
/// audio signet; it only establishes the synchronization condition required
/// before the Opening Sync Signet may be emitted.
#[derive(Debug)]
pub struct RecordingStartCoordinator {
    participants: BTreeSet<RecordingParticipantId>,
    ready: BTreeSet<RecordingParticipantId>,
}

impl RecordingStartCoordinator {
    /// Creates a coordinator with the recording participant set for one start.
    pub fn new<I>(participants: I) -> Self
    where
        I: IntoIterator<Item = RecordingParticipantId>,
    {
        Self {
            participants: participants.into_iter().collect(),
            ready: BTreeSet::new(),
        }
    }

    /// Returns the frozen participant set for this recording start.
    pub fn participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.participants
    }

    /// Returns the participants that have confirmed local capture is active.
    pub fn ready_participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.ready
    }

    /// Confirms READY for a participant in this recording start.
    ///
    /// A participant outside the frozen set cannot satisfy the READY barrier.
    pub fn confirm_ready(
        &mut self,
        participant: &RecordingParticipantId,
    ) -> Result<ReadyStatus, RecordingStartError> {
        if !self.participants.contains(participant) {
            return Err(RecordingStartError::NotRecordingParticipant);
        }

        self.ready.insert(participant.clone());

        if self.ready.len() == self.participants.len() {
            Ok(ReadyStatus::AllParticipantsReady)
        } else {
            Ok(ReadyStatus::WaitingForParticipants)
        }
    }

    /// Returns whether every participant selected for this recording is READY.
    pub fn all_ready(&self) -> bool {
        self.ready.len() == self.participants.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStartError {
    NotRecordingParticipant,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(id: &str) -> RecordingParticipantId {
        RecordingParticipantId::new(id)
    }

    // TEST-01 / CUE30
    // Verify: The participant set is frozen for one concrete recording start.
    #[test]
    fn recording_start_freezes_participant_set() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let late_joiner = participant("participant-3");

        let coordinator = RecordingStartCoordinator::new([first.clone(), second.clone()]);

        assert_eq!(coordinator.participants().len(), 2);
        assert!(coordinator.participants().contains(&first));
        assert!(coordinator.participants().contains(&second));
        assert!(!coordinator.participants().contains(&late_joiner));
    }

    // TEST-02 / CUE30
    // Verify: The Opening Sync Signet barrier is reached only after every
    // recording participant has confirmed READY.
    #[test]
    fn opening_barrier_waits_for_all_ready_participants() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStartCoordinator::new([first.clone(), second.clone()]);

        assert_eq!(
            coordinator.confirm_ready(&first),
            Ok(ReadyStatus::WaitingForParticipants)
        );
        assert!(!coordinator.all_ready());

        assert_eq!(
            coordinator.confirm_ready(&second),
            Ok(ReadyStatus::AllParticipantsReady)
        );
        assert!(coordinator.all_ready());
    }

    // TEST-03 / CUE30
    // Verify: A session member outside the frozen recording participant set
    // cannot satisfy the READY barrier.
    #[test]
    fn non_recording_participant_cannot_confirm_ready() {
        let recording_participant = participant("participant-1");
        let non_recording_participant = participant("participant-2");
        let mut coordinator = RecordingStartCoordinator::new([recording_participant]);

        assert_eq!(
            coordinator.confirm_ready(&non_recording_participant),
            Err(RecordingStartError::NotRecordingParticipant)
        );
        assert!(!coordinator.all_ready());
    }

    // TEST-04 / CUE30
    // Verify: Repeated READY messages are idempotent and cannot distort the
    // participant barrier.
    #[test]
    fn repeated_ready_is_idempotent() {
        let participant = participant("participant-1");
        let mut coordinator = RecordingStartCoordinator::new([participant.clone()]);

        assert_eq!(
            coordinator.confirm_ready(&participant),
            Ok(ReadyStatus::AllParticipantsReady)
        );
        assert_eq!(
            coordinator.confirm_ready(&participant),
            Ok(ReadyStatus::AllParticipantsReady)
        );
        assert_eq!(coordinator.ready_participants().len(), 1);
    }
}
