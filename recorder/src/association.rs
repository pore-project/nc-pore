//! Association between a technical RecordingArtifact and its originating
//! domain recording context.
//!
//! The recorder crate intentionally does not depend on the core crate.
//! Therefore the domain identifiers cross this boundary as opaque values.
//! The caller at the application boundary is responsible for supplying
//! the ProductionSession/Recording identifiers from the core model.
//!
//! This keeps domain rules in core while preserving traceability in the
//! persisted RecordingArtifact.

use crate::artifact::RecordingArtifact;

/// Opaque reference to the domain recording context that produced an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingArtifactAssociation {
    production_id: String,
    recording_id: String,
}

impl RecordingArtifactAssociation {
    /// Creates an association from the originating production and recording.
    pub fn new(production_id: impl Into<String>, recording_id: impl Into<String>) -> Self {
        Self {
            production_id: production_id.into(),
            recording_id: recording_id.into(),
        }
    }

    /// Returns the originating production identifier as an opaque value.
    pub fn production_id(&self) -> &str {
        &self.production_id
    }

    /// Returns the originating domain recording identifier as an opaque value.
    pub fn recording_id(&self) -> &str {
        &self.recording_id
    }
}

/// Attaches the originating domain association to a recording artifact.
pub fn associate_artifact(
    mut artifact: RecordingArtifact,
    association: RecordingArtifactAssociation,
) -> RecordingArtifact {
    artifact.set_domain_association(association.production_id, association.recording_id);
    artifact
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::RecordingArtifact;
    use crate::session::RecordingSessionId;

    #[test]
    fn association_preserves_originating_identifiers() {
        let association = RecordingArtifactAssociation::new("production-001", "recording-017");

        assert_eq!(association.production_id(), "production-001");
        assert_eq!(association.recording_id(), "recording-017");
    }

    #[test]
    fn association_can_be_attached_to_artifact() {
        let artifact = RecordingArtifact::new(
            "artifact-001",
            RecordingSessionId::new("recording-session-001"),
        );

        let artifact = associate_artifact(
            artifact,
            RecordingArtifactAssociation::new("production-001", "recording-017"),
        );

        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
    }
}
