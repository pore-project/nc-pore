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
    SampleFormat, SyncSignet, SyncSignetEmissionError, SyncSignetKind,
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
    pub fn stream_config_for_sample_rate(&self, sample_rate_hz: u32) -> Option<cpal::StreamConfig> {
        let range = cpal::SupportedStreamConfigRange::new(
            self.channels,
            self.min_sample_rate_hz,
            self.max_sample_rate_hz,
            self.buffer_size,
            self.sample_format,
        );

        range.try_with_sample_rate(sample_rate_hz).map(Into::into)
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

struct PendingSignet {
    offset_bytes: usize,
    payload: Vec<u8>,
}

/// Runtime state used by the capture callback to split the incoming
/// audio stream into configured-duration chunks without stopping capture.
struct CaptureChunkBuffer {
    chunks: Vec<CaptureChunk>,
    current_payload: Vec<u8>,
    next_sequence: u32,
    chunk_size_bytes: usize,
    sample_format: SampleFormat,
    channels: u16,
    sample_rate_hz: u32,
    captured_bytes: usize,
    pending_signets: Vec<PendingSignet>,
}

impl CaptureChunkBuffer {
    fn new(configuration: &RecordingConfiguration) -> Self {
        let bytes_per_sample = match configuration.sample_format() {
            SampleFormat::Pcm24 => 3,
            SampleFormat::F32 => 4,
        };
        let frames_per_chunk = configuration
            .chunk_duration()
            .seconds()
            .saturating_mul(configuration.sample_rate_hz());
        let bytes_per_frame = bytes_per_sample * usize::from(configuration.channels());
        let chunk_size_bytes = usize::try_from(frames_per_chunk)
            .expect("Chunkgröße überschreitet die lokale Speicherkapazität.")
            .checked_mul(bytes_per_frame)
            .expect("Chunkgröße überschreitet die lokale Speicherkapazität.");

        Self {
            chunks: Vec::new(),
            current_payload: Vec::new(),
            next_sequence: 1,
            chunk_size_bytes,
            sample_format: configuration.sample_format(),
            channels: configuration.channels(),
            sample_rate_hz: configuration.sample_rate_hz(),
            captured_bytes: 0,
            pending_signets: Vec::new(),
        }
    }

    fn request_signet(&mut self, signet: SyncSignet) {
        let payload = render_signet(
            signet,
            self.sample_rate_hz,
            self.channels,
            self.sample_format,
        );

        self.pending_signets.push(PendingSignet {
            offset_bytes: self.captured_bytes,
            payload,
        });
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        if self.chunk_size_bytes == 0 {
            return;
        }

        self.captured_bytes += bytes.len();

        let mut remaining = bytes;
        while !remaining.is_empty() {
            let available = self.chunk_size_bytes - self.current_payload.len();
            let take = available.min(remaining.len());
            self.current_payload.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];

            if self.current_payload.len() == self.chunk_size_bytes {
                self.finish_current_chunk();
            }
        }
    }

    fn finish_current_chunk(&mut self) {
        if self.current_payload.is_empty() {
            return;
        }

        let payload = std::mem::take(&mut self.current_payload);
        let sequence = self.next_sequence;
        self.next_sequence += 1;
        self.chunks
            .push(CaptureChunk::with_payload(sequence, payload));
    }

    fn finish(mut self) -> Vec<CaptureChunk> {
        self.finish_current_chunk();

        let mut chunks = Vec::with_capacity(self.chunks.len());
        let mut chunk_offset = 0usize;

        for chunk in self.chunks {
            let chunk_start = chunk_offset;
            let chunk_end = chunk_start + chunk.payload().len();
            let mut payload = chunk.payload().to_vec();

            for signet in &self.pending_signets {
                mix_signet_into_chunk(
                    &mut payload,
                    chunk_start,
                    chunk_end,
                    signet.offset_bytes,
                    &signet.payload,
                    self.sample_format,
                );
            }

            chunk_offset = chunk_end;
            chunks.push(CaptureChunk::with_payload(chunk.sequence, payload));
        }

        chunks
    }
}

