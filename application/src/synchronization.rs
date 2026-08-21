//! Persistent synchronization work and transfer boundaries.
//!
//! Synchronization work is persisted separately from the local RecordingArtifact.
//! The artifact remains the source of recording data; this module stores only the
//! recoverable work reference and synchronization state needed to resume later.
//!
//! The transfer contract in this module is deliberately vendor- and
//! transport-neutral. It is the application boundary prepared for the later
//! concrete transfer implementation.
//!
//! See ADR-068 and #66 / #143 / #144 / #145.

use std::fs;
use std::path::{Path, PathBuf};

use nc_pore_core::recording::{
    RecordingArtifactId, RecordingArtifactSynchronization, RecordingArtifactSynchronizationError,
    RecordingArtifactSynchronizationStatus,
};
use serde::{Deserialize, Serialize};

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

    pub fn artifact_id(&self) -> &RecordingArtifactId {
        &self.artifact_id
    }

    pub fn manifest_hash(&self) -> &[u8; 32] {
        &self.manifest_hash
    }

    pub fn status(&self) -> RecordingArtifactSynchronizationStatus {
        self.status
    }

    fn lifecycle(&self) -> RecordingArtifactSynchronization {
        RecordingArtifactSynchronization::reconstitute(self.artifact_id.clone(), self.status)
    }

    fn from_lifecycle(&mut self, lifecycle: RecordingArtifactSynchronization) {
        self.status = lifecycle.status();
    }
}

/// Serializable representation kept outside Core so the domain remains free
/// of persistence and serialization concerns.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedSynchronizationWork {
    artifact_id: String,
    manifest_hash: [u8; 32],
    status: PersistedSynchronizationStatus,
}

impl From<&SynchronizationWork> for PersistedSynchronizationWork {
    fn from(work: &SynchronizationWork) -> Self {
        Self {
            artifact_id: work.artifact_id.value().to_owned(),
            manifest_hash: work.manifest_hash,
            status: PersistedSynchronizationStatus::from_core(work.status),
        }
    }
}

