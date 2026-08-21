//! Application-layer synchronization boundaries.
//!
//! Synchronization work is kept separate from the local RecordingArtifact.
//! The application boundary stores only the recoverable artifact reference,
//! synchronization state and manifest identity required to resume work.
//!
//! Concrete persistence belongs in infrastructure. Transfer semantics are
//! vendor- and transport-neutral and provide the boundary required by #144/#145.

use nc_pore_core::recording::{
    RecordingArtifactId, RecordingArtifactSynchronization, RecordingArtifactSynchronizationError,
    RecordingArtifactSynchronizationStatus,
};

/// Stable reference to one persisted local artifact version that requires synchronization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SynchronizationWork {
    artifact_id: RecordingArtifactId,
    manifest_hash: [u8; 32],
    status: RecordingArtifactSynchronizationStatus,
}

impl SynchronizationWork {
    pub fn new(artifact_id: RecordingArtifactId, manifest_hash: [u8; 32]) -> Self {
        Self {
            artifact_id,
            manifest_hash,
            status: RecordingArtifactSynchronizationStatus::Local,
        }
    }

    pub fn reconstitute(
        artifact_id: RecordingArtifactId,
        manifest_hash: [u8; 32],
        status: RecordingArtifactSynchronizationStatus,
    ) -> Self {
        Self {
            artifact_id,
            manifest_hash,
            status,
        }
    }

    pub fn artifact_id(&self) -> &RecordingArtifactId {
        &self.artifact_id
    }

    pub fn manifest_hash(&self) -> &[u8; 32] {
        &self.manifest_hash
    }

    pub fn status(&self) -> RecordingArtifactSynchronizationStatus {
        self.status
    }

    pub fn transfer_request(&self) -> ArtifactTransferRequest {
        ArtifactTransferRequest::new(self.artifact_id.clone(), self.manifest_hash)
    }

    fn lifecycle(&self) -> RecordingArtifactSynchronization {
        RecordingArtifactSynchronization::reconstitute(self.artifact_id.clone(), self.status)
    }

    fn set_status(&mut self, status: RecordingArtifactSynchronizationStatus) {
        self.status = status;
    }
}

/// Persistent store for synchronization work. It contains references, never recording payloads.
pub trait SynchronizationWorkStore {
    fn save(&mut self, work: SynchronizationWork) -> Result<(), SynchronizationWorkStoreError>;
    fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationWorkStoreError {
    Io(String),
    Serialization(String),
}

pub struct InMemorySynchronizationWorkStore {
    work: Vec<SynchronizationWork>,
}

impl InMemorySynchronizationWorkStore {
    pub fn new() -> Self {
        Self { work: Vec::new() }
    }
}

impl Default for InMemorySynchronizationWorkStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SynchronizationWorkStore for InMemorySynchronizationWorkStore {
    fn save(&mut self, work: SynchronizationWork) -> Result<(), SynchronizationWorkStoreError> {
        if let Some(existing) = self.work.iter_mut().find(|item| item.artifact_id == work.artifact_id) {
            *existing = work;
        } else {
            self.work.push(work);
        }
        Ok(())
    }

    fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError> {
        let mut work = self.work.clone();
        work.sort_by(|left, right| left.artifact_id.value().cmp(right.artifact_id.value()));
        Ok(work)
    }
}

pub struct PersistentSynchronizationQueue<S>
where
    S: SynchronizationWorkStore,
{
    store: S,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationQueueError {
    Store(SynchronizationWorkStoreError),
    Lifecycle(RecordingArtifactSynchronizationError),
    ArtifactVersionConflict { artifact_id: RecordingArtifactId },
    ArtifactNotFound { artifact_id: RecordingArtifactId },
}

impl<S> PersistentSynchronizationQueue<S>
where
    S: SynchronizationWorkStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    pub fn enqueue(
        &mut self,
        artifact_id: RecordingArtifactId,
        manifest_hash: [u8; 32],
    ) -> Result<SynchronizationWork, SynchronizationQueueError> {
        if let Some(existing) = self
            .store
            .list()
            .map_err(SynchronizationQueueError::Store)?
            .into_iter()
            .find(|work| work.artifact_id == artifact_id)
        {
            if existing.manifest_hash == manifest_hash {
                return Ok(existing);
            }
            return Err(SynchronizationQueueError::ArtifactVersionConflict { artifact_id });
        }

        let mut work = SynchronizationWork::new(artifact_id, manifest_hash);
        let mut lifecycle = work.lifecycle();
        lifecycle.queue().map_err(SynchronizationQueueError::Lifecycle)?;
        work.set_status(lifecycle.status());
        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(work)
    }

    /// Claims pending work deterministically by artifact identity.
    pub fn claim_next(&mut self) -> Result<Option<SynchronizationWork>, SynchronizationQueueError> {
        let Some(mut work) = self
            .store
            .list()
            .map_err(SynchronizationQueueError::Store)?
            .into_iter()
            .find(|work| work.status() == RecordingArtifactSynchronizationStatus::Pending)
        else {
            return Ok(None);
        };

        let mut lifecycle = work.lifecycle();
        lifecycle
            .begin_transfer()
            .map_err(SynchronizationQueueError::Lifecycle)?;
        work.set_status(lifecycle.status());
        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(Some(work))
    }

    /// Maps a completed transfer attempt back to the existing lifecycle.
    pub fn apply_transfer_result(
        &mut self,
        artifact_id: &RecordingArtifactId,
        result: &ArtifactTransferResult,
    ) -> Result<SynchronizationWork, SynchronizationQueueError> {
        let mut work = self
            .store
            .list()
            .map_err(SynchronizationQueueError::Store)?
            .into_iter()
            .find(|work| work.artifact_id() == artifact_id)
            .ok_or_else(|| SynchronizationQueueError::ArtifactNotFound {
                artifact_id: artifact_id.clone(),
            })?;

        let mut lifecycle = work.lifecycle();
        match result {
            ArtifactTransferResult::Succeeded | ArtifactTransferResult::AlreadySynchronized => {
                lifecycle
                    .mark_synchronized()
                    .map_err(SynchronizationQueueError::Lifecycle)?;
            }
            ArtifactTransferResult::RetryableFailure { .. } => {
                lifecycle
                    .retry()
                    .map_err(SynchronizationQueueError::Lifecycle)?;
            }
            ArtifactTransferResult::Conflict { .. }
            | ArtifactTransferResult::IntegrityFailure { .. }
            | ArtifactTransferResult::PermanentFailure { .. } => {
                lifecycle
                    .mark_failed()
                    .map_err(SynchronizationQueueError::Lifecycle)?;
            }
        }
        work.set_status(lifecycle.status());

        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(work)
    }

    /// Requeues in-progress work after process interruption.
    pub fn recover_interrupted(&mut self) -> Result<usize, SynchronizationQueueError> {
        let mut recovered = 0;
        for mut work in self
            .store
            .list()
            .map_err(SynchronizationQueueError::Store)?
        {
            if work.status() != RecordingArtifactSynchronizationStatus::Transferring {
                continue;
            }
            let mut lifecycle = work.lifecycle();
            lifecycle.retry().map_err(SynchronizationQueueError::Lifecycle)?;
            work.set_status(lifecycle.status());
            self.store
                .save(work)
                .map_err(SynchronizationQueueError::Store)?;
            recovered += 1;
        }
        Ok(recovered)
    }

    pub fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationQueueError> {
        self.store.list().map_err(SynchronizationQueueError::Store)
    }

    pub fn store(&self) -> &S {
        &self.store
    }
}

/// Vendor- and transport-neutral request for one artifact version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTransferRequest {
    artifact_id: RecordingArtifactId,
    manifest_hash: [u8; 32],
}

impl ArtifactTransferRequest {
    pub fn new(artifact_id: RecordingArtifactId, manifest_hash: [u8; 32]) -> Self {
        Self {
            artifact_id,
            manifest_hash,
        }
    }

    pub fn artifact_id(&self) -> &RecordingArtifactId {
        &self.artifact_id
    }

    pub fn manifest_hash(&self) -> &[u8; 32] {
        &self.manifest_hash
    }
}

/// Opaque provider continuation. The application never interprets its contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransferContinuation(Vec<u8>);

impl TransferContinuation {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

/// Transport-neutral transfer outcomes used by #144/#145.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactTransferResult {
    Succeeded,
    AlreadySynchronized,
    RetryableFailure {
        reason: String,
        continuation: Option<TransferContinuation>,
    },
    Conflict { reason: String },
    IntegrityFailure { reason: String },
    PermanentFailure { reason: String },
}

/// Boundary for future remote implementations. No vendor or protocol types cross it.
pub trait ArtifactTransfer {
    fn transfer(&mut self, request: &ArtifactTransferRequest) -> ArtifactTransferResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact_id(value: &str) -> RecordingArtifactId {
        RecordingArtifactId::new(value)
    }

    fn manifest_hash(value: u8) -> [u8; 32] {
        [value; 32]
    }

    fn claimed_queue() -> PersistentSynchronizationQueue<InMemorySynchronizationWorkStore> {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue.enqueue(artifact_id("artifact"), manifest_hash(1)).unwrap();
        queue.claim_next().unwrap();
        queue
    }

    // TEST-01
    #[test]
    fn enqueue_creates_pending_work() {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        let work = queue.enqueue(artifact_id("artifact-001"), manifest_hash(1)).unwrap();
        assert_eq!(work.status(), RecordingArtifactSynchronizationStatus::Pending);
        assert_eq!(queue.list().unwrap().len(), 1);
    }

    // TEST-02
    #[test]
    fn duplicate_enqueue_is_idempotent() {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        let first = queue.enqueue(artifact_id("artifact-002"), manifest_hash(2)).unwrap();
        let second = queue.enqueue(artifact_id("artifact-002"), manifest_hash(2)).unwrap();
        assert_eq!(first, second);
        assert_eq!(queue.list().unwrap().len(), 1);
    }

