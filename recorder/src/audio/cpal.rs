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

use crate::audio::{CaptureChunk, CaptureResult, CaptureTrack};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

/// Discovers the default input device and prints its supported
/// input configurations.
///
/// This is currently a technical integration probe for CPAL.
/// It does not yet participate in the CaptureProvider boundary.
impl CpalCaptureProvider {
    pub fn new() -> Self {
        Self {
            samples: Arc::new(Mutex::new(Vec::new())),
            stream: None,
        }
    }
}

pub struct CpalCaptureProvider {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
}

impl crate::audio::CaptureProvider for CpalCaptureProvider {
    fn start_capture(&mut self) {
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
