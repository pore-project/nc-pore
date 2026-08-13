//! Audio capture boundary.
//!
//! This module defines the boundary between the recorder workflow
//! and concrete audio capture implementations.
//!
//! It intentionally does not contain:
//! - audio backend implementations
//! - hardware access
//! - platform-specific code
//! - file encoding logic
//!
//! See:
//! - ADR-039 Recording Architecture and Capture Boundary

mod result;

pub use result::CaptureResult;

#[cfg(test)]
pub use result::{CaptureChunk, CaptureTrack};

/// Defines the interface between recorder workflow
/// and audio capture implementations.
///
/// Concrete implementations may use different audio
/// technologies without affecting the recorder architecture.
pub trait CaptureProvider {
    /// Starts audio capture.
    fn start_capture(&mut self);

    /// Stops audio capture and returns the capture result.
    fn stop_capture(&mut self) -> CaptureResult;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCaptureProvider {
        started: bool,
    }

    impl TestCaptureProvider {
        fn new() -> Self {
            Self { started: false }
        }
    }

    impl CaptureProvider for TestCaptureProvider {
        fn start_capture(&mut self) {
            self.started = true;
        }

        fn stop_capture(&mut self) -> CaptureResult {
            self.started = false;

            CaptureResult::new("test-capture")
        }
    }

    // TEST-06
    // Verify: Capture implementations can follow the defined boundary.
    //
    // This protects ADR-039:
    // Recorder workflow remains independent from
    // concrete audio technology.
    #[test]
    fn capture_provider_can_start_and_stop_capture() {
        let mut capture = TestCaptureProvider::new();

        capture.start_capture();

        assert!(capture.started);

        let result = capture.stop_capture();

        assert_eq!(result.id(), "test-capture");
        assert!(!capture.started);
    }
}
