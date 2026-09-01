//! Lossless FLAC transport encoding for captured PCM.
//!
//! PCM is the capture representation. FLAC is the transport representation:
//! it preserves the captured samples exactly while reducing transfer size.

use flacenc::bitsink::ByteSink;
use flacenc::component::BitRepr;
use flacenc::error::Verify;

use super::{CaptureChunk, RecordingConfiguration, SampleFormat};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FlacEncodeError {
    UnsupportedSampleFormat,
    InvalidPcmPayload,
    Configuration(String),
    Encoding(String),
}

impl std::fmt::Display for FlacEncodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSampleFormat => write!(f, "FLAC transport does not support this sample format"),
            Self::InvalidPcmPayload => write!(f, "PCM payload does not contain complete samples"),
            Self::Configuration(error) => write!(f, "invalid FLAC encoder configuration: {error}"),
            Self::Encoding(error) => write!(f, "FLAC encoding failed: {error}"),
        }
    }
}

impl std::error::Error for FlacEncodeError {}

/// Encodes interleaved captured PCM chunks as one FLAC transport stream.
///
/// The input samples are never resampled or otherwise transformed. Integer
/// PCM is converted to the encoder's i32 representation without changing its
/// sample value. F32 is intentionally rejected here: the native capture
/// selection must choose an integer PCM representation before transport.
pub fn encode_chunks(
    chunks: &[CaptureChunk],
    configuration: RecordingConfiguration,
) -> Result<Vec<u8>, FlacEncodeError> {
    let bits_per_sample = match configuration.sample_format() {
        SampleFormat::Pcm16 => 16,
        SampleFormat::Pcm24 => 24,
        SampleFormat::F32 => return Err(FlacEncodeError::UnsupportedSampleFormat),
    };
    let bytes_per_sample = usize::from(bits_per_sample / 8);
    let channels = usize::from(configuration.channels());
    let frame_width = bytes_per_sample * channels;
    if frame_width == 0 {
        return Err(FlacEncodeError::InvalidPcmPayload);
    }

    let mut samples = Vec::new();
    for chunk in chunks {
        let payload = chunk.payload();
        if payload.len() % frame_width != 0 {
            return Err(FlacEncodeError::InvalidPcmPayload);
        }
        match configuration.sample_format() {
            SampleFormat::Pcm16 => {
                for sample in payload.chunks_exact(2) {
                    samples.push(i16::from_ne_bytes([sample[0], sample[1]]) as i32);
                }
            }
            SampleFormat::Pcm24 => {
                for sample in payload.chunks_exact(3) {
                    let sign = if sample[2] & 0x80 != 0 { 0xff } else { 0 };
                    samples.push(i32::from_ne_bytes([sample[0], sample[1], sample[2], sign]));
                }
            }
            SampleFormat::F32 => unreachable!(),
        }
    }

    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|error| FlacEncodeError::Configuration(error.to_string()))?;
    let source = flacenc::source::MemSource::from_samples(
        &samples,
        configuration.channels(),
        bits_per_sample,
        configuration.sample_rate_hz(),
    );
    let stream = flacenc::encode_with_fixed_block_size(&config, source, config.block_size)
        .map_err(|error| FlacEncodeError::Encoding(error.to_string()))?;
    let mut sink = ByteSink::new();
    stream.write(&mut sink);
    Ok(sink.as_slice().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_pcm16_chunks_as_flac() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut payload = Vec::new();
        for sample in [0i16, 1000, -1000, 32767, -32768] {
            payload.extend_from_slice(&sample.to_ne_bytes());
        }
        let chunks = [CaptureChunk::with_payload(1, payload)];

        let encoded = encode_chunks(&chunks, configuration).expect("FLAC encoding succeeds");

        assert!(encoded.starts_with(b"fLaC"));
        assert!(encoded.len() > 42);
    }

    #[test]
    fn encodes_pcm24_chunks_as_flac() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm24);
        let payload = vec![0, 0, 0, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f];
        let chunks = [CaptureChunk::with_payload(1, payload)];

        let encoded = encode_chunks(&chunks, configuration).expect("FLAC encoding succeeds");

        assert!(encoded.starts_with(b"fLaC"));
    }

    #[test]
    fn rejects_partial_samples() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let chunks = [CaptureChunk::with_payload(1, vec![0])];

        assert_eq!(
            encode_chunks(&chunks, configuration),
            Err(FlacEncodeError::InvalidPcmPayload)
        );
    }

    #[test]
    fn rejects_float_transport() {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let chunks = [CaptureChunk::with_payload(1, vec![0; 4])];

        assert_eq!(
            encode_chunks(&chunks, configuration),
            Err(FlacEncodeError::UnsupportedSampleFormat)
        );
    }
}
