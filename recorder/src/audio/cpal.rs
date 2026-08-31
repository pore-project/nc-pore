//! CPAL-based audio capture.
//!
//! CPAL is an implementation detail of the Capture Boundary. The provider
//! discovers the selected device's native capabilities, delegates policy to
//! `native_selection`, and starts a stream using the selected native format.
//! No resampling or bit-depth expansion is performed here.

use crate::audio::{
    CaptureChunk, CaptureProvider, CaptureResult, CaptureStartError, CaptureTrack,
    NativeAudioCapability, NativeSampleFormat, RecordingConfiguration, SampleFormat, SyncSignet,
    SyncSignetEmissionError, SyncSignetKind, select_best_native_capture,
};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

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

    pub const fn channels(&self) -> u16 { self.channels }
    pub const fn min_sample_rate_hz(&self) -> u32 { self.min_sample_rate_hz }
    pub const fn max_sample_rate_hz(&self) -> u32 { self.max_sample_rate_hz }
    pub const fn sample_format(&self) -> cpal::SampleFormat { self.sample_format }

    pub fn matches_recording_configuration(&self, configuration: &RecordingConfiguration) -> bool {
        let Some(capability) = self.native_capability() else { return false; };
        capability.channels() == configuration.channels()
            && capability.min_sample_rate_hz() <= configuration.sample_rate_hz()
            && configuration.sample_rate_hz() <= capability.max_sample_rate_hz()
            && capability.sample_format().as_recording_format() == configuration.sample_format()
    }

    fn native_capability(&self) -> Option<NativeAudioCapability> {
        let format = match self.sample_format {
            cpal::SampleFormat::I16 => NativeSampleFormat::Pcm16,
            cpal::SampleFormat::I24 => NativeSampleFormat::Pcm24,
            cpal::SampleFormat::F32 => NativeSampleFormat::F32,
            _ => return None,
        };
        Some(NativeAudioCapability::new(
            self.channels,
            self.min_sample_rate_hz,
            self.max_sample_rate_hz,
            format,
        ))
    }

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

fn native_capabilities(configurations: &[CpalInputConfiguration]) -> Vec<NativeAudioCapability> {
    configurations.iter().filter_map(CpalInputConfiguration::native_capability).collect()
}

fn find_configuration_for_selection(
    selection: &crate::audio::NativeCaptureConfiguration,
    configurations: &[CpalInputConfiguration],
) -> Option<CpalInputConfiguration> {
    configurations.iter().copied().find(|configuration| {
        configuration.native_capability() == Some(selection.capability())
            && configuration.min_sample_rate_hz <= selection.sample_rate_hz()
            && selection.sample_rate_hz() <= configuration.max_sample_rate_hz
    })
}

struct PendingSignet { offset_bytes: usize, payload: Vec<u8> }

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
            SampleFormat::Pcm16 => 2,
            SampleFormat::Pcm24 => 3,
            SampleFormat::F32 => 4,
        };
        let frames = configuration.chunk_duration().seconds() as usize * configuration.sample_rate_hz() as usize;
        let chunk_size_bytes = frames * bytes_per_sample * usize::from(configuration.channels());
        Self { chunks: Vec::new(), current_payload: Vec::new(), next_sequence: 1, chunk_size_bytes,
            sample_format: configuration.sample_format(), channels: configuration.channels(),
            sample_rate_hz: configuration.sample_rate_hz(), captured_bytes: 0, pending_signets: Vec::new() }
    }

    fn request_signet(&mut self, signet: SyncSignet) {
        self.pending_signets.push(PendingSignet { offset_bytes: self.captured_bytes,
            payload: render_signet(signet, self.sample_rate_hz, self.channels, self.sample_format) });
    }

    fn push_bytes(&mut self, bytes: &[u8]) {
        if self.chunk_size_bytes == 0 { return; }
        self.captured_bytes += bytes.len();
        let mut remaining = bytes;
        while !remaining.is_empty() {
            let available = self.chunk_size_bytes - self.current_payload.len();
            let take = available.min(remaining.len());
            self.current_payload.extend_from_slice(&remaining[..take]);
            remaining = &remaining[take..];
            if self.current_payload.len() == self.chunk_size_bytes { self.finish_current_chunk(); }
        }
    }

    fn finish_current_chunk(&mut self) {
        if self.current_payload.is_empty() { return; }
        let payload = std::mem::take(&mut self.current_payload);
        self.chunks.push(CaptureChunk::with_payload(self.next_sequence, payload));
        self.next_sequence += 1;
    }

    fn finish(mut self) -> Vec<CaptureChunk> {
        self.finish_current_chunk();
        let mut result = Vec::with_capacity(self.chunks.len());
        let mut chunk_offset = 0usize;
        for chunk in self.chunks {
            let start = chunk_offset; let end = start + chunk.payload().len();
            let mut payload = chunk.payload().to_vec();
            for signet in &self.pending_signets {
                mix_signet_into_chunk(&mut payload, start, end, signet.offset_bytes, &signet.payload, self.sample_format);
            }
            chunk_offset = end;
            result.push(CaptureChunk::with_payload(chunk.sequence, payload));
        }
        result
    }
}

