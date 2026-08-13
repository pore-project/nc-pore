use crate::artifact::{
    ArtifactCoordinator, ArtifactStatus, CaptureResult, RecordingArtifactAssociation,
    RecordingSessionId,
};
use crate::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

/// Processes completed recording captures into persisted artifacts.
pub struct RecordingArtifactProcessor {
    coordinator: ArtifactCoordinator<InMemoryPersistenceProvider>,
}

impl RecordingArtifactProcessor {
    pub fn new(coordinator: ArtifactCoordinator<InMemoryPersistenceProvider>) -> Self {
        Self { coordinator }
    }

    pub fn process(
        &mut self,
        capture_result: CaptureResult,
        session_id: RecordingSessionId,
        association: RecordingArtifactAssociation,
    ) -> crate::artifact::RecordingArtifact {
        self.coordinator
            .create_artifact(capture_result, session_id, association)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn processor_creates_and_coordinates_artifact() {
        let persistence = InMemoryPersistenceProvider::new();

        let coordinator = ArtifactCoordinator::new(persistence);

        let mut processor = RecordingArtifactProcessor::new(coordinator);

        let capture_result = CaptureResult::new("capture-001");

        let artifact = processor.process(
            capture_result,
            RecordingSessionId::new("session-001"),
            RecordingArtifactAssociation::new("production-001", "recording-017"),
        );

        assert_eq!(artifact.id.value(), "capture-001");
        assert_eq!(artifact.status(), &ArtifactStatus::Available);
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));

        let persisted = match processor
            .coordinator
            .persistence()
            .load("capture-001")
        {
            PersistenceLoadResult::Valid(artifact) => artifact,
            result => panic!("processed artifact must be valid, got {result:?}"),
        };

        assert_eq!(persisted.production_id(), Some("production-001"));
        assert_eq!(persisted.recording_id(), Some("recording-017"));
    }
}
