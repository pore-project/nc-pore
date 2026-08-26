//! Application-level synchronization orchestration.
//!
//! The orchestrator consumes persisted synchronization work, validates the
//! referenced local artifact before transfer, applies the vendor-neutral
//! transfer outcome to the synchronization lifecycle, and deliberately keeps
//! retryable work pending when the remote side is unavailable.
//!
//! See ADR-068 and #66 / #146.

use recorder::persistence::{PersistenceLoadResult, PersistenceProvider};

use crate::synchronization::{
    ArtifactTransfer, ArtifactTransferResult, PersistentSynchronizationQueue,
    SynchronizationQueueError, SynchronizationWork, SynchronizationWorkStore,
};

/// Result of one deterministic synchronization processing step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationProcessOutcome {
    NoPendingWork,
    Synchronized,
    Retryable { reason: String },
    Failed { reason: String },
}

/// Errors raised while preparing or executing one orchestration step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationOrchestrationError {
    Queue(SynchronizationQueueError),
    TransferContract(String),
}

/// Deterministic application service for persisted synchronization work.
///
/// One call processes at most one work item. A retryable transfer outcome is
/// persisted as `Pending` and is therefore retried by a later invocation. No
/// background worker or network-dependent recording path is introduced here.
pub struct SynchronizationOrchestrator<W, P, T>
where
    W: SynchronizationWorkStore,
    P: PersistenceProvider,
    T: ArtifactTransfer,
{
    queue: PersistentSynchronizationQueue<W>,
    persistence: P,
    transfer: T,
}

