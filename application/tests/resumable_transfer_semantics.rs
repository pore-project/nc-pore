use std::collections::{HashMap, HashSet};

use nc_pore_application::synchronization::{
    ArtifactTransfer, ArtifactTransferRequest, ArtifactTransferResult,
    InMemorySynchronizationWorkStore, PersistentSynchronizationQueue, TransferContinuation,
};
use nc_pore_core::recording::{RecordingArtifactId, RecordingArtifactSynchronizationStatus};

fn artifact_id(value: &str) -> RecordingArtifactId {
    RecordingArtifactId::new(value)
}

fn manifest_hash(value: u8) -> [u8; 32] {
    [value; 32]
}

/// Deterministic remote used only to exercise the vendor-neutral transfer contract.
///
/// It models the three remote states relevant to #145 without introducing a
/// provider, protocol, authentication mechanism, or network dependency:
/// absent, present with an identical manifest, and present with a conflicting
/// manifest. Interruptions and integrity failures are deterministic faults.
struct DeterministicRemote {
    objects: HashMap<RecordingArtifactId, [u8; 32]>,
    interrupt_once: HashSet<RecordingArtifactId>,
    corrupt_once: HashSet<RecordingArtifactId>,
}

impl DeterministicRemote {
    fn new() -> Self {
        Self {
            objects: HashMap::new(),
            interrupt_once: HashSet::new(),
            corrupt_once: HashSet::new(),
        }
    }

    fn interrupt_once(&mut self, artifact_id: RecordingArtifactId) {
        self.interrupt_once.insert(artifact_id);
    }

    fn corrupt_once(&mut self, artifact_id: RecordingArtifactId) {
        self.corrupt_once.insert(artifact_id);
    }

    fn remote_hash(&self, artifact_id: &RecordingArtifactId) -> Option<[u8; 32]> {
        self.objects.get(artifact_id).copied()
    }
}

impl ArtifactTransfer for DeterministicRemote {
    fn transfer(&mut self, request: &ArtifactTransferRequest) -> ArtifactTransferResult {
        if let Some(remote_hash) = self.objects.get(request.artifact_id()) {
            if remote_hash == request.manifest_hash() {
                return ArtifactTransferResult::AlreadySynchronized;
            }
            return ArtifactTransferResult::Conflict {
                reason: "remote artifact has a different manifest".to_owned(),
            };
        }

        if self.interrupt_once.remove(request.artifact_id()) {
            return ArtifactTransferResult::RetryableFailure {
                reason: "deterministic interruption".to_owned(),
                continuation: Some(TransferContinuation::new(b"resume-token")),
            };
        }

        if self.corrupt_once.remove(request.artifact_id()) {
            return ArtifactTransferResult::IntegrityFailure {
                reason: "deterministic integrity mismatch".to_owned(),
            };
        }

        self.objects
            .insert(request.artifact_id().clone(), *request.manifest_hash());
        ArtifactTransferResult::Succeeded
    }
}

#[test]
fn interrupted_transfer_can_resume_and_finish_without_duplicate_remote_state() {
    let mut remote = DeterministicRemote::new();
    let id = artifact_id("artifact-resume");
    remote.interrupt_once(id.clone());

    let request = ArtifactTransferRequest::new(id.clone(), manifest_hash(1));
    let first = remote.transfer(&request);

    assert!(matches!(
        first,
        ArtifactTransferResult::RetryableFailure {
            continuation: Some(_),
            ..
        }
    ));
    assert_eq!(remote.remote_hash(&id), None);

    let second = remote.transfer(&request);
    assert_eq!(second, ArtifactTransferResult::Succeeded);
    assert_eq!(remote.remote_hash(&id), Some(manifest_hash(1)));

    let third = remote.transfer(&request);
    assert_eq!(third, ArtifactTransferResult::AlreadySynchronized);
}

#[test]
fn incompatible_remote_representation_is_a_deterministic_conflict() {
    let mut remote = DeterministicRemote::new();
    let id = artifact_id("artifact-conflict");

    let original = ArtifactTransferRequest::new(id.clone(), manifest_hash(2));
    assert_eq!(
        remote.transfer(&original),
        ArtifactTransferResult::Succeeded
    );

    let incompatible = ArtifactTransferRequest::new(id.clone(), manifest_hash(3));
    assert_eq!(
        remote.transfer(&incompatible),
        ArtifactTransferResult::Conflict {
            reason: "remote artifact has a different manifest".to_owned(),
        }
    );
    assert_eq!(remote.remote_hash(&id), Some(manifest_hash(2)));
}

#[test]
fn integrity_failure_does_not_create_or_modify_remote_artifact() {
    let mut remote = DeterministicRemote::new();
    let id = artifact_id("artifact-integrity");
    remote.corrupt_once(id.clone());

    let request = ArtifactTransferRequest::new(id.clone(), manifest_hash(4));
    assert_eq!(
        remote.transfer(&request),
        ArtifactTransferResult::IntegrityFailure {
            reason: "deterministic integrity mismatch".to_owned(),
        }
    );
    assert_eq!(remote.remote_hash(&id), None);
}

#[test]
fn transfer_outcomes_drive_the_persistent_lifecycle_without_mutating_local_identity() {
    let mut queue = PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());
    let id = artifact_id("artifact-lifecycle");
    let hash = manifest_hash(5);

    queue.enqueue(id.clone(), hash).unwrap();
    queue.claim_next().unwrap();

    let work = queue
        .apply_transfer_result(
            &id,
            &ArtifactTransferResult::IntegrityFailure {
                reason: "deterministic integrity mismatch".to_owned(),
            },
        )
        .unwrap();

    assert_eq!(
        work.status(),
        RecordingArtifactSynchronizationStatus::Failed
    );
    assert_eq!(work.artifact_id(), &id);
    assert_eq!(work.manifest_hash(), &hash);
}