fn render_signet(
    signet: SyncSignet,
    sample_rate_hz: u32,
    channels: u16,
    sample_format: SampleFormat,
) -> Vec<u8> {
    let bytes_per_sample = match sample_format {
        SampleFormat::Pcm24 => 3,
        SampleFormat::F32 => 4,
    };
    let channels = usize::from(channels);
    let total_frames =
        usize::try_from(u64::from(signet.duration_ms()) * u64::from(sample_rate_hz) / 1_000)
            .expect("Signetdauer überschreitet die lokale Speicherkapazität.");
    let mut payload = Vec::with_capacity(
        total_frames
            .saturating_mul(channels)
            .saturating_mul(bytes_per_sample),
    );
    let mut state = match signet.kind() {
        SyncSignetKind::Opening => 0x1357_9bdf,
        SyncSignetKind::Closing => 0x2468_ace1,
    };

    for frame in 0..total_frames {
        let time_ms = frame as u64 * 1_000 / u64::from(sample_rate_hz);
        let active = signet.events().iter().any(|event| {
            time_ms >= u64::from(event.start_ms())
                && time_ms < u64::from(event.start_ms() + event.duration_ms())
        });
        let sample = if active {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let normalized = (state as f32 / u32::MAX as f32) * 2.0 - 1.0;
            let polarity = match signet.kind() {
                SyncSignetKind::Opening => 1.0,
                SyncSignetKind::Closing => -1.0,
            };
            normalized * 0.12 * polarity
        } else {
            0.0
        };

        for _ in 0..channels {
            match sample_format {
                SampleFormat::F32 => payload.extend_from_slice(&sample.to_ne_bytes()),
                SampleFormat::Pcm24 => payload.extend_from_slice(&encode_i24(sample)),
            }
        }
    }

    payload
}

fn mix_signet_into_chunk(
    payload: &mut [u8],
    chunk_start: usize,
    chunk_end: usize,
    signet_offset: usize,
    signet_payload: &[u8],
    sample_format: SampleFormat,
) {
    let bytes_per_sample = match sample_format {
        SampleFormat::Pcm24 => 3,
        SampleFormat::F32 => 4,
    };
    let signet_end = signet_offset + signet_payload.len();
    let overlap_start = chunk_start.max(signet_offset);
    let overlap_end = chunk_end.min(signet_end);

    if overlap_start >= overlap_end {
        return;
    }

    let aligned_start = overlap_start
        + (bytes_per_sample - (overlap_start - signet_offset) % bytes_per_sample)
            % bytes_per_sample;
    let aligned_end = overlap_end - (overlap_end - signet_offset) % bytes_per_sample;

    if aligned_start >= aligned_end {
        return;
    }

    for absolute_offset in (aligned_start..aligned_end).step_by(bytes_per_sample) {
        let payload_offset = absolute_offset - chunk_start;
        let signet_payload_offset = absolute_offset - signet_offset;

        match sample_format {
            SampleFormat::F32 => {
                let input = f32::from_ne_bytes(
                    payload[payload_offset..payload_offset + 4]
                        .try_into()
                        .expect("F32-Sample muss vier Bytes enthalten."),
                );
                let signet = f32::from_ne_bytes(
                    signet_payload[signet_payload_offset..signet_payload_offset + 4]
                        .try_into()
                        .expect("F32-Signet muss vier Bytes enthalten."),
                );
                payload[payload_offset..payload_offset + 4]
                    .copy_from_slice(&(input + signet).clamp(-1.0, 1.0).to_ne_bytes());
            }
            SampleFormat::Pcm24 => {
                let input = decode_i24(&payload[payload_offset..payload_offset + 3]);
                let signet =
                    decode_i24(&signet_payload[signet_payload_offset..signet_payload_offset + 3]);
                payload[payload_offset..payload_offset + 3]
                    .copy_from_slice(&encode_i24_sample(input.saturating_add(signet)));
            }
        }
    }
}

