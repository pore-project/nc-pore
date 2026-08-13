//! Persistence assessment results.
//!
//! The persistence boundary distinguishes between an artifact that can be
//! restored as a valid artifact and persisted data that must not be treated
//! as valid merely because an artifact directory exists.
//!
//! See ADR-053 Artifact Recovery and Consistency Boundary.

use crate::artifact::RecordingArtifact;

/// Result of assessing one persisted RecordingArtifact.
#[derive(Debug, Clone)]
pub enum PersistenceLoadResult {
    /// The persisted representation is complete and internally consistent.
    Valid(RecordingArtifact),
    /// Persisted data is structurally incomplete and cannot be restored yet.
    Incomplete,
    /// Persisted data is present but internally inconsistent or invalid.
    Inconsistent,
    /// No persisted artifact exists for the requested identifier.
    NotFound,
}
