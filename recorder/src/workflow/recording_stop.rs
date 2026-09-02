use std::collections::BTreeSet;

use crate::audio::SyncSignet;

use super::recording_start::RecordingParticipantId;

/// Coordinates the logical stop of one concrete distributed recording.
///
/// The participant set is frozen for this stop attempt. The Closing Sync Signet
/// is an optional synchronization anchor. It may be emitted at most once while
/// affected local recorders are still active, but technical stop completion is
/// independent of whether the signet exists, is emitted, or is captured.
#[derive(Debug)]
pub struct RecordingStopCoordinator {
    participants: BTreeSet<RecordingParticipantId>,
    completed: BTreeSet<RecordingParticipantId>,
    closing_signet_emitted: bool,
}

impl RecordingStopCoordinator {
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

    pub fn participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.participants
    }

    pub fn completed_participants(&self) -> &BTreeSet<RecordingParticipantId> {
        &self.completed
    }

    /// Returns the optional Closing Signet at most once.
    ///
    /// Not calling this method is valid. An empty participant set also produces
    /// no signet. Neither case prevents the technical stop from completing.
    pub fn closing_sync_signet(&mut self) -> Option<SyncSignet> {
        if self.closing_signet_emitted || self.participants.is_empty() {
            return None;
        }

        self.closing_signet_emitted = true;
        Some(SyncSignet::closing())
    }

    /// Confirms local technical completion for one participant.
    ///
    /// Confirmation is independent of Closing Signet delivery. A participant
    /// may therefore confirm completion even when it did not hear/capture the
    /// Closing Signet. Repeated confirmations remain idempotent.
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

    pub fn pending_participants(&self) -> BTreeSet<RecordingParticipantId> {
        self.participants
            .difference(&self.completed)
            .cloned()
            .collect()
    }

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
        assert_eq!(
            coordinator.pending_participants(),
            [second.clone()].into_iter().collect()
        );
        assert!(!coordinator.all_completed());
        assert_eq!(
            coordinator.confirm_ok(&second),
            Ok(StopStatus::AllParticipantsCompleted)
        );
        assert!(coordinator.pending_participants().is_empty());
    }

    #[test]
    fn stop_completes_without_closing_signet() {
        let first = participant("participant-1");
        let second = participant("participant-2");
        let mut coordinator = RecordingStopCoordinator::new([first.clone(), second.clone()]);

        assert_eq!(
            coordinator.confirm_ok(&first),
            Ok(StopStatus::WaitingForParticipants)
        );
        assert_eq!(
            coordinator.confirm_ok(&second),
            Ok(StopStatus::AllParticipantsCompleted)
        );
        assert!(coordinator.all_completed());
    }

    #[test]
    fn non_recording_participant_cannot_confirm_stop() {
        let recording_participant = participant("participant-1");
        let outsider = participant("participant-2");
        let mut coordinator = RecordingStopCoordinator::new([recording_participant]);

        assert_eq!(
            coordinator.confirm_ok(&outsider),
            Err(RecordingStopError::NotRecordingParticipant)
        );
        assert!(!coordinator.all_completed());
    }

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

    #[test]
    fn empty_stop_has_no_closing_signet() {
        let mut coordinator = RecordingStopCoordinator::new([]);
        assert_eq!(coordinator.closing_sync_signet(), None);
        assert!(coordinator.all_completed());
    }
}
