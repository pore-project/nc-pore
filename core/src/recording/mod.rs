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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_recording_starts_as_prepared() {
        let recording = Recording::new();

        assert_eq!(recording.status(), RecordingStatus::Prepared);
    }

    #[test]
    fn recording_can_transition_to_recording() {
        let mut recording = Recording::new();

        recording.start();

        assert_eq!(recording.status(), RecordingStatus::Recording);
    }

    #[test]
    fn recording_can_be_completed() {
        let mut recording = Recording::new();

        recording.start();
        recording.complete();

        assert_eq!(recording.status(), RecordingStatus::Completed);
    }
}
