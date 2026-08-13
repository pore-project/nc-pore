//! Recording Artifact processing boundary.
//!
//! This module transforms completed CaptureResult instances
//! into managed RecordingArtifact instances.
//!
//! It connects:
//! - CaptureResult
//! - RecordingArtifactFactory
//! - ArtifactCoordinator
//! - originating domain recording association
//!
//! It intentionally does not contain:
//! - audio capture logic
//! - workflow coordination
//! - persistence implementation
//! - registry implementation
//! - domain rules
//!
//! See:
//! - ADR-051 Recording Artifact Processing Boundary

use crate::artifact::RecordingArtifactAssociation;
use crate::artifact::coordination::ArtifactCoordinator;
use crate::artifact::factory::RecordingArtifactFactory;
use crate::audio::CaptureResult;
use crate::persistence::{PersistenceLoadResult, PersistenceProvider};
use crate::session::RecordingSessionId;

/// Processes completed capture results into recording artifacts.
///
/// The processor connects capture completion with artifact management
/// while keeping workflow coordination independent from artifact details.
pub struct RecordingArtifactProcessor<P>
where
    P: PersistenceProvider,
{
    coordinator: ArtifactCoordinator<P>,
}

impl<P> RecordingArtifactProcessor<P>
where
    P: PersistenceProvider,
{
    /// Creates a new recording artifact processor.
    pub fn new(coordinator: ArtifactCoordinator<P>) -> Self {
        Self { coordinator }
    }

    /// Processes a completed capture result and preserves its originating
    /// domain recording association on the resulting artifact.
    pub fn process(
        &mut self,
        capture_result: CaptureResult,
        recording_session_id: RecordingSessionId,
        association: RecordingArtifactAssociation,
    ) -> crate::artifact::RecordingArtifact {
        let mut artifact = RecordingArtifactFactory::create(capture_result, recording_session_id);

        artifact.set_domain_association(association.production_id(), association.recording_id());
        artifact.make_available();

        self.coordinator.register_and_store(artifact.clone());

        artifact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::ArtifactStatus;
    use crate::persistence::InMemoryPersistenceProvider;

    // TEST-23
    //
    // Protects ADR-051:
    // Processing connects capture results with artifact creation
    // and artifact coordination.
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
        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));

        let persisted = match processor.coordinator.persistence().load("capture-001") {
            PersistenceLoadResult::Valid(artifact) => artifact,
            result => panic!("processed artifact must be valid, got {result:?}"),
        };

        assert_eq!(persisted.production_id(), Some("production-001"));
        assert_eq!(persisted.recording_id(), Some("recording-017"));
        assert_eq!(persisted.status(), &ArtifactStatus::Stored);
    }

    // TEST-34
    //
    // Protects ADR-060:
    // Successful persistence is represented by a Stored artifact at the
    // processing boundary rather than returning the pre-persistence state.
    #[test]
    fn processor_returns_stored_artifact_after_successful_persistence() {
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let mut processor = RecordingArtifactProcessor::new(coordinator);

        let artifact = processor.process(
            CaptureResult::new("capture-034"),
            RecordingSessionId::new("session-034"),
            RecordingArtifactAssociation::new("production-034", "recording-034"),
        );

        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
    }

    // TEST-35
    //
    // Protects ADR-060:
    // Processing the same artifact identity repeatedly must be idempotent
    // and must not create duplicate persisted artifacts.
    #[test]
    fn processor_is_idempotent_for_same_artifact_identity() {
        let persistence = InMemoryPersistenceProvider::new();
        let coordinator = ArtifactCoordinator::new(persistence);
        let mut processor = RecordingArtifactProcessor::new(coordinator);

        let association = RecordingArtifactAssociation::new("production-035", "recording-035");

        processor.process(
            CaptureResult::new("capture-035"),
            RecordingSessionId::new("session-035"),
            association.clone(),
        );

        processor.process(
            CaptureResult::new("capture-035"),
            RecordingSessionId::new("session-035"),
            association,
        );

        assert_eq!(processor.coordinator.persistence().list().len(), 1);
    }
}