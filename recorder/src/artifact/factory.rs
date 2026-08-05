//! Recording Artifact factory.
//!
//! This module creates RecordingArtifact instances
//! from completed capture results.
//!
//! It intentionally does not contain:
//! - workflow coordination
//! - capture logic
//! - persistence logic
//! - registry management
//!
//! See:
//! - ADR-050 Recording Artifact Factory

use crate::artifact::RecordingArtifact;
use crate::audio::CaptureResult;

/// Creates RecordingArtifact instances.
///
/// The factory encapsulates artifact construction
/// and keeps creation logic separate from workflow coordination.
pub struct RecordingArtifactFactory;

impl RecordingArtifactFactory {
    /// Creates a new RecordingArtifact from a capture result.
    pub fn create(
        capture_result: CaptureResult,
        recording_session_id: impl Into<String>,
    ) -> RecordingArtifact {
        RecordingArtifact::new(capture_result.id(), recording_session_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-22
    //
    // Protects ADR-050:
    // Artifact creation is encapsulated in the factory.
    #[test]
    fn factory_creates_artifact_from_capture_result() {
        let capture_result = CaptureResult::new("capture-001");

        let artifact = RecordingArtifactFactory::create(capture_result, "session-001");

        assert_eq!(artifact.id, "capture-001");
        assert_eq!(artifact.recording_session_id, "session-001");
    }
}
