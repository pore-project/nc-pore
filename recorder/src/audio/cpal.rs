//! CPAL-based audio capture discovery.
//!
//! This module currently provides the technical bridge to CPAL
//! and exposes the available input device configurations.
//!
//! It intentionally does not yet:
//! - select a recording format
//! - start an audio stream
//! - write audio data
//! - define recording policy
//!
//! Format selection belongs to a later recording implementation
//! step once the required recording format has been specified.

use cpal::traits::{DeviceTrait, HostTrait};

/// Discovers the default input device and prints its supported
/// input configurations.
///
/// This is currently a technical integration probe for CPAL.
/// It does not yet participate in the CaptureProvider boundary.
pub fn inspect_default_input_device() -> Result<(), String> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;

    println!("Standard-Eingabegerät: {}", device);

    let configurations = device
        .supported_input_configs()
        .map_err(|error| format!("Eingabekonfigurationen konnten nicht gelesen werden: {error}"))?;

    for configuration in configurations {
        println!(
            "  Kanäle: {}, Sample-Rate: {}–{} Hz, Sample-Format: {:?}",
            configuration.channels(),
            configuration.min_sample_rate(),
            configuration.max_sample_rate(),
            configuration.sample_format(),
        );
    }

    Ok(())
}