fn encode_i24(sample: f32) -> [u8; 3] {
    let value = (sample.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
    encode_i24_sample(value)
}

fn encode_i24_sample(value: i32) -> [u8; 3] {
    let value = value.clamp(-8_388_608, 8_388_607);
    let bytes = value.to_ne_bytes();
    [bytes[0], bytes[1], bytes[2]]
}

fn decode_i24(bytes: &[u8]) -> i32 {
    let sign = if bytes[2] & 0x80 != 0 { 0xff } else { 0x00 };
    i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], sign])
}

pub struct CpalCaptureProvider {
    chunk_buffer: Arc<Mutex<Option<CaptureChunkBuffer>>>,
    capture_error: Arc<Mutex<Option<String>>>,
    stream: Option<cpal::Stream>,
    active_configuration: Option<RecordingConfiguration>,
}

/// Discovers the default input device and exposes its supported
/// input configuration ranges.
impl CpalCaptureProvider {
    pub fn new() -> Self {
        Self {
            chunk_buffer: Arc::new(Mutex::new(None)),
            capture_error: Arc::new(Mutex::new(None)),
            stream: None,
            active_configuration: None,
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

    /// Queues a synchronization signet at the current capture position.
    ///
    /// The signet is mixed into the captured sample stream when the
    /// capture result is finalized. This keeps the realtime CPAL callback
    /// free of waveform generation while preserving the byte position at
    /// which the synchronization event was requested.
    pub fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
        let mut buffer = self
            .chunk_buffer
            .lock()
            .expect("Chunk-Puffer konnte nicht für das Signet gesperrt werden.");
        let buffer = buffer
            .as_mut()
            .ok_or(SyncSignetEmissionError::NotCapturing)?;

        buffer.request_signet(*signet);
        Ok(())
    }
}

impl crate::audio::CaptureProvider for CpalCaptureProvider {
    fn start_capture(
        &mut self,
        configuration: &RecordingConfiguration,
    ) -> Result<(), CaptureStartError> {
        if self.stream.is_some() || self.active_configuration.is_some() {
            return Err(CaptureStartError::AlreadyCapturing);
        }

        *self
            .chunk_buffer
            .lock()
            .expect("Chunk-Puffer konnte nicht initialisiert werden.") =
            Some(CaptureChunkBuffer::new(configuration));
        *self
            .capture_error
            .lock()
            .expect("Capture-Fehlerzustand konnte nicht initialisiert werden.") = None;

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(CaptureStartError::DeviceUnavailable)?;

        let capabilities = device
            .supported_input_configs()
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?
            .map(|configuration| CpalInputConfiguration::from_supported_config(&configuration))
            .collect::<Vec<_>>();

        let selected_configuration =
            require_exact_input_configuration(configuration, &capabilities)?;

        let stream_config = selected_configuration
            .stream_config_for_sample_rate(configuration.sample_rate_hz())
            .ok_or(CaptureStartError::UnsupportedRecordingConfiguration)?;
        let chunk_buffer = Arc::clone(&self.chunk_buffer);
        let capture_error = Arc::clone(&self.capture_error);

        let stream = device
            .build_input_stream_raw(
                stream_config,
                selected_configuration.sample_format(),
                move |data, _| {
                    let mut chunk_buffer = chunk_buffer.lock().unwrap();
                    if let Some(chunk_buffer) = chunk_buffer.as_mut() {
                        chunk_buffer.push_bytes(data.bytes());
                    }
                },
                move |error| {
                    let mut capture_error = capture_error.lock().unwrap();
                    if capture_error.is_none() {
                        *capture_error = Some(error.to_string());
                    }
                },
                None,
            )
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?;

        stream
            .play()
            .map_err(|_| CaptureStartError::ConfigurationUnavailable)?;

        self.active_configuration = Some(*configuration);
        self.stream = Some(stream);

        Ok(())
    }

    fn emit_sync_signet(
        &mut self,
        signet: &SyncSignet,
    ) -> Result<(), SyncSignetEmissionError> {
        CpalCaptureProvider::emit_sync_signet(self, signet)
    }

