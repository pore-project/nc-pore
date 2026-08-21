use super::RecordingArtifactId;

/// Synchronization lifecycle of a locally completed recording artifact.
///
/// This state machine is deliberately independent of any transport, remote
/// storage vendor, authentication mechanism, or UI concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingArtifactSynchronizationStatus {
    Local,
    Pending,
    Transferring,
    Synchronized,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingArtifactSynchronizationError {
    InvalidTransition {
        from: RecordingArtifactSynchronizationStatus,
        to: RecordingArtifactSynchronizationStatus,
    },
    RetryNotAllowed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingArtifactSynchronization {
    artifact_id: RecordingArtifactId,
    status: RecordingArtifactSynchronizationStatus,
}

impl RecordingArtifactSynchronization {
    /// A synchronization lifecycle always starts from an already persisted
    /// local artifact. The artifact itself remains locally available in every
    /// synchronization state.
    pub fn new(artifact_id: RecordingArtifactId) -> Self {
        Self {
            artifact_id,
            status: RecordingArtifactSynchronizationStatus::Local,
        }
    }

    /// Reconstitutes persisted synchronization state without performing any
    /// transport operation.
    pub fn reconstitute(
        artifact_id: RecordingArtifactId,
        status: RecordingArtifactSynchronizationStatus,
    ) -> Self {
        Self {
            artifact_id,
            status,
        }
    }

    pub fn artifact_id(&self) -> &RecordingArtifactId {
        &self.artifact_id
    }

    pub fn status(&self) -> RecordingArtifactSynchronizationStatus {
        self.status
    }

    pub fn queue(&mut self) -> Result<(), RecordingArtifactSynchronizationError> {
        self.transition_to(RecordingArtifactSynchronizationStatus::Pending)
    }

    pub fn begin_transfer(&mut self) -> Result<(), RecordingArtifactSynchronizationError> {
        self.transition_to(RecordingArtifactSynchronizationStatus::Transferring)
    }

    /// Marks the artifact as synchronized. Repeating the same completion is
    /// idempotent for this artifact identity; transfer conflicts are handled by
    /// the transfer boundary defined in the subsequent synchronization work.
    pub fn mark_synchronized(&mut self) -> Result<(), RecordingArtifactSynchronizationError> {
        if self.status == RecordingArtifactSynchronizationStatus::Synchronized {
            return Ok(());
        }
        self.transition_to(RecordingArtifactSynchronizationStatus::Synchronized)
    }

    pub fn mark_failed(&mut self) -> Result<(), RecordingArtifactSynchronizationError> {
        self.transition_to(RecordingArtifactSynchronizationStatus::Failed)
    }

    /// An interrupted transfer is recoverable without changing the local
    /// artifact. Returning to Pending makes restart semantics deterministic.
    pub fn retry(&mut self) -> Result<(), RecordingArtifactSynchronizationError> {
        match self.status {
            RecordingArtifactSynchronizationStatus::Failed
            | RecordingArtifactSynchronizationStatus::Transferring => {
                self.status = RecordingArtifactSynchronizationStatus::Pending;
                Ok(())
            }
            _ => Err(RecordingArtifactSynchronizationError::RetryNotAllowed),
        }
    }

    fn transition_to(
        &mut self,
        target: RecordingArtifactSynchronizationStatus,
    ) -> Result<(), RecordingArtifactSynchronizationError> {
        if matches!(
            (self.status, target),
            (
                RecordingArtifactSynchronizationStatus::Local,
                RecordingArtifactSynchronizationStatus::Pending
            ) | (
                RecordingArtifactSynchronizationStatus::Pending,
                RecordingArtifactSynchronizationStatus::Transferring
            ) | (
                RecordingArtifactSynchronizationStatus::Transferring,
                RecordingArtifactSynchronizationStatus::Synchronized
            ) | (
                RecordingArtifactSynchronizationStatus::Transferring,
                RecordingArtifactSynchronizationStatus::Failed
            )
        ) {
            self.status = target;
            return Ok(());
        }

        Err(RecordingArtifactSynchronizationError::InvalidTransition {
            from: self.status,
            to: target,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact_id() -> RecordingArtifactId {
        RecordingArtifactId::new("artifact-sync-test-01")
    }

    fn synchronization() -> RecordingArtifactSynchronization {
        RecordingArtifactSynchronization::new(artifact_id())
    }

    // TEST-01
    #[test]
    fn starts_local_with_artifact_identity() {
        let synchronization = synchronization();
        assert_eq!(
            synchronization.status(),
            RecordingArtifactSynchronizationStatus::Local
        );
        assert_eq!(
            synchronization.artifact_id().value(),
            "artifact-sync-test-01"
        );
    }

    // TEST-02
    #[test]
    fn follows_local_pending_transferring_synchronized_lifecycle() {
        let mut synchronization = synchronization();
        synchronization.queue().unwrap();
        synchronization.begin_transfer().unwrap();
        synchronization.mark_synchronized().unwrap();
        assert_eq!(
            synchronization.status(),
            RecordingArtifactSynchronizationStatus::Synchronized
        );
    }

    // TEST-03
    #[test]
    fn rejects_skipping_pending_and_transfer() {
        let mut synchronization = synchronization();
        assert_eq!(
            synchronization.mark_synchronized(),
            Err(RecordingArtifactSynchronizationError::InvalidTransition {
                from: RecordingArtifactSynchronizationStatus::Local,
                to: RecordingArtifactSynchronizationStatus::Synchronized,
            })
        );
    }

    // TEST-04
    #[test]
    fn failed_transfer_can_be_retried_without_changing_artifact_identity() {
        let mut synchronization = synchronization();
        let artifact = synchronization.artifact_id().clone();
        synchronization.queue().unwrap();
        synchronization.begin_transfer().unwrap();
        synchronization.mark_failed().unwrap();
        synchronization.retry().unwrap();
        assert_eq!(
            synchronization.status(),
            RecordingArtifactSynchronizationStatus::Pending
        );
        assert_eq!(synchronization.artifact_id(), &artifact);
    }

    // TEST-05
    #[test]
    fn interrupted_transfer_can_be_restarted() {
        let mut synchronization = synchronization();
        synchronization.queue().unwrap();
        synchronization.begin_transfer().unwrap();
        synchronization.retry().unwrap();
        synchronization.begin_transfer().unwrap();
        synchronization.mark_synchronized().unwrap();
        assert_eq!(
            synchronization.status(),
            RecordingArtifactSynchronizationStatus::Synchronized
        );
    }

    // TEST-06
    #[test]
    fn synchronized_completion_is_idempotent() {
        let mut synchronization = synchronization();
        synchronization.queue().unwrap();
        synchronization.begin_transfer().unwrap();
        synchronization.mark_synchronized().unwrap();
        assert_eq!(synchronization.mark_synchronized(), Ok(()));
    }

    // TEST-07
    #[test]
    fn invalid_transitions_are_rejected() {
        let mut synchronization = synchronization();
        synchronization.queue().unwrap();
        assert_eq!(
            synchronization.queue(),
            Err(RecordingArtifactSynchronizationError::InvalidTransition {
                from: RecordingArtifactSynchronizationStatus::Pending,
                to: RecordingArtifactSynchronizationStatus::Pending,
            })
        );
        assert_eq!(
            synchronization.retry(),
            Err(RecordingArtifactSynchronizationError::RetryNotAllowed)
        );
    }

    // TEST-08
    #[test]
    fn persisted_state_can_be_reconstituted() {
        let synchronization = RecordingArtifactSynchronization::reconstitute(
            artifact_id(),
            RecordingArtifactSynchronizationStatus::Transferring,
        );
        assert_eq!(
            synchronization.status(),
            RecordingArtifactSynchronizationStatus::Transferring
        );
        assert_eq!(
            synchronization.artifact_id().value(),
            "artifact-sync-test-01"
        );
    }
}
