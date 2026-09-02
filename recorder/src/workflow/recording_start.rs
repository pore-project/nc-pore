use std::collections::BTreeSet;

use crate::audio::SyncSignet;

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
/// The coordinator does not start or stop audio; it triggers the Opening Sync
/// Signet exactly once after the READY barrier has been reached.
#[derive(Debug)]
pub struct RecordingStartCoordinator {
    participants: BTreeSet<RecordingParticipantId>,
    ready: BTreeSet<RecordingParticipantId>,
    opening_signet_emitted: bool,
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
            opening_signet_emitted: false,
        }
    }

    pub fn participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.participants
    }

    pub fn ready_participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.ready
    }

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

    /// Confirms READY and returns the configured Opening Sync Signet only when
    /// the complete barrier has been reached.
    pub fn confirm_ready_and_opening_signet(
        &mut self,
        participant: &RecordingParticipantId,
    ) -> Result<Option<SyncSignet>, RecordingStartError> {
        self.confirm_ready(participant)?;
        Ok(self.opening_sync_signet())
    }

    /// Returns the default Opening Sync Signet once the READY barrier is reached.
    ///
    /// Kept as a convenience for lower-level callers. Higher-level recording
    /// configuration should use `opening_sync_signet_with`.
    pub fn opening_sync_signet(&mut self) -> Option<SyncSignet> {
        self.opening_sync_signet_with(SyncSignet::opening())
    }

    /// Returns the supplied Opening Sync Signet once the READY barrier is reached.
    ///
    /// The coordinator owns only the lifecycle trigger and exactly-once rule;
    /// the supplied signet contains the configurable technical description.
    pub fn opening_sync_signet_with(&mut self, signet: SyncSignet) -> Option<SyncSignet> {
        if !self.all_ready() || self.opening_signet_emitted {
            return None;
        }

        self.opening_signet_emitted = true;
        Some(signet)
    }

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

    // TEST-05 / CUE30
    #[test]
    fn opening_signet_waits_for_ready_barrier() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStartCoordinator::new([first.clone(), second.clone()]);

        coordinator.confirm_ready(&first).unwrap();
        assert_eq!(coordinator.opening_sync_signet(), None);
        coordinator.confirm_ready(&second).unwrap();
        assert_eq!(
            coordinator.opening_sync_signet(),
            Some(SyncSignet::opening())
        );
    }

    // TEST-06 / CUE30
    #[test]
    fn opening_signet_is_emitted_only_once() {
        let participant = participant("participant-1");
        let mut coordinator = RecordingStartCoordinator::new([participant.clone()]);

        coordinator.confirm_ready(&participant).unwrap();
        assert_eq!(
            coordinator.opening_sync_signet(),
            Some(SyncSignet::opening())
        );
        assert_eq!(coordinator.opening_sync_signet(), None);
    }

    // TEST-07 / CUE30
    #[test]
    fn non_recording_participant_cannot_trigger_opening_signet() {
        let recording_participant = participant("participant-1");
        let non_recording_participant = participant("participant-2");
        let mut coordinator = RecordingStartCoordinator::new([recording_participant]);

        assert_eq!(
            coordinator.confirm_ready(&non_recording_participant),
            Err(RecordingStartError::NotRecordingParticipant)
        );
        assert_eq!(coordinator.opening_sync_signet(), None);
    }

    // TEST-08 / CUE30
    #[test]
    fn combined_ready_transition_opens_only_at_barrier() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStartCoordinator::new([first.clone(), second.clone()]);

        assert_eq!(
            coordinator.confirm_ready_and_opening_signet(&first),
            Ok(None)
        );
        assert_eq!(
            coordinator.confirm_ready_and_opening_signet(&second),
            Ok(Some(SyncSignet::opening()))
        );
        assert_eq!(
            coordinator.confirm_ready_and_opening_signet(&second),
            Ok(None)
        );
    }

    // TEST-09 / CUE30
    #[test]
    fn configured_opening_signet_is_returned_unchanged() {
        let participant = participant("participant-1");
        let mut coordinator = RecordingStartCoordinator::new([participant.clone()]);
        let configured = SyncSignet::opening();

        coordinator.confirm_ready(&participant).unwrap();

        assert_eq!(
            coordinator.opening_sync_signet_with(configured),
            Some(configured)
        );
    }
}