    fn stop_capture(&mut self) -> CaptureResult {
        self.stream.take();

        let configuration = self
            .active_configuration
            .take()
            .expect("Keine aktive Aufnahmekonfiguration vorhanden.");

        let chunks = self
            .chunk_buffer
            .lock()
            .expect("Chunk-Puffer konnte nicht gelesen werden.")
            .take()
            .map(CaptureChunkBuffer::finish)
            .unwrap_or_default();

        let mut track = CaptureTrack::with_configuration("cpal-track", configuration);
        for chunk in chunks {
            track.add_chunk(chunk);
        }

        let capture_error = self
            .capture_error
            .lock()
            .expect("Capture-Fehlerzustand konnte nicht gelesen werden.")
            .take();

        let mut result = match capture_error {
            Some(error) => CaptureResult::failed("cpal-capture", error),
            None => CaptureResult::new("cpal-capture"),
        };
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

    let samples = Arc::new(Mutex::new(Vec::<u8>::new()));

    let stream_config: cpal::StreamConfig = configuration.clone().into();

    let samples_for_callback = Arc::clone(&samples);

    let stream = device
        .build_input_stream_raw(
            stream_config,
            configuration.sample_format(),
            move |data, _| {
                let mut samples = samples_for_callback.lock().unwrap();
                samples.extend_from_slice(data.bytes());
            },
            |error| {
                eprintln!("CPAL Input-Stream-Fehler: {error}");
            },
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

    println!("Empfangene Sample-Bytes: {count}");

    let payload = samples
        .lock()
        .map_err(|_| "Sample-Puffer konnte nicht gelesen werden.".to_string())?
        .clone();

    let chunk = CaptureChunk::with_payload(1, payload);

    println!("CaptureChunk erzeugt: {} Bytes", chunk.payload().len());

    let mut track = CaptureTrack::with_configuration(
        "test-track",
        RecordingConfiguration::new(
            configuration.sample_rate(),
            configuration.channels(),
            SampleFormat::F32,
        ),
    );
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
    use crate::audio::CaptureProvider;

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

    // TEST-01
    #[test]
    fn exact_match_accepts_rate_inside_supported_range() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(available.matches_recording_configuration(&requested));
    }

    // TEST-02
    #[test]
    fn exact_match_rejects_wrong_channel_count() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let available = capability(2, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(!available.matches_recording_configuration(&requested));
    }

    // TEST-03
    #[test]
    fn exact_match_maps_pcm24_to_cpal_i24() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm24);
        let available = capability(1, 48_000, 48_000, cpal::SampleFormat::I24);

        assert!(available.matches_recording_configuration(&requested));
    }

