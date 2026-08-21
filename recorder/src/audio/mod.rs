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
//! - ADR-068 Recording Start and Audio Synchronization Signet

mod configuration;
mod cpal;
mod result;
mod signet;

pub use configuration::{RecordingChunkDuration, RecordingConfiguration, SampleFormat};
pub use cpal::inspect_default_input_device;
pub use cpal::test_input_stream;
pub use cpal::{CpalCaptureProvider, CpalInputConfiguration};
pub use result::{CaptureChunk, CaptureTrack};
pub use result::{CaptureResult, CaptureStatus};
pub use signet::{SignetEvent, SyncSignet, SyncSignetKind};

/// Returned when a capture provider cannot start audio capture.
#[derive(Debug, PartialEq, Eq)]
pub enum CaptureStartError {
    DeviceUnavailable,
    ConfigurationUnavailable,
    UnsupportedRecordingConfiguration,
    AlreadyCapturing,
}

/// Returned when a synchronization signet cannot be emitted into a capture.
#[derive(Debug, PartialEq, Eq)]
pub enum SyncSignetEmissionError {
    NotCapturing,
    Unsupported,
}

/// Defines the interface between recorder workflow
/// and concrete audio capture implementations.
///
/// Concrete implementations may use different audio
/// technologies without affecting the recorder architecture.
pub trait CaptureProvider {
    /// Starts audio capture using the requested recording configuration.
    fn start_capture(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError>;

    /// Emits a synchronization signet into the active capture.
    ///
    /// The default implementation keeps existing non-audio test providers
    /// source-compatible while concrete capture backends opt into signet
    /// injection explicitly.
    fn emit_sync_signet(&mut self, _signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
        Err(SyncSignetEmissionError::Unsupported)
    }

    /// Stops audio capture and returns the capture result.
    fn stop_capture(&mut self) -> CaptureResult;
}
