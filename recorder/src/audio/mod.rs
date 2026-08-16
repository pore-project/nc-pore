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
//! - ADR-061 Configurable Recording Configuration

mod configuration;
mod cpal;
mod result;

pub use configuration::{RecordingConfiguration, SampleFormat};
pub use cpal::{CpalCaptureProvider, CpalInputConfiguration};
pub use cpal::inspect_default_input_device;
pub use cpal::test_input_stream;
pub use result::CaptureResult;

pub use result::{CaptureChunk, CaptureTrack};

/// Returned when a capture provider cannot start audio capture.
#[derive(Debug, PartialEq, Eq)]
pub struct CaptureStartError;

/// Defines the interface between recorder workflow
/// and audio capture implementations.
///
/// Concrete implementations may use different audio
/// technologies without affecting the recorder architecture.
pub trait CaptureProvider {
    /// Starts audio capture using the requested recording configuration.
    fn start_capture(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError>;

    /// Stops audio capture and returns the capture result.
    fn stop_capture(&mut self) -> CaptureResult;
}
