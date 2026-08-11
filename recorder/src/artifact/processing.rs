//! Recording Artifact processing boundary.
//!
//! This module transforms completed CaptureResult instances
//! into managed RecordingArtifact instances.
//!
//! It connects:
//! - CaptureResult
//! - RecordingArtifactFactory
//! - ArtifactCoordinator
//!
//! It intentionally does not contain:
//! - audio capture logic
//! - workflow coordination
//! - persistence implementation
//! - registry implementation
//!
//! See:
//! - ADR-051 Recording Artifact Processing Boundary

use crate::artifact::coordination::ArtifactCoordinator;
use crate::artifact::factory::RecordingArtifactFactory;
use crate::audio::CaptureResult;
use crate::persistence::PersistenceProvider;

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

    /// Processes a completed capture result.
    pub fn process(
        &mut self,
        capture_result: CaptureResult,
        recording_session_id: impl Into<String>,
    ) -> crate::artifact::RecordingArtifact {
        let mut artifact = RecordingArtifactFactory::create(capture_result, recording_session_id);

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

        let artifact = processor.process(capture_result, "session-001");

        assert_eq!(artifact.id.value(), "capture-001");
        assert_eq!(artifact.status(), &ArtifactStatus::Available);

        assert!(
            processor
                .coordinator
                .persistence()
                .load("capture-001")
                .is_some()
        );
    }
}
