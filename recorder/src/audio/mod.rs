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

mod cpal;
mod result;

pub use cpal::CpalCaptureProvider;
pub use cpal::inspect_default_input_device;
pub use cpal::test_input_stream;
pub use result::CaptureResult;

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

