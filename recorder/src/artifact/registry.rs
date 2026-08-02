//! Local Artifact Registry.
//!
//! The registry tracks locally known Recording Artifacts independently from
//! persistence and storage implementations.
//!
//! See:
//! - ADR-047 Local Artifact Registry and Discovery Strategy

/// Technical reference entry for a locally known artifact.
///
/// The registry stores knowledge about artifacts,
/// not the artifact data itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactRegistryEntry {
    pub artifact_id: String,
    pub recording_session_id: String,
}

impl ArtifactRegistryEntry {
    pub fn new(artifact_id: impl Into<String>, recording_session_id: impl Into<String>) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            recording_session_id: recording_session_id.into(),
        }
    }
}

/// Technical registry for discovering local artifacts.
///
/// The registry intentionally does not store recording data.
/// It maintains references needed by recovery and consistency processes.
pub struct LocalArtifactRegistry {
    entries: Vec<ArtifactRegistryEntry>,
}

impl LocalArtifactRegistry {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn register(&mut self, entry: ArtifactRegistryEntry) {
        self.entries.push(entry);
    }

    pub fn find(&self, artifact_id: &str) -> Option<ArtifactRegistryEntry> {
        self.entries
            .iter()
            .find(|entry| entry.artifact_id == artifact_id)
            .cloned()
    }

    pub fn list(&self) -> Vec<ArtifactRegistryEntry> {
        self.entries.clone()
    }

    pub fn remove(&mut self, artifact_id: &str) {
        self.entries
            .retain(|entry| entry.artifact_id != artifact_id);
    }
}

impl Default for LocalArtifactRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-16
    //
    // Protects ADR-047:
    // The registry provides a separate discovery layer for local artifacts.
    #[test]
    fn registry_can_register_artifact() {
        let mut registry = LocalArtifactRegistry::new();

        registry.register(ArtifactRegistryEntry::new("artifact-001", "session-001"));

        assert_eq!(registry.list().len(), 1);
    }

    // TEST-17
    //
    // Protects ADR-047:
    // Artifacts can be discovered independently from persistence.
    #[test]
    fn registry_can_find_artifact() {
        let mut registry = LocalArtifactRegistry::new();

        registry.register(ArtifactRegistryEntry::new("artifact-001", "session-001"));

        let entry = registry.find("artifact-001");

        assert!(entry.is_some());
        assert_eq!(entry.unwrap().artifact_id, "artifact-001");
    }

    // TEST-18
    //
    // Protects ADR-047:
    // Registry entries can be removed without affecting artifact storage.
    #[test]
    fn registry_can_remove_artifact() {
        let mut registry = LocalArtifactRegistry::new();

        registry.register(ArtifactRegistryEntry::new("artifact-001", "session-001"));

        registry.remove("artifact-001");

        assert!(registry.find("artifact-001").is_none());
    }
}
