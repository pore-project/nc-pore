//! Persistence Provider Interface.
//!
//! This module defines the boundary between the Recorder workflow
//! and concrete persistence implementations.
//!
//! The interface intentionally contains no storage technology details.
//! Concrete decisions such as filesystem layout, databases or cloud
//! storage are handled by separate implementations.

use crate::artifact::RecordingArtifact;
use crate::persistence::PersistenceLoadResult;

/// Persistence contract used by the Recorder workflow.
///
/// The workflow depends on this abstraction instead of concrete storage.
/// This keeps persistence replaceable and prevents storage concerns from
/// leaking into recording logic.
pub trait PersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact);

    /// Assesses the persisted representation of one artifact.
    ///
    /// The boundary deliberately distinguishes a valid artifact from
    /// incomplete or inconsistent persisted data. This prevents recovery
    /// code from treating every existing artifact directory as valid.
    fn load(&self, id: &str) -> PersistenceLoadResult;

    /// Lists identifiers for persisted artifact candidates without loading
    /// their artifact representations.
    ///
    /// Recovery uses this to enumerate candidates and then delegates the
    /// actual consistency assessment to `load`.
    fn list_ids(&self) -> Vec<String>;

    fn list(&self) -> Vec<RecordingArtifact>;

    fn remove(&mut self, id: &str);
}
