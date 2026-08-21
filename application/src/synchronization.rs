//! Application-layer synchronization boundaries.
//!
//! Synchronization work is kept separate from the local RecordingArtifact.
//! The application boundary stores only the recoverable artifact reference,
//! synchronization state and manifest identity required to resume work.
//!
//! Concrete persistence belongs in the infrastructure layer. The transfer
//! contract is deliberately vendor- and transport-neutral and defines the
//! correctness boundary required by #144/#145 without committing the
//! application to a concrete remote provider.
//!
//! See ADR-068 and #66 / #143 / #144 / #145.

use nc_pore_core::recording::{
    RecordingArtifactId, RecordingArtifactSynchronization, RecordingArtifactSynchronizationError,
    RecordingArtifactSynchronizationStatus,
};

/// Stable reference to one persisted local artifact version that requires
/// synchronization.
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

    /// Reconstitutes persisted work without performing any transport action.
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

    fn from_lifecycle(&mut self, lifecycle: RecordingArtifactSynchronization) {
        self.status = lifecycle.status();
    }

    fn apply_transfer_result(
        &mut self,
        result: &ArtifactTransferResult,
    ) -> Result<(), SynchronizationQueueError> {
        let mut lifecycle = self.lifecycle();

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

        self.from_lifecycle(lifecycle);
        Ok(())
    }
}

/// Persistent store for synchronization work.
///
/// The store contains work references, never recording payload data. Concrete
/// implementations belong outside the application layer.
pub trait SynchronizationWorkStore {
    fn save(&mut self, work: SynchronizationWork) -> Result<(), SynchronizationWorkStoreError>;
    fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationWorkStoreError {
    Io(String),
    Serialization(String),
}

/// In-memory reference implementation used by application-level tests.
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
        if let Some(existing) = self
            .work
            .iter_mut()
            .find(|existing| existing.artifact_id == work.artifact_id)
        {
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

/// Persistent application boundary for synchronization work.
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
}

impl<S> PersistentSynchronizationQueue<S>
where
    S: SynchronizationWorkStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }

    /// Enqueues a completed local artifact without copying its recording data.
    /// Re-enqueueing the same artifact/version is idempotent. Reusing the same
    /// artifact identity for different content is rejected explicitly.
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
        lifecycle
            .queue()
            .map_err(SynchronizationQueueError::Lifecycle)?;
        work.from_lifecycle(lifecycle);
        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(work)
    }

    /// Claims the next pending work item deterministically by artifact identity.
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
        work.from_lifecycle(lifecycle);
        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(Some(work))
    }

    /// Applies a transport-neutral transfer outcome to the synchronization
    /// lifecycle. Retryable outcomes return work to Pending; terminal and
    /// integrity outcomes become Failed; successful or already-synchronized
    /// outcomes become Synchronized.
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

        work.apply_transfer_result(result)?;
        self.store
            .save(work.clone())
            .map_err(SynchronizationQueueError::Store)?;
        Ok(work)
    }

    /// Requeues all in-progress work after process interruption.
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
            lifecycle
                .retry()
                .map_err(SynchronizationQueueError::Lifecycle)?;
            work.from_lifecycle(lifecycle);
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

/// Vendor- and transport-neutral request passed to an artifact transfer.
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

/// Opaque continuation information returned by a transfer implementation.
///
/// The application may persist and return this value to a later transfer
/// attempt, but never interprets its contents. A provider may leave it absent
/// when restarting a transfer from the beginning is sufficient.
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

/// Transfer result semantics required by #144/#145.
///
/// These outcomes contain no HTTP, cloud-provider, or transport-specific
/// types. They are sufficient for deterministic application-level handling of
/// success, retry, conflict, and integrity failure.
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

