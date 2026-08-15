//! CPAL-based audio capture discovery.
//!
//! This module provides the technical bridge to CPAL and exposes
//! the input capabilities of the selected capture device.
//!
//! It intentionally does not yet:
//! - select a recording format
//! - apply fallback policy
//! - convert audio formats
//! - define recording policy
//!
//! Format selection belongs to a later recording implementation
//! step once the required recording format has been specified.

use crate::audio::{CaptureChunk, CaptureResult, CaptureTrack, RecordingConfiguration};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// One input configuration range reported by CPAL.
///
/// The type deliberately belongs to the concrete CPAL provider.
/// It describes technical backend capabilities and is not part of
/// the backend-independent recording configuration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CpalInputConfiguration {
    channels: u16,
    min_sample_rate_hz: u32,
    max_sample_rate_hz: u32,
    sample_format: cpal::SampleFormat,
}

impl CpalInputConfiguration {
    fn from_supported_config(config: &cpal::SupportedStreamConfigRange) -> Self {
        Self {
            channels: config.channels(),
            min_sample_rate_hz: config.min_sample_rate().0,
            max_sample_rate_hz: config.max_sample_rate().0,
            sample_format: config.sample_format(),
        }
    }

    pub const fn channels(&self) -> u16 {
        self.channels
    }

    pub const fn min_sample_rate_hz(&self) -> u32 {
        self.min_sample_rate_hz
    }

    pub const fn max_sample_rate_hz(&self) -> u32 {
        self.max_sample_rate_hz
    }

    pub const fn sample_format(&self) -> cpal::SampleFormat {
        self.sample_format
    }
}

/// Discovers the default input device and exposes its supported
/// input configuration ranges.
impl CpalCaptureProvider {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }

    /// Returns the input configuration ranges supported by the
    /// default input device.
    ///
    /// This is capability discovery only. It does not select a
    /// configuration for a RecordingConfiguration and does not
    /// start an audio stream.
    pub fn discover_input_configurations(&self) -> Result<Vec<CpalInputConfiguration>, String> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;

        let configurations = device
            .supported_input_configs()
            .map_err(|error| {
                format!("Unterstützte Eingabekonfigurationen konnten nicht gelesen werden: {error}")
            })?
            .map(|configuration| CpalInputConfiguration::from_supported_config(&configuration))
            .collect();

        Ok(configurations)
    }
}

pub struct CpalCaptureProvider {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

impl crate::audio::CaptureProvider for CpalCaptureProvider {
    fn start_capture(&mut self, _configuration: &RecordingConfiguration) {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("Kein Standard-Eingabegerät gefunden.");

        let configuration = device
            .default_input_config()
            .expect("Standard-Eingabekonfiguration konnte nicht gelesen werden.");

        let stream_config: cpal::StreamConfig = configuration.clone().into();
        let samples = Arc::clone(&self.samples);

        let stream = device
            .build_input_stream::<f32, _, _>(
                stream_config,
                move |data, _| {
                    let mut samples = samples.lock().unwrap();
                    samples.extend_from_slice(data);
                },
                |_error| {},
                None,
            )
            .expect("Input-Stream konnte nicht erstellt werden.");

        stream
            .play()
            .expect("Audio-Stream konnte nicht gestartet werden.");

        self.stream = Some(stream);
    }

    fn stop_capture(&mut self) -> CaptureResult {
        self.stream.take();

        let payload = self
            .samples
            .lock()
            .expect("Sample-Puffer konnte nicht gelesen werden.")
            .iter()
            .flat_map(|sample| sample.to_le_bytes())
            .collect::<Vec<u8>>();

        let chunk = CaptureChunk::with_payload(1, payload);

        let mut track = CaptureTrack::new("cpal-track");
        track.add_chunk(chunk);

        let mut result = CaptureResult::new("cpal-capture");
        result.add_track(track);

        result
    }
}

pub fn test_input_stream() -> Result<(), String> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;

    println!("Test-Eingabegerät: {}", device);

    let configuration = device.default_input_config().map_err(|error| {
        format!("Standard-Eingabekonfiguration konnte nicht gelesen werden: {error}")
    })?;

    println!(
        "Test-Konfiguration: {} Kanal, {} Hz, {:?}",
        configuration.channels(),
        configuration.sample_rate(),
        configuration.sample_format(),
    );

    let samples = Arc::new(Mutex::new(Vec::<f32>::new()));

    let stream_config: cpal::StreamConfig = configuration.clone().into();

    let samples_for_callback = Arc::clone(&samples);

    let stream = device
        .build_input_stream::<f32, _, _>(
            stream_config,
            move |data, _| {
                let mut samples = samples_for_callback.lock().unwrap();
                samples.extend_from_slice(data);
            },
            |_error| {},
            None,
        )
        .map_err(|error| format!("Input-Stream konnte nicht erstellt werden: {error}"))?;

    stream
        .play()
        .map_err(|error| format!("Audio-Stream konnte nicht gestartet werden: {error}"))?;

    println!("Input-Stream erfolgreich gestartet.");
    std::thread::sleep(std::time::Duration::from_secs(1));

    let count = samples
        .lock()
        .map_err(|_| "Sample-Puffer konnte nicht gelesen werden.".to_string())?
        .len();

    println!("Empfangene Samples: {count}");

    let payload = samples
        .lock()
        .map_err(|_| "Sample-Puffer konnte nicht gelesen werden.".to_string())?
        .iter()
        .flat_map(|sample| sample.to_le_bytes())
        .collect::<Vec<u8>>();

    let chunk = CaptureChunk::with_payload(1, payload);

    println!("CaptureChunk erzeugt: {} Bytes", chunk.payload().len());

    let mut track = CaptureTrack::new("test-track");
    track.add_chunk(chunk);

    println!("CaptureTrack erzeugt: {} Chunk", track.chunks().len());

    let mut result = CaptureResult::new("test-capture");
    result.add_track(track);

    println!("CaptureResult erzeugt: {} Track", result.tracks().len());

    Ok(())
}

pub fn inspect_default_input_device() -> Result<(), String> {
    let host = cpal::default_host();

    let device = host
        .default_input_device()
        .ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;

    println!("Standard-Eingabegerät: {}", device);

    let configuration = device.default_input_config().map_err(|error| {
        format!("Standard-Eingabekonfiguration konnte nicht gelesen werden: {error}")
    })?;

    println!(
        "Standard-Konfiguration: {} Kanal, {} Hz, {:?}",
        configuration.channels(),
        configuration.sample_rate(),
        configuration.sample_format(),
    );

    Ok(())
}
