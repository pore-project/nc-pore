use std::collections::BTreeSet;

use crate::audio::SyncSignet;

use super::recording_start::RecordingParticipantId;

/// Coordinates the logical stop of one concrete distributed recording.
///
/// The participant set is frozen for this stop attempt. The coordinator emits
/// the Closing Sync Signet exactly once while all affected local recorders are
/// still expected to be active. The outer transport layer can then repeat the
/// technical stop command for the participants that have not acknowledged
/// completion yet.
#[derive(Debug)]
pub struct RecordingStopCoordinator {
    participants: BTreeSet<RecordingParticipantId>,
    completed: BTreeSet<RecordingParticipantId>,
    closing_signet_emitted: bool,
}

impl RecordingStopCoordinator {
    /// Creates a stop coordinator for the fixed set of recording participants.
    pub fn new<I>(participants: I) -> Self
    where
        I: IntoIterator<Item = RecordingParticipantId>,
    {
        Self {
            participants: participants.into_iter().collect(),
            completed: BTreeSet::new(),
            closing_signet_emitted: false,
        }
    }

    /// Returns the participant set affected by this stop operation.
    pub fn participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.participants
    }

    /// Returns the participants that have technically completed local capture.
    pub fn completed_participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.completed
    }

    /// Emits the Closing Sync Signet exactly once.
    ///
    /// The outer layer must call this before sending the technical stop command
    /// to any participant, so that all affected local recorders can capture the
    /// shared logical end marker.
    pub fn closing_sync_signet(&mut self) -> Option<SyncSignet> {
        if self.closing_signet_emitted || self.participants.is_empty() {
            return None;
        }

        self.closing_signet_emitted = true;
        Some(SyncSignet::closing())
    }

    /// Confirms that one recording participant has actually finished local
    /// capture. Repeated confirmations are idempotent.
    pub fn confirm_ok(
        &mut self,
        participant: &RecordingParticipantId,
    ) -> Result<StopStatus, RecordingStopError> {
        if !self.participants.contains(participant) {
            return Err(RecordingStopError::NotRecordingParticipant);
        }

        self.completed.insert(participant.clone());

        if self.all_completed() {
            Ok(StopStatus::AllParticipantsCompleted)
        } else {
            Ok(StopStatus::WaitingForParticipants)
        }
    }

    /// Returns the participants for which the technical stop is still pending.
    pub fn pending_participants(&self) -> BTreeSet<RecordingParticipantId> {
        self.participants
            .difference(&self.completed)
            .cloned()
            .collect()
    }

    /// Returns whether every affected participant has confirmed technical stop.
    pub fn all_completed(&self) -> bool {
        self.completed.len() == self.participants.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopStatus {
    WaitingForParticipants,
    AllParticipantsCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStopError {
    NotRecordingParticipant,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn participant(id: &str) -> RecordingParticipantId {
        RecordingParticipantId::new(id)
    }

    // TEST-01 / CUE30
    // Verify: The Closing Sync Signet is emitted only once and before completion.
    #[test]
    fn closing_signet_is_emitted_once() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStopCoordinator::new([first, second]);

        assert_eq!(
            coordinator.closing_sync_signet(),
            Some(SyncSignet::closing())
        );
        assert_eq!(coordinator.closing_sync_signet(), None);
    }

    // TEST-02 / CUE30
    // Verify: Technical completion is tracked per recording participant.
    #[test]
    fn stop_waits_for_all_participant_confirmations() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStopCoordinator::new([first.clone(), second.clone()]);

        coordinator.closing_sync_signet();

        assert_eq!(
            coordinator.confirm_ok(&first),
            Ok(StopStatus::WaitingForParticipants)
        );
        assert_eq!(coordinator.pending_participants(), [second.clone()].into_iter().collect());
        assert!(!coordinator.all_completed());

        assert_eq!(
            coordinator.confirm_ok(&second),
            Ok(StopStatus::AllParticipantsCompleted)
        );
        assert!(coordinator.pending_participants().is_empty());
        assert!(coordinator.all_completed());
    }

    // TEST-03 / CUE30
    // Verify: A participant outside the frozen recording set cannot complete the stop.
    #[test]
    fn non_recording_participant_cannot_confirm_stop() {
        let recording_participant = participant("participant-1");
        let non_recording_participant = participant("participant-2");
        let mut coordinator = RecordingStopCoordinator::new([recording_participant]);

        assert_eq!(
            coordinator.confirm_ok(&non_recording_participant),
            Err(RecordingStopError::NotRecordingParticipant)
        );
        assert!(!coordinator.all_completed());
    }

    // TEST-04 / CUE30
    // Verify: Repeated technical completion confirmations are idempotent.
    #[test]
    fn repeated_ok_is_idempotent() {
        let participant = participant("participant-1");
        let mut coordinator = RecordingStopCoordinator::new([participant.clone()]);

        assert_eq!(
            coordinator.confirm_ok(&participant),
            Ok(StopStatus::AllParticipantsCompleted)
        );
        assert_eq!(
            coordinator.confirm_ok(&participant),
            Ok(StopStatus::AllParticipantsCompleted)
        );
        assert_eq!(coordinator.completed_participants().len(), 1);
    }

    // TEST-05 / CUE30
    // Verify: An empty stop participant set cannot emit a Closing Sync Signet.
    #[test]
    fn empty_stop_has_no_closing_signet() {
        let mut coordinator = RecordingStopCoordinator::new([]);

        assert_eq!(coordinator.closing_sync_signet(), None);
        assert!(coordinator.all_completed());
    }
}