fn render_signet(signet: SyncSignet, rate: u32, channels: u16, format: SampleFormat) -> Vec<u8> {
    let bps = match format { SampleFormat::Pcm16 => 2, SampleFormat::Pcm24 => 3, SampleFormat::F32 => 4 };
    let frames = (u64::from(signet.duration_ms()) * u64::from(rate) / 1000) as usize;
    let mut payload = Vec::with_capacity(frames * usize::from(channels) * bps);
    let mut state = match signet.kind() { SyncSignetKind::Opening => 0x1357_9bdf, SyncSignetKind::Closing => 0x2468_ace1 };
    for frame in 0..frames {
        let ms = frame as u64 * 1000 / u64::from(rate);
        let active = signet.events().iter().any(|event| ms >= u64::from(event.start_ms()) && ms < u64::from(event.start_ms() + event.duration_ms()));
        let sample = if active {
            state ^= state << 13; state ^= state >> 17; state ^= state << 5;
            ((state as f32 / u32::MAX as f32) * 2.0 - 1.0) * 0.12 * match signet.kind() { SyncSignetKind::Opening => 1.0, SyncSignetKind::Closing => -1.0 }
        } else { 0.0 };
        for _ in 0..channels { match format {
            SampleFormat::F32 => payload.extend_from_slice(&sample.to_ne_bytes()),
            SampleFormat::Pcm24 => payload.extend_from_slice(&encode_i24(sample)),
            SampleFormat::Pcm16 => { let value = (sample.clamp(-1.0, 1.0) * 32767.0).round() as i16; payload.extend_from_slice(&value.to_ne_bytes()); }
        }}
    }
    payload
}

fn encode_i24(sample: f32) -> [u8; 3] {
    let value = (sample.clamp(-1.0, 1.0) * 8_388_607.0).round() as i32;
    let bytes = value.to_ne_bytes(); [bytes[0], bytes[1], bytes[2]]
}

fn mix_signet_into_chunk(payload: &mut [u8], chunk_start: usize, chunk_end: usize, signet_offset: usize,
    signet_payload: &[u8], format: SampleFormat) {
    let bps = match format { SampleFormat::Pcm16 => 2, SampleFormat::Pcm24 => 3, SampleFormat::F32 => 4 };
    let overlap_start = chunk_start.max(signet_offset); let overlap_end = chunk_end.min(signet_offset + signet_payload.len());
    if overlap_start >= overlap_end { return; }
    let aligned_start = overlap_start + (bps - (overlap_start - signet_offset) % bps) % bps;
    let aligned_end = overlap_end - (overlap_end - signet_offset) % bps;
    for absolute in (aligned_start..aligned_end).step_by(bps) {
        let po = absolute - chunk_start; let so = absolute - signet_offset;
        match format {
            SampleFormat::F32 => { let a = f32::from_ne_bytes(payload[po..po + 4].try_into().unwrap()); let b = f32::from_ne_bytes(signet_payload[so..so + 4].try_into().unwrap()); payload[po..po + 4].copy_from_slice(&(a + b).clamp(-1.0, 1.0).to_ne_bytes()); }
            SampleFormat::Pcm24 => { let a = decode_i24(&payload[po..po + 3]); let b = decode_i24(&signet_payload[so..so + 3]); payload[po..po + 3].copy_from_slice(&encode_i24_sample(a.saturating_add(b))); }
            SampleFormat::Pcm16 => { let a = i16::from_ne_bytes(payload[po..po + 2].try_into().unwrap()) as i32; let b = i16::from_ne_bytes(signet_payload[so..so + 2].try_into().unwrap()) as i32; let value = a.saturating_add(b).clamp(i32::from(i16::MIN), i32::from(i16::MAX)) as i16; payload[po..po + 2].copy_from_slice(&value.to_ne_bytes()); }
        }
    }
}

fn encode_i24_sample(value: i32) -> [u8; 3] { let bytes = value.clamp(-8_388_608, 8_388_607).to_ne_bytes(); [bytes[0], bytes[1], bytes[2]] }
fn decode_i24(bytes: &[u8]) -> i32 { i32::from_ne_bytes([bytes[0], bytes[1], bytes[2], if bytes[2] & 0x80 != 0 { 0xff } else { 0 }]) }

