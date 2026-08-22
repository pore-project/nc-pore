//! Provider-neutral metadata carried with synchronization work.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactTransferMetadata {
    display_name: Option<String>,
    recorded_at: Option<String>,
}

impl ArtifactTransferMetadata {
    pub fn new(display_name: Option<String>, recorded_at: Option<String>) -> Self {
        Self { display_name, recorded_at }
    }

    pub fn display_name(&self) -> Option<&str> { self.display_name.as_deref() }
    pub fn recorded_at(&self) -> Option<&str> { self.recorded_at.as_deref() }
}
