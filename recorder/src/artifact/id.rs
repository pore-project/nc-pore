//! Recording Artifact identity.
//!
//! ArtifactId represents the technical identity
//! of a RecordingArtifact.
//!
//! It is intentionally independent from:
//! - persistence paths
//! - filenames
//! - storage providers
//! - synchronization mechanisms
//!
//! See:
//! - ADR-042 Recording Artifact Model and Lifecycle Boundary
//! - ADR-054 Recording Artifact and Local Recording Data Association

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Creates a new artifact identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Recording Track identity.
///
/// RecordingTrackId represents the technical identity
/// of a RecordingTrack and is intentionally distinct
/// from ArtifactId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingTrackId(String);

impl RecordingTrackId {
    /// Creates a new recording track identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_track_id_preserves_value() {
        let id = RecordingTrackId::new("track-001");

        assert_eq!(id.value(), "track-001");
    }
}
