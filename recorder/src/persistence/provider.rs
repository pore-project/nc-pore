//! Persistence Provider Interface.
//!
//! This module defines the boundary between the Recorder workflow
//! and concrete persistence implementations.
//!
//! The interface intentionally contains no storage technology details.
//! Concrete decisions such as filesystem layout, databases or cloud
//! storage are handled by separate implementations.

use crate::artifact::RecordingArtifact;

/// Persistence contract used by the Recorder workflow.
///
/// The workflow depends on this abstraction instead of concrete storage.
/// This keeps persistence replaceable and prevents storage concerns from
/// leaking into recording logic.
pub trait PersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact);

    fn load(&self, id: &str) -> Option<RecordingArtifact>;

    fn list(&self) -> Vec<RecordingArtifact>;

    fn remove(&mut self, id: &str);
}