    // TEST-04
    #[test]
    fn resolver_returns_none_without_exact_match() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm24);
        let capabilities = [capability(2, 48_000, 48_000, cpal::SampleFormat::I24)];

        assert_eq!(
            find_exact_input_configuration(&requested, &capabilities),
            None
        );
    }

    // TEST-05
    #[test]
    fn unsupported_configuration_returns_start_error() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let capabilities = [capability(2, 48_000, 48_000, cpal::SampleFormat::F32)];

        assert_eq!(
            require_exact_input_configuration(&requested, &capabilities),
            Err(CaptureStartError::UnsupportedRecordingConfiguration)
        );
    }

    // TEST-06
    #[test]
    fn supported_configuration_passes_start_validation() {
        let requested = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let capabilities = [capability(1, 48_000, 48_000, cpal::SampleFormat::F32)];

        assert_eq!(
            require_exact_input_configuration(&requested, &capabilities),
            Ok(capabilities[0])
        );
    }

    // TEST-07
    #[test]
    fn stream_config_uses_requested_sample_rate() {
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        let stream_config = available
            .stream_config_for_sample_rate(48_000)
            .expect("48 kHz should be supported");

        assert_eq!(stream_config.channels, 1);
        assert_eq!(stream_config.sample_rate, 48_000);
    }

    // TEST-08
    #[test]
    fn stream_config_rejects_rate_outside_supported_range() {
        let available = capability(1, 44_100, 96_000, cpal::SampleFormat::F32);

        assert!(available.stream_config_for_sample_rate(96_001).is_none());
    }

    // TEST-09
    #[test]
    fn stop_capture_returns_final_partial_chunk() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            10,
            1,
            SampleFormat::F32,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        buffer.push_bytes(&[0x01, 0x02, 0x03]);

        let chunks = buffer.finish();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].sequence, 1);
        assert_eq!(chunks[0].payload(), &[0x01, 0x02, 0x03]);
    }

    // TEST-10
    #[test]
    fn runtime_chunking_splits_at_configured_duration() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            2,
            1,
            SampleFormat::F32,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        let chunk_size = 2 * 10 * 4;
        let payload = vec![0x01; chunk_size + 3];

        buffer.push_bytes(&payload);
        let chunks = buffer.finish();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].sequence, 1);
        assert_eq!(chunks[0].payload().len(), chunk_size);
        assert_eq!(chunks[1].sequence, 2);
        assert_eq!(chunks[1].payload(), &[0x01, 0x01, 0x01]);
    }

    // TEST-11
    #[test]
    fn runtime_chunking_preserves_sequence_across_callbacks() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            1,
            1,
            SampleFormat::F32,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        buffer.push_bytes(&vec![0x01; 20]);
        buffer.push_bytes(&vec![0x02; 20]);

        let chunks = buffer.finish();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].sequence, 1);
        assert_eq!(chunks[0].payload().len(), 40);
    }

    // TEST-12
    #[test]
    fn runtime_chunking_keeps_multiple_channels_in_frame_sized_chunks() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            2,
            2,
            SampleFormat::Pcm24,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        let chunk_size = 10 * 2 * 3 * 2;
        let payload = vec![0x01; chunk_size + 6];

        buffer.push_bytes(&payload);
        let chunks = buffer.finish();

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].payload().len(), chunk_size);
        assert_eq!(chunks[1].payload().len(), 6);
    }

    // TEST-13
    #[test]
    fn stop_capture_clears_runtime_chunk_state_for_next_capture() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            1,
            1,
            SampleFormat::F32,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        buffer.push_bytes(&[0x01, 0x02]);
        let first = buffer.finish();

        let mut next_buffer = CaptureChunkBuffer::new(&configuration);
        let second = next_buffer.finish();

        assert_eq!(first.len(), 1);
        assert_eq!(first[0].sequence, 1);
        assert!(second.is_empty());
    }

    // TEST-14 / CUE30
    // Verify: A requested Opening Sync Signet is mixed into the captured
    // payload at the byte position where the signet was requested.
    #[test]
    fn opening_signet_is_present_at_requested_capture_position() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            48_000,
            1,
            SampleFormat::F32,
            crate::audio::RecordingChunkDuration::TenSeconds,
        );
        let mut buffer = CaptureChunkBuffer::new(&configuration);
        buffer.push_bytes(&vec![0; 4 * 100]);
        buffer.request_signet(SyncSignet::opening());
        buffer.push_bytes(&vec![0; 4 * 400]);

        let chunks = buffer.finish();
        let payload = chunks[0].payload();

        assert!(payload[400..]
            .chunks_exact(4)
            .any(|sample| f32::from_ne_bytes(sample.try_into().unwrap()).abs() > 0.0));
        assert!(payload[..400]
            .chunks_exact(4)
            .all(|sample| f32::from_ne_bytes(sample.try_into().unwrap()) == 0.0));
    }

    // TEST-15 / CUE30
    // Verify: Opening and Closing signets remain distinguishable in the
    // concrete capture representation.
    #[test]
    fn opening_and_closing_signet_payloads_are_distinct() {
        let opening = render_signet(SyncSignet::opening(), 48_000, 1, SampleFormat::F32);
        let closing = render_signet(SyncSignet::closing(), 48_000, 1, SampleFormat::F32);

        assert_eq!(opening.len(), closing.len());
        assert_ne!(opening, closing);
    }

    // TEST-16 / CUE30
    // Verify: A signet request made while capture is inactive is rejected.
    #[test]
    fn signet_emission_requires_active_capture() {
        let mut provider = CpalCaptureProvider::new();

        assert_eq!(
            provider.emit_sync_signet(&SyncSignet::opening()),
            Err(SyncSignetEmissionError::NotCapturing)
        );
    }
}
