//! Result of a completed audio capture operation.
//!
//! CaptureResult represents the technical outcome
//! of a capture operation.
//!
//! It intentionally does not contain:
//! - artifact lifecycle rules
//! - persistence logic
//! - synchronization logic

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureResult {
    id: String,
}

impl CaptureResult {
    /// Creates a new capture result.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into() }
    }

    /// Returns the identifier of the capture result.
    pub fn id(&self) -> &str {
        &self.id
    }
}