pub struct CpalCaptureProvider {
    chunk_buffer: Arc<Mutex<Option<CaptureChunkBuffer>>>,
    capture_error: Arc<Mutex<Option<String>>>,
    stream: Option<cpal::Stream>,
    active_configuration: Option<RecordingConfiguration>,
}

impl CpalCaptureProvider {
    pub fn new() -> Self { Self { chunk_buffer: Arc::new(Mutex::new(None)), capture_error: Arc::new(Mutex::new(None)), stream: None, active_configuration: None } }

    pub fn discover_input_configurations(&self) -> Result<Vec<CpalInputConfiguration>, String> {
        let device = cpal::default_host().default_input_device().ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;
        device.supported_input_configs().map_err(|e| format!("Unterstützte Eingabekonfigurationen konnten nicht gelesen werden: {e}")).map(|configs| configs.map(|c| CpalInputConfiguration::from_supported_config(&c)).collect())
    }

    pub fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> {
        self.chunk_buffer.lock().unwrap().as_mut().ok_or(SyncSignetEmissionError::NotCapturing)?.request_signet(*signet); Ok(())
    }
}

impl CaptureProvider for CpalCaptureProvider {
    fn start_capture(&mut self, requested: &RecordingConfiguration) -> Result<(), CaptureStartError> {
        if self.stream.is_some() { return Err(CaptureStartError::AlreadyCapturing); }
        let device = cpal::default_host().default_input_device().ok_or(CaptureStartError::DeviceUnavailable)?;
        let configs: Vec<_> = device.supported_input_configs().map_err(|_| CaptureStartError::ConfigurationUnavailable)?.map(|c| CpalInputConfiguration::from_supported_config(&c)).collect();
        let capabilities = native_capabilities(&configs);
        let selection = select_best_native_capture(requested, &capabilities).ok_or(CaptureStartError::UnsupportedRecordingConfiguration)?;
        let concrete = find_configuration_for_selection(&selection, &configs).ok_or(CaptureStartError::UnsupportedRecordingConfiguration)?;
        let format = match selection.sample_format() { NativeSampleFormat::Pcm16 => SampleFormat::Pcm16, NativeSampleFormat::Pcm24 => SampleFormat::Pcm24, NativeSampleFormat::F32 => SampleFormat::F32 };
        let actual = RecordingConfiguration::with_chunk_duration(selection.sample_rate_hz(), selection.channels(), format, requested.chunk_duration());
        let stream_config = concrete.stream_config_for_sample_rate(selection.sample_rate_hz()).ok_or(CaptureStartError::UnsupportedRecordingConfiguration)?;
        *self.chunk_buffer.lock().unwrap() = Some(CaptureChunkBuffer::new(&actual)); *self.capture_error.lock().unwrap() = None;
        let buffer = Arc::clone(&self.chunk_buffer); let error = Arc::clone(&self.capture_error);
        let stream = device.build_input_stream_raw(stream_config, concrete.sample_format(), move |data, _| { if let Some(b) = buffer.lock().unwrap().as_mut() { b.push_bytes(data.bytes()); } }, move |e| { if error.lock().unwrap().is_none() { *error.lock().unwrap() = Some(e.to_string()); } }, None).map_err(|_| CaptureStartError::ConfigurationUnavailable)?;
        stream.play().map_err(|_| CaptureStartError::ConfigurationUnavailable)?;
        self.active_configuration = Some(actual); self.stream = Some(stream); Ok(())
    }

    fn emit_sync_signet(&mut self, signet: &SyncSignet) -> Result<(), SyncSignetEmissionError> { self.emit_sync_signet(signet) }

    fn stop_capture(&mut self) -> CaptureResult {
        self.stream.take(); let configuration = self.active_configuration.take().expect("Keine aktive Aufnahmekonfiguration vorhanden.");
        let chunks = self.chunk_buffer.lock().unwrap().take().map(CaptureChunkBuffer::finish).unwrap_or_default();
        let mut track = CaptureTrack::with_configuration("cpal-track", configuration); for chunk in chunks { track.add_chunk(chunk); }
        let error = self.capture_error.lock().unwrap().take(); let mut result = match error { Some(e) => CaptureResult::failed("cpal-capture", e), None => CaptureResult::new("cpal-capture") }; result.add_track(track); result
    }
}

pub fn test_input_stream() -> Result<(), String> { inspect_default_input_device() }

pub fn inspect_default_input_device() -> Result<(), String> {
    let device = cpal::default_host().default_input_device().ok_or_else(|| "Kein Standard-Eingabegerät gefunden.".to_string())?;
    println!("Standard-Eingabegerät: {device}");
    let configuration = device.default_input_config().map_err(|e| e.to_string())?;
    println!("Standard-Konfiguration: {} Kanal, {} Hz, {:?}", configuration.channels(), configuration.sample_rate(), configuration.sample_format());
    Ok(())
}
