//! Integration boundary between completed local persistence and synchronization.
//!
//! The helper intentionally loads the artifact again through the persistence
//! boundary before creating synchronization work. This guarantees that a work
//! item cannot become pending before the local artifact has actually been
//! persisted and validated.

use nc_pore_core::recording::RecordingArtifactId;
use recorder::persistence::{PersistenceLoadResult, PersistenceProvider};

use crate::synchronization::{PersistentSynchronizationQueue, SynchronizationQueueError, SynchronizationWork, SynchronizationWorkStore};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SynchronizationEnqueueError {
    Queue(SynchronizationQueueError),
    ArtifactNotPersisted,
    ArtifactInvalid,
}

/// Enqueues a completed artifact only after it can be loaded as valid persisted data.
///
/// No synchronization work is created for missing, incomplete or inconsistent
/// artifacts. The local persistence boundary therefore remains authoritative
/// for whether an artifact is eligible for remote synchronization.
pub fn enqueue_persisted_artifact<W, P>(
    queue: &mut PersistentSynchronizationQueue<W>,
    persistence: &P,
    artifact_id: &RecordingArtifactId,
) -> Result<SynchronizationWork, SynchronizationEnqueueError>
where
    W: SynchronizationWorkStore,
    P: PersistenceProvider,
{
    let artifact = match persistence.load(artifact_id.value()) {
        PersistenceLoadResult::Valid(artifact) => artifact,
        PersistenceLoadResult::NotFound => {
            return Err(SynchronizationEnqueueError::ArtifactNotPersisted);
        }
        PersistenceLoadResult::Incomplete | PersistenceLoadResult::Inconsistent => {
            return Err(SynchronizationEnqueueError::ArtifactInvalid);
        }
    };

    queue
        .enqueue(
            RecordingArtifactId::new(artifact.id.value()),
            *artifact.manifest_hash().as_bytes(),
        )
        .map_err(SynchronizationEnqueueError::Queue)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synchronization::InMemorySynchronizationWorkStore;
    use recorder::artifact::RecordingArtifact;
    use recorder::persistence::InMemoryPersistenceProvider;
    use recorder::session::RecordingSessionId;

    fn artifact(id: &str) -> RecordingArtifact {
        RecordingArtifact::new(id, RecordingSessionId::new("session-163"))
    }

    // TEST-01: persisted artifacts become pending synchronization work.
    #[test]
    fn persisted_artifact_is_enqueued_after_successful_persistence() {
        let mut persistence = InMemoryPersistenceProvider::new();
        let stored = persistence.store_checked(artifact("artifact-163-01")).unwrap();
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());

        let work = enqueue_persisted_artifact(
            &mut queue,
            &persistence,
            &RecordingArtifactId::new(stored.id.value()),
        )
        .unwrap();

        assert_eq!(work.artifact_id().value(), "artifact-163-01");
        assert_eq!(
            work.status(),
            nc_pore_core::recording::RecordingArtifactSynchronizationStatus::Pending
        );
    }

    // TEST-02: missing persistence never creates synchronization work.
    #[test]
    fn missing_artifact_is_not_enqueued() {
        let persistence = InMemoryPersistenceProvider::new();
        let mut queue =
            PersistentSynchronizationQueue::new(InMemorySynchronizationWorkStore::new());

        assert_eq!(
            enqueue_persisted_artifact(
                &mut queue,
                &persistence,
                &RecordingArtifactId::new("missing"),
            ),
            Err(SynchronizationEnqueueError::ArtifactNotPersisted)
        );
        assert!(queue.list().unwrap().is_empty());
    }
}
