//! Identity reference to the technical RecordingArtifact associated with a domain Recording.
//!
//! RecordingArtifactId is intentionally distinct from the technical
//! recorder-side ArtifactId type. The Core domain stores only this opaque
//! reference and does not depend on the technical RecordingArtifact model.
//!
//! See ADR-054 Recording Artifact and Local Recording Data Association.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingArtifactId(String);

impl RecordingArtifactId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_artifact_id_preserves_value() {
        let id = RecordingArtifactId::new("artifact-001");

        assert_eq!(id.value(), "artifact-001");
    }
}
