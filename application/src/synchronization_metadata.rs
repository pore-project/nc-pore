//! Provider-neutral metadata carried with synchronization work.
//!
//! This type deliberately contains semantic values only. Date/time formatting,
//! folder layout and provider-specific naming remain connector concerns.

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ArtifactTransferMetadata {
    display_name: Option<String>,
    recorded_at: Option<String>,
}

impl ArtifactTransferMetadata {
    pub fn new(display_name: Option<String>, recorded_at: Option<String>) -> Self {
        Self {
            display_name,
            recorded_at,
        }
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }

    pub fn recorded_at(&self) -> Option<&str> {
        self.recorded_at.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-META-01: metadata remains opaque and provider-neutral at the
    // application boundary.
    #[test]
    fn metadata_preserves_semantic_values() {
        let metadata = ArtifactTransferMetadata::new(
            Some("Interview mit Frizz".to_owned()),
            Some("2026-08-22T18:30:00+02:00".to_owned()),
        );

        assert_eq!(metadata.display_name(), Some("Interview mit Frizz"));
        assert_eq!(metadata.recorded_at(), Some("2026-08-22T18:30:00+02:00"));
    }

    // TEST-META-02: missing optional metadata is a valid state; connectors
    // can apply their own deterministic fallback policy.
    #[test]
    fn metadata_can_be_absent() {
        let metadata = ArtifactTransferMetadata::default();

        assert_eq!(metadata.display_name(), None);
        assert_eq!(metadata.recorded_at(), None);
    }
}