impl PersistedSynchronizationWork {
    fn into_work(self) -> SynchronizationWork {
        SynchronizationWork {
            artifact_id: RecordingArtifactId::new(self.artifact_id),
            manifest_hash: self.manifest_hash,
            status: self.status.into_core(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedSynchronizationStatus {
    Local,
    Pending,
    Transferring,
    Synchronized,
    Failed,
}

impl PersistedSynchronizationStatus {
    fn into_core(self) -> RecordingArtifactSynchronizationStatus {
        match self {
            Self::Local => RecordingArtifactSynchronizationStatus::Local,
            Self::Pending => RecordingArtifactSynchronizationStatus::Pending,
            Self::Transferring => RecordingArtifactSynchronizationStatus::Transferring,
            Self::Synchronized => RecordingArtifactSynchronizationStatus::Synchronized,
            Self::Failed => RecordingArtifactSynchronizationStatus::Failed,
        }
    }

    fn from_core(status: RecordingArtifactSynchronizationStatus) -> Self {
        match status {
            RecordingArtifactSynchronizationStatus::Local => Self::Local,
            RecordingArtifactSynchronizationStatus::Pending => Self::Pending,
            RecordingArtifactSynchronizationStatus::Transferring => Self::Transferring,
            RecordingArtifactSynchronizationStatus::Synchronized => Self::Synchronized,
            RecordingArtifactSynchronizationStatus::Failed => Self::Failed,
        }
    }
}

/// Persistent store for synchronization work.
///
/// The store contains work references, never recording payload data.
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

/// Filesystem implementation of the synchronization work store.
///
/// Synchronization state is kept in its own file and is therefore independent
/// from the local artifact directory and its payload files.
pub struct FilesystemSynchronizationWorkStore {
    path: PathBuf,
}

impl FilesystemSynchronizationWorkStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            path: root.into().join("synchronization-work.json"),
        }
    }

    fn read(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError> {
        if !self.path.is_file() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        let persisted: Vec<PersistedSynchronizationWork> = serde_json::from_str(&content)
            .map_err(|error| SynchronizationWorkStoreError::Serialization(error.to_string()))?;

        Ok(persisted
            .into_iter()
            .map(PersistedSynchronizationWork::into_work)
            .collect())
    }

    fn write(&self, work: &[SynchronizationWork]) -> Result<(), SynchronizationWorkStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        }

        let persisted: Vec<PersistedSynchronizationWork> = work
            .iter()
            .map(PersistedSynchronizationWork::from)
            .collect();
        let content = serde_json::to_string_pretty(&persisted)
            .map_err(|error| SynchronizationWorkStoreError::Serialization(error.to_string()))?;
        let temp_path = self.path.with_extension("json.tmp");

        fs::write(&temp_path, content)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        fs::rename(&temp_path, &self.path)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;

        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl SynchronizationWorkStore for FilesystemSynchronizationWorkStore {
    fn save(&mut self, work: SynchronizationWork) -> Result<(), SynchronizationWorkStoreError> {
        let mut all = self.read()?;

        if let Some(existing) = all
            .iter_mut()
            .find(|existing| existing.artifact_id == work.artifact_id)
        {
            *existing = work;
        } else {
            all.push(work);
        }

        all.sort_by(|left, right| left.artifact_id.value().cmp(right.artifact_id.value()));
        self.write(&all)
    }

    fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError> {
        let mut work = self.read()?;
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

/// Transfer result semantics required by #144/#145.
///
/// These outcomes contain no HTTP, cloud-provider, or transport-specific
/// types. They are sufficient for deterministic application-level handling of
/// success, retry, conflict, and integrity failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactTransferResult {
    Succeeded,
    AlreadySynchronized,
    RetryableFailure(String),
    Conflict(String),
    IntegrityFailure(String),
    PermanentFailure(String),
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
    fn filesystem_store_survives_reconstruction() {
        let root = std::env::temp_dir().join("nc-pore-sync-work-test-06");
        let _ = fs::remove_dir_all(&root);

        let mut first =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        first
            .enqueue(artifact_id("artifact-007"), manifest_hash(7))
            .unwrap();
        drop(first);

        let second =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        let work = second.list().unwrap();

        assert_eq!(work.len(), 1);
        assert_eq!(work[0].artifact_id().value(), "artifact-007");
        assert_eq!(
            work[0].status(),
            RecordingArtifactSynchronizationStatus::Pending
        );

        let _ = fs::remove_dir_all(root);
    }

    // TEST-07
    #[test]
    fn filesystem_store_recovers_interrupted_work_after_reconstruction() {
        let root = std::env::temp_dir().join("nc-pore-sync-work-test-07");
        let _ = fs::remove_dir_all(&root);

        let mut first =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        first
            .enqueue(artifact_id("artifact-008"), manifest_hash(8))
            .unwrap();
        first.claim_next().unwrap();
        drop(first);

        let mut second =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        assert_eq!(second.recover_interrupted().unwrap(), 1);
        assert_eq!(
            second.list().unwrap()[0].status(),
            RecordingArtifactSynchronizationStatus::Pending
        );

        let _ = fs::remove_dir_all(root);
    }

    // TEST-08
    #[test]
    fn transfer_request_preserves_artifact_and_manifest_identity() {
        let request = ArtifactTransferRequest::new(artifact_id("artifact-009"), manifest_hash(9));

        assert_eq!(request.artifact_id().value(), "artifact-009");
        assert_eq!(request.manifest_hash(), &manifest_hash(9));
    }

    // TEST-09
    #[test]
    fn transfer_results_are_vendor_neutral() {
        assert_eq!(
            ArtifactTransferResult::Succeeded,
            ArtifactTransferResult::Succeeded
        );
        assert_eq!(
            ArtifactTransferResult::RetryableFailure("offline".to_owned()),
            ArtifactTransferResult::RetryableFailure("offline".to_owned())
        );
        assert_eq!(
            ArtifactTransferResult::IntegrityFailure("hash mismatch".to_owned()),
            ArtifactTransferResult::IntegrityFailure("hash mismatch".to_owned())
        );
    }
}
