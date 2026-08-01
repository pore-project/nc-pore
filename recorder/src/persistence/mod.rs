use crate::artifact::RecordingArtifact;

/// Technical persistence boundary for Recording Artifacts.
///
/// This trait intentionally abstracts persistence from the recorder workflow.
/// The first implementation is in-memory only to validate the architectural
/// boundary without introducing a storage technology decision.
///
/// Future implementations may provide filesystem, database or other storage
/// mechanisms without changing the recorder workflow.
pub trait PersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact);
    fn get_all(&self) -> Vec<RecordingArtifact>;
}

/// Temporary technical implementation of the persistence boundary.
///
/// This implementation validates the interaction between
/// Recording Artifact lifecycle and persistence handling.
///
/// It deliberately does not represent the final storage solution.
#[derive(Debug, Default)]
pub struct InMemoryPersistenceProvider {
    artifacts: Vec<RecordingArtifact>,
}

impl PersistenceProvider for InMemoryPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.artifacts.push(artifact);
    }

    fn get_all(&self) -> Vec<RecordingArtifact> {
        self.artifacts.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::RecordingArtifact;

    // Test-01:
    // Verify that a Recording Artifact can cross the persistence boundary.
    //
    // This validates the basic contract between the recorder workflow
    // and the persistence layer.
    #[test]
    fn test_01_artifact_can_be_persisted_in_memory() {
        let artifact = RecordingArtifact::new(
            "artifact-001".to_string(),
            "session-001".to_string(),
        );

        let mut provider = InMemoryPersistenceProvider::default();

        provider.store(artifact);

        assert_eq!(provider.get_all().len(), 1);
    }

    // Test-02:
    // Verify that a persisted Recording Artifact can be retrieved again.
    //
    // This validates that persistence keeps the technical identity
    // and session reference of the stored artifact.
    #[test]
    fn test_02_persisted_artifact_can_be_retrieved() {
        let artifact = RecordingArtifact::new(
            "artifact-002".to_string(),
            "session-002".to_string(),
        );

        let mut provider = InMemoryPersistenceProvider::default();

        provider.store(artifact);

        let stored_artifacts = provider.get_all();

        assert_eq!(stored_artifacts[0].id, "artifact-002");
        assert_eq!(
            stored_artifacts[0].recording_session_id,
            "session-002"
        );
    }
}
