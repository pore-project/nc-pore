use super::RecordingLifecycleError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStopMode {
    Host,
    Safety,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingClosingOutcome {
    Emitted,
    Missed,
    Unavailable,
    ParticipantGone,
    NotAttempted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStopCoordinatorStatus {
    Recording,
    CoreStopPersisted,
    ClosingAttempted,
    TechnicalStopping,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingStopCoordinatorError {
    InvalidState,
    ClosingAlreadyAttempted,
    ClosingNotAllowed,
    SafetyStopHasNoClosing,
    Lifecycle(RecordingLifecycleError),
}

/// Coordinates the host-neutral stop boundary of one recording.
///
/// The coordinator models the ordering between the authoritative Core stop,
/// the optional Closing signet, technical capture stop, and completion. It
/// deliberately does not emit audio or persist a session itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordingStopCoordinator {
    status: RecordingStopCoordinatorStatus,
    mode: RecordingStopMode,
    closing: Option<RecordingClosingOutcome>,
}

impl RecordingStopCoordinator {
    pub fn new(mode: RecordingStopMode) -> Self {
        Self {
            status: RecordingStopCoordinatorStatus::Recording,
            mode,
            closing: None,
        }
    }

    pub fn status(&self) -> RecordingStopCoordinatorStatus {
        self.status
    }

    pub fn mode(&self) -> RecordingStopMode {
        self.mode
    }

    pub fn closing(&self) -> Option<RecordingClosingOutcome> {
        self.closing
    }

    /// Marks the persisted fachliche Core stop. This is the authoritative
    /// recording boundary and must precede any optional Closing attempt.
    pub fn persist_core_stop(&mut self) -> Result<(), RecordingStopCoordinatorError> {
        if self.mode != RecordingStopMode::Host
            || self.status != RecordingStopCoordinatorStatus::Recording
        {
            return Err(RecordingStopCoordinatorError::InvalidState);
        }
        self.status = RecordingStopCoordinatorStatus::CoreStopPersisted;
        Ok(())
    }

    /// Records the result of the optional Closing attempt. Every outcome is
    /// terminal for the Closing phase; none may block technical stop.
    pub fn record_closing_outcome(
        &mut self,
        outcome: RecordingClosingOutcome,
    ) -> Result<(), RecordingStopCoordinatorError> {
        if self.mode != RecordingStopMode::Host
            || self.status != RecordingStopCoordinatorStatus::CoreStopPersisted
        {
            return Err(RecordingStopCoordinatorError::ClosingNotAllowed);
        }
        if self.closing.is_some() {
            return Err(RecordingStopCoordinatorError::ClosingAlreadyAttempted);
        }
        self.closing = Some(outcome);
        self.status = RecordingStopCoordinatorStatus::ClosingAttempted;
        Ok(())
    }

    /// Advances to technical capture stop. Closing is never required to reach
    /// this state, so callers may explicitly use `NotAttempted` for a missing
    /// or already-gone participant.
    pub fn begin_technical_stop(&mut self) -> Result<(), RecordingStopCoordinatorError> {
        match self.mode {
            RecordingStopMode::Host
                if matches!(
                    self.status,
                    RecordingStopCoordinatorStatus::CoreStopPersisted
                        | RecordingStopCoordinatorStatus::ClosingAttempted
                ) =>
            {
                self.status = RecordingStopCoordinatorStatus::TechnicalStopping;
                Ok(())
            }
            RecordingStopMode::Safety
                if self.status == RecordingStopCoordinatorStatus::Recording =>
            {
                self.status = RecordingStopCoordinatorStatus::TechnicalStopping;
                Ok(())
            }
            _ => Err(RecordingStopCoordinatorError::InvalidState),
        }
    }

    /// A reconnecting recorder can observe the persisted Core stop and enter
    /// technical stop directly. No Closing is replayed in this path.
    pub fn observe_persisted_stop_on_reconnect(
        &mut self,
    ) -> Result<(), RecordingStopCoordinatorError> {
        if self.status != RecordingStopCoordinatorStatus::Recording {
            return Err(RecordingStopCoordinatorError::InvalidState);
        }
        self.status = RecordingStopCoordinatorStatus::TechnicalStopping;
        Ok(())
    }

    /// Safety Stop is intentionally Closing-free.
    pub fn safety_stop(&mut self) -> Result<(), RecordingStopCoordinatorError> {
        if self.mode != RecordingStopMode::Safety
            || self.status != RecordingStopCoordinatorStatus::Recording
        {
            return Err(RecordingStopCoordinatorError::InvalidState);
        }
        if self.closing.is_some() {
            return Err(RecordingStopCoordinatorError::SafetyStopHasNoClosing);
        }
        self.status = RecordingStopCoordinatorStatus::TechnicalStopping;
        Ok(())
    }

    pub fn complete(&mut self) -> Result<(), RecordingStopCoordinatorError> {
        if self.status != RecordingStopCoordinatorStatus::TechnicalStopping {
            return Err(RecordingStopCoordinatorError::InvalidState);
        }
        self.status = RecordingStopCoordinatorStatus::Completed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_stop_persists_core_stop_before_closing() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);

        assert_eq!(
            coordinator.record_closing_outcome(RecordingClosingOutcome::Emitted),
            Err(RecordingStopCoordinatorError::ClosingNotAllowed)
        );
        coordinator.persist_core_stop().unwrap();
        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::CoreStopPersisted
        );
    }

    #[test]
    fn closing_is_not_a_barrier_to_technical_stop() {
        for outcome in [
            RecordingClosingOutcome::Emitted,
            RecordingClosingOutcome::Missed,
            RecordingClosingOutcome::Unavailable,
            RecordingClosingOutcome::ParticipantGone,
            RecordingClosingOutcome::NotAttempted,
        ] {
            let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
            coordinator.persist_core_stop().unwrap();
            coordinator.record_closing_outcome(outcome).unwrap();
            coordinator.begin_technical_stop().unwrap();
            coordinator.complete().unwrap();
            assert_eq!(
                coordinator.status(),
                RecordingStopCoordinatorStatus::Completed
            );
        }
    }

    #[test]
    fn closing_cannot_be_replayed() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        coordinator.persist_core_stop().unwrap();
        coordinator
            .record_closing_outcome(RecordingClosingOutcome::ParticipantGone)
            .unwrap();

        assert_eq!(
            coordinator.record_closing_outcome(RecordingClosingOutcome::Emitted),
            Err(RecordingStopCoordinatorError::ClosingNotAllowed)
        );
    }

    #[test]
    fn reconnect_after_persisted_stop_skips_closing() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        coordinator.observe_persisted_stop_on_reconnect().unwrap();

        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::TechnicalStopping
        );
        assert_eq!(coordinator.closing(), None);
        assert_eq!(
            coordinator.record_closing_outcome(RecordingClosingOutcome::Emitted),
            Err(RecordingStopCoordinatorError::ClosingNotAllowed)
        );
    }

    #[test]
    fn safety_stop_has_no_closing_path() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Safety);
        coordinator.safety_stop().unwrap();
        assert_eq!(coordinator.closing(), None);
        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::TechnicalStopping
        );
        coordinator.complete().unwrap();
    }

    #[test]
    fn safety_stop_cannot_record_closing() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Safety);
        assert_eq!(
            coordinator.record_closing_outcome(RecordingClosingOutcome::Emitted),
            Err(RecordingStopCoordinatorError::ClosingNotAllowed)
        );
    }

    #[test]
    fn technical_stop_can_follow_persisted_stop_without_closing() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        coordinator.persist_core_stop().unwrap();
        coordinator.begin_technical_stop().unwrap();
        assert_eq!(
            coordinator.status(),
            RecordingStopCoordinatorStatus::TechnicalStopping
        );
    }

    #[test]
    fn completion_requires_technical_stop() {
        let mut coordinator = RecordingStopCoordinator::new(RecordingStopMode::Host);
        coordinator.persist_core_stop().unwrap();
        assert_eq!(
            coordinator.complete(),
            Err(RecordingStopCoordinatorError::InvalidState)
        );
    }
}
