//! Provider-neutral information exposed to remote artifact providers.
//!
//! The remote boundary deliberately carries recording information rather than
//! provider-specific paths or filenames. A provider may use or ignore any
//! field when constructing its own remote representation.

use std::time::SystemTime;

use crate::artifact::RecordingArtifact;

/// Optional provider-neutral recording information for a finished artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteArtifactMetadata {
    recording_started_at: SystemTime,
    display_name: Option<String>,
}

impl RemoteArtifactMetadata {
    /// Creates metadata for a finished recording.
    pub fn new(recording_started_at: SystemTime, display_name: Option<String>) -> Self {
        Self {
            recording_started_at,
            display_name: normalize_display_name(display_name),
        }
    }

    /// Returns the original recording start time.
    pub fn recording_started_at(&self) -> SystemTime {
        self.recording_started_at
    }

    /// Returns the optional host-provided human-readable name.
    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }
}

/// Provider-neutral transfer view of a completed recording artifact.
///
/// The artifact remains the authoritative technical transfer unit. The
/// metadata is deliberately supplied alongside it so providers can use
/// recording information without introducing provider-specific fields into
/// the artifact model.
#[derive(Debug, Clone)]
pub struct RemoteArtifact<'a> {
    artifact: &'a RecordingArtifact,
    metadata: RemoteArtifactMetadata,
}

impl<'a> RemoteArtifact<'a> {
    /// Creates a provider-neutral view of a completed artifact.
    pub fn new(
        artifact: &'a RecordingArtifact,
        recording_started_at: SystemTime,
        display_name: Option<String>,
    ) -> Self {
        Self {
            artifact,
            metadata: RemoteArtifactMetadata::new(recording_started_at, display_name),
        }
    }

    /// Returns the complete finished artifact that remains the transfer unit.
    pub fn artifact(&self) -> &'a RecordingArtifact {
        self.artifact
    }

    /// Returns the provider-neutral recording metadata.
    pub fn metadata(&self) -> &RemoteArtifactMetadata {
        &self.metadata
    }
}

fn normalize_display_name(display_name: Option<String>) -> Option<String> {
    display_name.and_then(|name| {
        let trimmed = name.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::RecordingSessionId;

    // TEST-01: Empty display names are absent rather than semantic fallbacks.
    #[test]
    fn empty_display_name_is_absent() {
        let metadata = RemoteArtifactMetadata::new(SystemTime::UNIX_EPOCH, Some("  ".into()));

        assert_eq!(metadata.display_name(), None);
    }

    // TEST-02: Host-provided display names are preserved after whitespace normalization.
    #[test]
    fn display_name_is_preserved() {
        let metadata = RemoteArtifactMetadata::new(
            SystemTime::UNIX_EPOCH,
            Some(" Interview Frizz Feick ".into()),
        );

        assert_eq!(metadata.display_name(), Some("Interview Frizz Feick"));
    }

    // TEST-03: Providers receive the complete artifact as the transfer unit.
    #[test]
    fn remote_view_keeps_artifact_as_transfer_unit() {
        let session_id = RecordingSessionId::new("session-001");
        let artifact = RecordingArtifact::new("artifact-001", session_id);
        let remote = RemoteArtifact::new(&artifact, SystemTime::UNIX_EPOCH, None);

        assert_eq!(remote.artifact().id.value(), "artifact-001");
        assert_eq!(remote.metadata().display_name(), None);
        assert_eq!(
            remote.metadata().recording_started_at(),
            SystemTime::UNIX_EPOCH
        );
    }
}
