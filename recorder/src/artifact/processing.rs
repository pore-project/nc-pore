//! Recording Artifact processing boundary.
//!
//! This module transforms completed CaptureResult instances
//! into managed RecordingArtifact instances through the explicit
//! local preservation boundary.
//!
//! It connects:
//! - CaptureResult
//! - CapturePreserver
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
use crate::persistence::{PersistenceProvider, PersistenceStoreError};
use crate::preservation::CapturePreserver;
use crate::session::RecordingSessionId;

/// Processes completed capture results into recording artifacts.
///
/// The processor first crosses the capture -> preservation boundary. Artifact
/// creation therefore consumes an owned preservation snapshot rather than
/// treating the raw capture result as if it were already preserved.
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
    ///
    /// A successful result is the Stored artifact returned by persistence.
    /// Persistence failures are propagated without turning the Available
    /// artifact into a falsely Stored result.
    pub fn process(
        &mut self,
        capture_result: CaptureResult,
        recording_session_id: RecordingSessionId,
        association: RecordingArtifactAssociation,
    ) -> Result<crate::artifact::RecordingArtifact, PersistenceStoreError> {
        let preserved_capture = CapturePreserver::preserve(capture_result);
        let mut artifact =
            RecordingArtifactFactory::create(preserved_capture, recording_session_id);

        artifact.set_domain_association(association.production_id(), association.recording_id());
        artifact.make_available();

        self.coordinator.register_and_store(artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{ArtifactId, ArtifactStatus, RecordingArtifact};
    use crate::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

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

        let artifact = processor
            .process(
                capture_result,
                RecordingSessionId::new("session-001"),
                RecordingArtifactAssociation::new("production-001", "recording-017"),
            )
            .expect("processing should persist artifact");

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

        let artifact = processor
            .process(
                CaptureResult::new("capture-034"),
                RecordingSessionId::new("session-034"),
                RecordingArtifactAssociation::new("production-034", "recording-034"),
            )
            .expect("processing should persist artifact");

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

        processor
            .process(
                CaptureResult::new("capture-035"),
                RecordingSessionId::new("session-035"),
                association.clone(),
            )
            .expect("first processing should succeed");

        processor
            .process(
                CaptureResult::new("capture-035"),
                RecordingSessionId::new("session-035"),
                association,
            )
            .expect("equivalent processing should be idempotent");

        assert_eq!(processor.coordinator.persistence().list().len(), 1);
    }

    struct FailingPersistenceProvider {
        attempted: Option<RecordingArtifact>,
    }

    impl PersistenceProvider for FailingPersistenceProvider {
        fn store(&mut self, _artifact: RecordingArtifact) {}

        fn store_checked(
            &mut self,
            artifact: RecordingArtifact,
        ) -> Result<RecordingArtifact, PersistenceStoreError> {
            self.attempted = Some(artifact);
            Err(PersistenceStoreError::Io(
                "test persistence failure".to_owned(),
            ))
        }

        fn load(&self, _id: &str) -> PersistenceLoadResult {
            PersistenceLoadResult::NotFound
        }

        fn list_ids(&self) -> Vec<String> {
            Vec::new()
        }

        fn list(&self) -> Vec<RecordingArtifact> {
            Vec::new()
        }

        fn remove(&mut self, _id: &str) {}
    }

    // TEST-40
    //
    // Protects ADR-060:
    // A persistence failure must be propagated and the attempted artifact
    // must remain Available rather than being reported as Stored.
    #[test]
    fn processor_preserves_available_state_when_persistence_fails() {
        let coordinator = ArtifactCoordinator::new(FailingPersistenceProvider { attempted: None });
        let mut processor = RecordingArtifactProcessor::new(coordinator);

        let result = processor.process(
            CaptureResult::new("capture-040"),
            RecordingSessionId::new("session-040"),
            RecordingArtifactAssociation::new("production-040", "recording-040"),
        );

        assert!(matches!(
            result,
            Err(PersistenceStoreError::Io(message)) if message == "test persistence failure"
        ));

        let attempted = processor
            .coordinator
            .persistence()
            .attempted
            .as_ref()
            .expect("failing provider must retain attempted artifact for the test");

        assert_eq!(attempted.id, ArtifactId::new("capture-040"));
        assert_eq!(attempted.status(), &ArtifactStatus::Available);
    }
}
