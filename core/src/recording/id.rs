//! Identity of a domain Recording.
//!
//! RecordingId is intentionally distinct from identifiers of
//! production sessions and technical recording artifacts.
//!
//! See ADR-054 Recording Artifact and Local Recording Data Association.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordingId(String);

impl RecordingId {
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

    // TEST-01
    //
    // Protects the RecordingId boundary:
    // recording identity is represented by a dedicated value object.
    #[test]
    fn recording_id_preserves_value() {
        let id = RecordingId::new("recording-001");

        assert_eq!(id.value(), "recording-001");
    }
}