    // TEST-03
    #[test]
    fn duplicate_artifact_identity_with_different_manifest_is_rejected() {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue.enqueue(artifact_id("artifact-003"), manifest_hash(3)).unwrap();
        assert_eq!(
            queue.enqueue(artifact_id("artifact-003"), manifest_hash(4)),
            Err(SynchronizationQueueError::ArtifactVersionConflict {
                artifact_id: artifact_id("artifact-003"),
            })
        );
    }

    // TEST-04
    #[test]
    fn claim_is_deterministic() {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue.enqueue(artifact_id("artifact-b"), manifest_hash(2)).unwrap();
        queue.enqueue(artifact_id("artifact-a"), manifest_hash(1)).unwrap();
        let claimed = queue.claim_next().unwrap().unwrap();
        assert_eq!(claimed.artifact_id().value(), "artifact-a");
        assert_eq!(claimed.status(), RecordingArtifactSynchronizationStatus::Transferring);
    }

    // TEST-05
    #[test]
    fn interrupted_transfer_returns_to_pending() {
        let mut queue = claimed_queue();
        assert_eq!(queue.recover_interrupted().unwrap(), 1);
        assert_eq!(queue.list().unwrap()[0].status(), RecordingArtifactSynchronizationStatus::Pending);
    }

    // TEST-06
    #[test]
    fn transfer_request_preserves_identity() {
        let mut queue = claimed_queue();
        let work = queue.claim_next().unwrap();
        assert!(work.is_none());
        let work = queue.list().unwrap()[0].clone();
        let request = work.transfer_request();
        assert_eq!(request.artifact_id().value(), "artifact");
        assert_eq!(request.manifest_hash(), &manifest_hash(1));
    }

    // TEST-07
    #[test]
    fn successful_transfer_becomes_synchronized() {
        let mut queue = claimed_queue();
        let work = queue.apply_transfer_result(&artifact_id("artifact"), &ArtifactTransferResult::Succeeded).unwrap();
        assert_eq!(work.status(), RecordingArtifactSynchronizationStatus::Synchronized);
    }

    // TEST-08
    #[test]
    fn already_synchronized_transfer_is_idempotent() {
        let mut queue = claimed_queue();
        queue.apply_transfer_result(&artifact_id("artifact"), &ArtifactTransferResult::Succeeded).unwrap();
        let work = queue.apply_transfer_result(&artifact_id("artifact"), &ArtifactTransferResult::AlreadySynchronized).unwrap();
        assert_eq!(work.status(), RecordingArtifactSynchronizationStatus::Synchronized);
    }

    // TEST-09
    #[test]
    fn retryable_failure_returns_to_pending_and_can_carry_continuation() {
        let mut queue = claimed_queue();
        let result = ArtifactTransferResult::RetryableFailure {
            reason: "interrupted".to_owned(),
            continuation: Some(TransferContinuation::new([1_u8, 2, 3])),
        };
        let work = queue.apply_transfer_result(&artifact_id("artifact"), &result).unwrap();
        assert_eq!(work.status(), RecordingArtifactSynchronizationStatus::Pending);
    }

    // TEST-10
    #[test]
    fn conflict_integrity_and_permanent_failures_become_failed() {
        for result in [
            ArtifactTransferResult::Conflict { reason: "remote conflict".to_owned() },
            ArtifactTransferResult::IntegrityFailure { reason: "hash mismatch".to_owned() },
            ArtifactTransferResult::PermanentFailure { reason: "not retryable".to_owned() },
        ] {
            let mut queue = claimed_queue();
            let work = queue.apply_transfer_result(&artifact_id("artifact"), &result).unwrap();
            assert_eq!(work.status(), RecordingArtifactSynchronizationStatus::Failed);
        }
    }

    // TEST-11
    #[test]
    fn missing_artifact_is_reported_explicitly() {
        let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        let result = queue.apply_transfer_result(&artifact_id("missing"), &ArtifactTransferResult::Succeeded);
        assert_eq!(
            result,
            Err(SynchronizationQueueError::ArtifactNotFound {
                artifact_id: artifact_id("missing"),
            })
        );
    }

    // TEST-12
    #[test]
    fn deterministic_transfer_implementation_needs_no_vendor_types() {
        struct TestTransfer;
        impl ArtifactTransfer for TestTransfer {
            fn transfer(&mut self, request: &ArtifactTransferRequest) -> ArtifactTransferResult {
                if request.manifest_hash() == &manifest_hash(10) {
                    ArtifactTransferResult::Succeeded
                } else {
                    ArtifactTransferResult::RetryableFailure {
                        reason: "test failure".to_owned(),
                        continuation: None,
                    }
                }
            }
        }

        let mut transfer = TestTransfer;
        assert_eq!(
            transfer.transfer(&ArtifactTransferRequest::new(artifact_id("artifact-010"), manifest_hash(10))),
            ArtifactTransferResult::Succeeded
        );
    }
}
