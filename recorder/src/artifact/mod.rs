//! Recording Artifact model.
//!
//! This module represents the technical result of a local recording.
//!
//! A Recording Artifact is intentionally separated from:
//! - production domain objects
//! - storage implementations
//! - synchronization logic
//! - export processing
//!
//! See:
//! - ADR-042 Recording Artifact Model and Lifecycle Boundary

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactStatus {
    Created,
    Available,
    Stored,
}

#[derive(Debug, Clone)]
pub struct RecordingArtifact {
    pub id: String,
    pub recording_session_id: String,
    status: ArtifactStatus,
}

impl RecordingArtifact {
    /// Creates a new recording artifact.
    ///
    /// A new artifact represents a technical recording result
    /// that has been created but is not yet available or stored.
    pub fn new(
        id: impl Into<String>,
        recording_session_id: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            recording_session_id: recording_session_id.into(),
            status: ArtifactStatus::Created,
        }
    }

    /// Returns the current artifact status.
    pub fn status(&self) -> &ArtifactStatus {
        &self.status
    }

    /// Marks the artifact as available.
    pub fn make_available(&mut self) {
        self.status = ArtifactStatus::Available;
    }

    /// Marks the artifact as stored.
    pub fn store(&mut self) {
        self.status = ArtifactStatus::Stored;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-09
    // Verify: A new artifact starts in Created state.
    //
    // This protects ADR-042:
    // Recording artifacts have their own technical lifecycle
    // independent from domain recording states.
    #[test]
    fn new_artifact_starts_as_created() {
        let artifact = RecordingArtifact::new(
            "artifact-001",
            "session-001",
        );

        assert_eq!(artifact.status(), &ArtifactStatus::Created);
    }

    // TEST-10
    // Verify: Artifact lifecycle can progress from Created to Available.
    #[test]
    fn artifact_can_become_available() {
        let mut artifact = RecordingArtifact::new(
            "artifact-001",
            "session-001",
        );

        artifact.make_available();

        assert_eq!(artifact.status(), &ArtifactStatus::Available);
    }

    // TEST-11
    // Verify: Available artifacts can be stored.
    #[test]
    fn artifact_can_be_stored() {
        let mut artifact = RecordingArtifact::new(
            "artifact-001",
            "session-001",
        );

        artifact.make_available();
        artifact.store();

        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
    }
}