impl<W, P, T> SynchronizationOrchestrator<W, P, T>
where
    W: SynchronizationWorkStore,
    P: PersistenceProvider,
    T: ArtifactTransfer,
{
    pub fn new(queue: PersistentSynchronizationQueue<W>, persistence: P, transfer: T) -> Self {
        Self {
            queue,
            persistence,
            transfer,
        }
    }

    /// Recovers interrupted work and then processes the next pending item.
    pub fn process_next(
        &mut self,
    ) -> Result<SynchronizationProcessOutcome, SynchronizationOrchestrationError> {
        self.queue
            .recover_interrupted()
            .map_err(SynchronizationOrchestrationError::Queue)?;

        let Some(work) = self
            .queue
            .claim_next()
            .map_err(SynchronizationOrchestrationError::Queue)?
        else {
            return Ok(SynchronizationProcessOutcome::NoPendingWork);
        };

        let artifact_id = work.artifact_id().clone();
        let artifact_id_value = artifact_id.value().to_owned();

        let artifact = match self.persistence.load(&artifact_id_value) {
            PersistenceLoadResult::Valid(artifact) => artifact,
            PersistenceLoadResult::NotFound
            | PersistenceLoadResult::Incomplete
            | PersistenceLoadResult::Inconsistent => {
                let result = ArtifactTransferResult::IntegrityFailure {
                    reason: "persisted artifact is unavailable or invalid".to_owned(),
                };
                return self.apply_result(&work, &result);
            }
        };

        // The persisted work item identifies an exact artifact version. A
        // changed local representation must never be reported as synchronized.
        if artifact.manifest_hash().as_bytes() != work.manifest_hash() {
            let result = ArtifactTransferResult::IntegrityFailure {
                reason: "persisted artifact manifest does not match synchronization work"
                    .to_owned(),
            };
            return self.apply_result(&work, &result);
        }

        let result = self.transfer.transfer(&work.transfer_request());
        self.apply_result(&work, &result)
    }

    fn apply_result(
        &mut self,
        work: &SynchronizationWork,
        result: &ArtifactTransferResult,
    ) -> Result<SynchronizationProcessOutcome, SynchronizationOrchestrationError> {
        self.queue
            .apply_transfer_result(work.artifact_id(), result)
            .map_err(SynchronizationOrchestrationError::Queue)?;

        match result {
            ArtifactTransferResult::Succeeded | ArtifactTransferResult::AlreadySynchronized => {
                Ok(SynchronizationProcessOutcome::Synchronized)
            }
            ArtifactTransferResult::RetryableFailure { reason, .. } => {
                Ok(SynchronizationProcessOutcome::Retryable {
                    reason: reason.clone(),
                })
            }
            ArtifactTransferResult::Conflict { reason }
            | ArtifactTransferResult::IntegrityFailure { reason }
            | ArtifactTransferResult::PermanentFailure { reason } => {
                Ok(SynchronizationProcessOutcome::Failed {
                    reason: reason.clone(),
                })
            }
        }
    }

    pub fn queue(&self) -> &PersistentSynchronizationQueue<W> {
        &self.queue
    }

    pub fn queue_mut(&mut self) -> &mut PersistentSynchronizationQueue<W> {
        &mut self.queue
    }

    pub fn persistence(&self) -> &P {
        &self.persistence
    }

    pub fn transfer(&self) -> &T {
        &self.transfer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    use nc_pore_core::recording::RecordingArtifactId;
    use recorder::artifact::RecordingArtifact;
    use recorder::persistence::InMemoryPersistenceProvider;
    use recorder::session::RecordingSessionId;

    fn artifact(value: &str) -> RecordingArtifact {
        RecordingArtifact::new(value, RecordingSessionId::new("session-146"))
    }

    fn enqueue_artifact(
        queue: &mut PersistentSynchronizationQueue<
            crate::synchronization::InMemorySynchronizationWorkStore,
        >,
        artifact: &RecordingArtifact,
    ) {
        queue
            .enqueue(
                RecordingArtifactId::new(artifact.id.value()),
                *artifact.manifest_hash().as_bytes(),
            )
            .unwrap();
    }

    #[derive(Default)]
    struct ScriptedTransfer {
        results: VecDeque<ArtifactTransferResult>,
    }

    impl ScriptedTransfer {
        fn with_results(results: impl IntoIterator<Item = ArtifactTransferResult>) -> Self {
            Self {
                results: results.into_iter().collect(),
            }
        }
    }

    impl ArtifactTransfer for ScriptedTransfer {
        fn transfer(
            &mut self,
            _request: &crate::synchronization::ArtifactTransferRequest,
        ) -> ArtifactTransferResult {
            self.results
                .pop_front()
                .unwrap_or(ArtifactTransferResult::PermanentFailure {
                    reason: "test transfer exhausted".to_owned(),
                })
        }
    }

    // TEST-01
    #[test]
    fn persisted_pending_work_is_processed_to_synchronized() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-01");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::Succeeded]),
        );

        assert_eq!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Synchronized
        );
        assert_eq!(
            orchestrator.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
        );
    }

    // TEST-02
    #[test]
    fn offline_transfer_keeps_work_pending_and_local_artifact_available() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-02");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::RetryableFailure {
                reason: "offline".to_owned(),
                continuation: None,
            }]),
        );

        assert!(matches!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Retryable { reason } if reason == "offline"
        ));
        assert!(matches!(
            orchestrator.persistence().load("artifact-146-02"),
            PersistenceLoadResult::Valid(_)
        ));
        assert_eq!(
            orchestrator.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Pending
        );
    }

    // TEST-03
    #[test]
    fn retryable_transfer_can_be_retried_on_next_invocation() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-03");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([
                ArtifactTransferResult::RetryableFailure {
                    reason: "offline".to_owned(),
                    continuation: None,
                },
                ArtifactTransferResult::Succeeded,
            ]),
        );

        assert!(matches!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Retryable { .. }
        ));
        assert_eq!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Synchronized
        );
    }

    // TEST-04
    #[test]
    fn interrupted_processing_is_recovered_before_next_attempt() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-04");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);
        queue.claim_next().unwrap();

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::Succeeded]),
        );

        assert_eq!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Synchronized
        );
    }

    // TEST-05
    #[test]
    fn terminal_failure_does_not_remain_retryable() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-05");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::PermanentFailure {
                reason: "terminal".to_owned(),
            }]),
        );

        assert!(matches!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Failed { reason } if reason == "terminal"
        ));
        assert_eq!(
            orchestrator.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Failed
        );
    }

    // TEST-06
    #[test]
    fn manifest_mismatch_is_failed_before_transfer() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-06");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        queue
            .enqueue(RecordingArtifactId::new("artifact-146-06"), [0_u8; 32])
            .unwrap();

        let mut orchestrator = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::Succeeded]),
        );

        assert!(matches!(
            orchestrator.process_next().unwrap(),
            SynchronizationProcessOutcome::Failed { reason }
                if reason.contains("manifest")
        ));
        assert_eq!(
            orchestrator.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Failed
        );
        assert_eq!(stored.id.value(), "artifact-146-06");
    }

    // TEST-07
    #[test]
    fn retryable_work_survives_orchestrator_restart_before_recovery() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let artifact = artifact("artifact-146-07");
        let stored = persistence.store_checked(artifact.clone()).unwrap();
        let mut queue = PersistentSynchronizationQueue::new(
            crate::synchronization::InMemorySynchronizationWorkStore::new(),
        );
        enqueue_artifact(&mut queue, &stored);

        let mut first = SynchronizationOrchestrator::new(
            queue,
            persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::RetryableFailure {
                reason: "offline".to_owned(),
                continuation: None,
            }]),
        );

        assert!(matches!(
            first.process_next().unwrap(),
            SynchronizationProcessOutcome::Retryable { reason } if reason == "offline"
        ));
        assert_eq!(
            first.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Pending
        );

        let recovered_work = first.queue().list().unwrap();
        let mut recovered_store = crate::synchronization::InMemorySynchronizationWorkStore::new();
        for work in recovered_work {
            recovered_store.save(work).unwrap();
        }
        drop(first);

        let mut recovered_persistence = InMemoryPersistenceProvider::new();
        recovered_persistence.store_checked(stored).unwrap();

        let mut second = SynchronizationOrchestrator::new(
            PersistentSynchronizationQueue::new(recovered_store),
            recovered_persistence,
            ScriptedTransfer::with_results([ArtifactTransferResult::Succeeded]),
        );

        assert_eq!(
            second.process_next().unwrap(),
            SynchronizationProcessOutcome::Synchronized
        );
        assert_eq!(
            second.queue().list().unwrap()[0].status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Synchronized
        );
    }
}