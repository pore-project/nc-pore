//! Recording domain model.
//!
//! This module contains domain concepts related to recordings.
//!
//! It intentionally does not contain:
//! - audio backend access
//! - file handling
//! - hardware interaction
//! - synchronization logic
//!
//! See:
//! - ADR-039 Recording Architecture and Capture Boundary (future)
//! - ADR-038 Core Implementation Structure and Module Organization

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingStatus {
    Prepared,
    Recording,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    status: RecordingStatus,
}

impl Recording {
    pub fn new() -> Self {
        Self {
            status: RecordingStatus::Prepared,
        }
    }

    pub fn status(&self) -> RecordingStatus {
        self.status
    }

    pub fn start(&mut self) {
        self.status = RecordingStatus::Recording;
    }

    pub fn complete(&mut self) {
        self.status = RecordingStatus::Completed;
    }
}

impl Default for Recording {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-11
    // Verify: A new recording starts in Prepared state.
    //
    // Recording lifecycle:
    // Prepared -> Recording -> Completed
    #[test]
    fn new_recording_starts_as_prepared() {
        let recording = Recording::new();

        assert_eq!(recording.status(), RecordingStatus::Prepared);
    }

    // TEST-12
    // Verify: A prepared recording can transition into Recording state.
    //
    // Protects the recording lifecycle model.
    #[test]
    fn recording_can_transition_to_recording() {
        let mut recording = Recording::new();

        recording.start();

        assert_eq!(recording.status(), RecordingStatus::Recording);
    }

    // TEST-13
    // Verify: A recording can transition from Recording to Completed state.
    //
    // Lifecycle:
    // Prepared -> Recording -> Completed
    #[test]
    fn recording_can_be_completed() {
        let mut recording = Recording::new();

        recording.start();
        recording.complete();

        assert_eq!(recording.status(), RecordingStatus::Completed);
    }

    // TEST-14
    // Verify: Default construction creates the same initial state as new().
    //
    // Protects the default initialization contract.
    #[test]
    fn default_recording_starts_as_prepared() {
        let recording = Recording::default();

        assert_eq!(recording.status(), RecordingStatus::Prepared);
    }
}