/// Application transfer boundary prepared for concrete remote implementations.
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

    // TEST-01
    #[test]
    fn enqueue_creates_pending_work_without_copying_artifact_data() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());

        let work = queue
            .enqueue(artifact_id("artifact-001"), manifest_hash(1))
            .unwrap();

        assert_eq!(
            work.status(),
            RecordingArtifactSynchronizationStatus::Pending
        );
        assert_eq!(work.artifact_id().value(), "artifact-001");
        assert_eq!(work.manifest_hash(), &manifest_hash(1));
        assert_eq!(queue.list().unwrap().len(), 1);
    }

    // TEST-02
    #[test]
    fn duplicate_enqueue_is_idempotent() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        let first = queue
            .enqueue(artifact_id("artifact-002"), manifest_hash(2))
            .unwrap();
        let second = queue
            .enqueue(artifact_id("artifact-002"), manifest_hash(2))
            .unwrap();

        assert_eq!(first, second);
        assert_eq!(queue.list().unwrap().len(), 1);
    }

    // TEST-03
    #[test]
    fn duplicate_artifact_identity_with_different_manifest_is_rejected() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue
            .enqueue(artifact_id("artifact-003"), manifest_hash(3))
            .unwrap();

        assert_eq!(
            queue.enqueue(artifact_id("artifact-003"), manifest_hash(4)),
            Err(SynchronizationQueueError::ArtifactVersionConflict {
                artifact_id: artifact_id("artifact-003"),
            })
        );
    }

    // TEST-04
    #[test]
    fn claim_is_deterministic_and_moves_pending_work_to_transferring() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue
            .enqueue(artifact_id("artifact-005"), manifest_hash(5))
            .unwrap();
        queue
            .enqueue(artifact_id("artifact-004"), manifest_hash(4))
            .unwrap();

        let claimed = queue.claim_next().unwrap().unwrap();

        assert_eq!(claimed.artifact_id().value(), "artifact-004");
        assert_eq!(
            claimed.status(),
            RecordingArtifactSynchronizationStatus::Transferring
        );
    }

    // TEST-05
    #[test]
    fn interrupted_transfer_returns_to_pending() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue
            .enqueue(artifact_id("artifact-006"), manifest_hash(6))
            .unwrap();
        queue.claim_next().unwrap();

        assert_eq!(queue.recover_interrupted().unwrap(), 1);
        assert_eq!(
            queue.list().unwrap()[0].status(),
            RecordingArtifactSynchronizationStatus::Pending
        );
    }

    // TEST-06
    #[test]
    fn reconstituted_work_preserves_identity_and_state() {
        let work = SynchronizationWork::reconstitute(
            artifact_id("artifact-007"),
            manifest_hash(7),
            RecordingArtifactSynchronizationStatus::Pending,
        );

        assert_eq!(work.artifact_id().value(), "artifact-007");
        assert_eq!(work.manifest_hash(), &manifest_hash(7));
        assert_eq!(
            work.status(),
            RecordingArtifactSynchronizationStatus::Pending
        );
    }

    // TEST-07
    #[test]
    fn transfer_request_preserves_artifact_and_manifest_identity() {
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
        queue
            .enqueue(artifact_id("artifact-008"), manifest_hash(8))
            .unwrap();
        let work = queue.claim_next().unwrap().unwrap();
        let request = work.transfer_request();

        assert_eq!(request.artifact_id().value(), "artifact-008");
        assert_eq!(request.manifest_hash(), &manifest_hash(8));
    }

    // TEST-08
    #[test]
    fn transfer_results_map_to_synchronization_lifecycle() {
        let results = [
            (
                ArtifactTransferResult::Succeeded,
                RecordingArtifactSynchronizationStatus::Synchronized,
            ),
            (
                ArtifactTransferResult::AlreadySynchronized,
                RecordingArtifactSynchronizationStatus::Synchronized,
            ),
            (
                ArtifactTransferResult::RetryableFailure {
                    reason: "offline".to_owned(),
                    continuation: None,
                },
                RecordingArtifactSynchronizationStatus::Pending,
            ),
            (
                ArtifactTransferResult::Conflict {
                    reason: "remote version differs".to_owned(),
                },
                RecordingArtifactSynchronizationStatus::Failed,
            ),
            (
                ArtifactTransferResult::IntegrityFailure {
                    reason: "hash mismatch".to_owned(),
                },
                RecordingArtifactSynchronizationStatus::Failed,
            ),
            (
                ArtifactTransferResult::PermanentFailure {
                    reason: "not retryable".to_owned(),
                },
                RecordingArtifactSynchronizationStatus::Failed,
            ),
        ];

        for (result, expected_status) in results {
            let mut queue =
                PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
            queue
                .enqueue(artifact_id("artifact-result"), manifest_hash(9))
                .unwrap();
            queue.claim_next().unwrap();

            let work = queue
                .apply_transfer_result(&artifact_id("artifact-result"), &result)
                .unwrap();
            assert_eq!(work.status(), expected_status);
        }
    }

    // TEST-09
    #[test]
    fn retryable_failure_can_carry_opaque_continuation() {
        let continuation = TransferContinuation::new([1_u8, 2, 3]);
        let result = ArtifactTransferResult::RetryableFailure {
            reason: "interrupted".to_owned(),
            continuation: Some(continuation.clone()),
        };

        assert_eq!(
            match result {
                ArtifactTransferResult::RetryableFailure { continuation, .. } => continuation,
                _ => None,
            },
            Some(continuation)
        );
    }

    // TEST-10
    #[test]
    fn deterministic_transfer_implementation_can_be_tested_without_vendor_types() {
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
        let success = transfer.transfer(&ArtifactTransferRequest::new(
            artifact_id("artifact-010"),
            manifest_hash(10),
        ));
        assert_eq!(success, ArtifactTransferResult::Succeeded);
    }
}
