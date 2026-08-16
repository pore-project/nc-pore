//! CPAL-based audio capture discovery.
//!
//! This module provides the technical bridge to CPAL and exposes
//! the input capabilities of the selected capture device.
//!
//! It intentionally does not yet:
//! - apply fallback policy
//! - convert audio formats
//! - define recording policy
//!
//! Format selection belongs to a later recording implementation
//! step once the required recording format has been specified.

use crate::audio::{
    CaptureChunk, CaptureResult, CaptureStartError, CaptureTrack, RecordingConfiguration,
    SampleFormat,
};
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
    buffer_size: cpal::SupportedBufferSize,
}

impl CpalInputConfiguration {
    fn from_supported_config(config: &cpal::SupportedStreamConfigRange) -> Self {
        Self {
            channels: config.channels(),
            min_sample_rate_hz: config.min_sample_rate(),
            max_sample_rate_hz: config.max_sample_rate(),
            sample_format: config.sample_format(),
            buffer_size: *config.buffer_size(),
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

    /// Returns whether this CPAL capability exactly supports the
    /// requested recording configuration.
    pub fn matches_recording_configuration(&self, configuration: &RecordingConfiguration) -> bool {
        self.channels == configuration.channels()
            && self.min_sample_rate_hz <= configuration.sample_rate_hz()
            && configuration.sample_rate_hz() <= self.max_sample_rate_hz
            && self.sample_format == cpal_sample_format(configuration.sample_format())
    }

    /// Returns a concrete CPAL stream configuration for a supported
    /// sample rate.
    ///
    /// The selection is delegated to CPAL's range API so the native
    /// buffer-size information is retained as reported by the device.
    pub fn stream_config_for_sample_rate(
        &self,
        sample_rate_hz: u32,
    ) -> Option<cpal::StreamConfig> {
        let range = cpal::SupportedStreamConfigRange::new(
            self.channels,
            self.min_sample_rate_hz,
            self.max_sample_rate_hz,
            self.buffer_size,
            self.sample_format,
        );

        range
            .try_with_sample_rate(sample_rate_hz)
            .map(Into::into)
    }
}

fn cpal_sample_format(format: SampleFormat) -> cpal::SampleFormat {
    match format {
        SampleFormat::Pcm24 => cpal::SampleFormat::I24,
        SampleFormat::F32 => cpal::SampleFormat::F32,
    }
}

/// Finds the first exact native match for the requested recording
/// configuration.
///
/// No fallback, conversion, prioritization, or resampling policy is
/// applied. If no capability matches all requested parameters, the
/// result is `None`.
pub fn find_exact_input_configuration(
    requested: &RecordingConfiguration,
    capabilities: &[CpalInputConfiguration],
) -> Option<CpalInputConfiguration> {
    capabilities
        .iter()
        .copied()
        .find(|capability| capability.matches_recording_configuration(requested))
}

fn require_exact_input_configuration(
    requested: &RecordingConfiguration,
    capabilities: &[CpalInputConfiguration],
) -> Result<CpalInputConfiguration, CaptureStartError> {
    find_exact_input_configuration(requested, capabilities)
        .ok_or(CaptureStartError::UnsupportedRecordingConfiguration)
}

pub struct CpalCaptureProvider {
    samples: Arc<Mutex<Vec<f32>>>,
    stream: Option<cpal::Stream>,
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

impl crate::audio::CaptureProvider for CpalCaptureProvider {
    fn start_capture(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError> {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(CaptureStartError::DeviceUnavailable)?;

        let capabilities = device
            .supported_input_configs()
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?
            .map(|configuration| CpalInputConfiguration::from_supported_config(&configuration))
            .collect::<Vec<_>>();

        let _selected_configuration =
            require_exact_input_configuration(configuration, &capabilities)?;

        let configuration = device
            .default_input_config()
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?;

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
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?;

        stream
            .play()
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?;

        self.stream = Some(stream);

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn capability(
        channels: u16,
        min_sample_rate_hz: u32,
        max_sample_rate_hz: u32,
        sample_format: cpal::SampleFormat,
    ) -> CpalInputConfiguration {
        CpalInputConfiguration {
            channels,
            min_sample_rate_hz,
            max_sample_rate_hz,
            sample_format,
            buffer_size: cpal::SupportedBufferSize::Unknown,
        }
    }

    #[test]
    fn exact_match_accepts_rate_inside_supported_range() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(available.matches_recording_configuration(&requested));
    }

    #[test]
    fn exact_match_rejects_wrong_channel_count() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let available = capability(2, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(!available.matches_recording_configuration(&requested));
    }

    #[test]
    fn exact_match_maps_pcm24_to_cpal_i24() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm24);
        let available = capability(1, 48_000, 48_000, cpal::SampleFormat::I24);

        assert!(available.matches_recording_configuration(&requested));
    }

    #[test]
    fn resolver_returns_none_without_exact_match() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm24);
        let capabilities = [capability(2, 48_000, 48_000, cpal::SampleFormat::I24)];

        assert_eq!(
            find_exact_input_configuration(&requested, &capabilities),
            None
        );
    }

    #[test]
    fn unsupported_configuration_returns_start_error() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let capabilities = [capability(2, 48_000, 48_000, cpal::SampleFormat::F32)];

        assert_eq!(
            require_exact_input_configuration(&requested, &capabilities),
            Err(CaptureStartError::UnsupportedRecordingConfiguration)
        );
    }

    #[test]
    fn supported_configuration_passes_start_validation() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let capabilities = [capability(1, 48_000, 48_000, cpal::SampleFormat::F32)];

        assert_eq!(
            require_exact_input_configuration(&requested, &capabilities),
            Ok(capabilities[0])
        );
    }

    #[test]
    fn stream_config_uses_requested_sample_rate() {
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        let stream_config = available
            .stream_config_for_sample_rate(48_000)
            .expect("48 kHz should be supported");

        assert_eq!(stream_config.channels, 1);
        assert_eq!(stream_config.sample_rate, cpal::SampleRate(48_000));
    }

    #[test]
    fn stream_config_rejects_rate_outside_supported_range() {
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(available.stream_config_for_sample_rate(96_001).is_none());
    }
}
