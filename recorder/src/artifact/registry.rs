//! Local Artifact Registry.
//!
//! The registry tracks locally known Recording Artifacts independently from
//! persistence and storage implementations.
//!
//! See:
//! - ADR-047 Local Artifact Registry and Discovery Strategy

use crate::artifact::RecordingArtifact;

/// Technical registry for discovering and tracking local artifacts.
///
/// The registry intentionally does not store media data.
/// It maintains artifact references and metadata needed by
/// recovery and consistency processes.
pub struct LocalArtifactRegistry {
    artifacts: Vec<RecordingArtifact>,
}

impl LocalArtifactRegistry {
    pub fn new() -> Self {
        Self {
            artifacts: Vec::new(),
        }
    }

    pub fn register(&mut self, artifact: RecordingArtifact) {
        self.artifacts.push(artifact);
    }

    pub fn find(&self, id: &str) -> Option<RecordingArtifact> {
        self.artifacts
            .iter()
            .find(|artifact| artifact.id == id)
            .cloned()
    }

    pub fn list(&self) -> Vec<RecordingArtifact> {
        self.artifacts.clone()
    }

    pub fn remove(&mut self, id: &str) {
        self.artifacts.retain(|artifact| artifact.id != id);
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

        registry.register(RecordingArtifact::new("artifact-001", "session-001"));

        assert_eq!(registry.list().len(), 1);
    }

    // TEST-17
    //
    // Protects ADR-047:
    // Artifacts can be discovered independently from persistence.
    #[test]
    fn registry_can_find_artifact() {
        let mut registry = LocalArtifactRegistry::new();

        registry.register(RecordingArtifact::new("artifact-001", "session-001"));

        let artifact = registry.find("artifact-001");

        assert!(artifact.is_some());
        assert_eq!(artifact.unwrap().id, "artifact-001");
    }

    // TEST-18
    //
    // Protects ADR-047:
    // Registry entries can be removed without affecting artifact storage.
    #[test]
    fn registry_can_remove_artifact() {
        let mut registry = LocalArtifactRegistry::new();

        registry.register(RecordingArtifact::new("artifact-001", "session-001"));

        registry.remove("artifact-001");

        assert!(registry.find("artifact-001").is_none());
    }
}
