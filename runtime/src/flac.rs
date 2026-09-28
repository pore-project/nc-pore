use claxon::FlacReader;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

pub const OPERATION_CONVERT_FLAC_TO_WAV: &str = "artifact.convert_flac_to_wav";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConvertFlacToWavRequest {
    pub protocol_version: u16,
    pub operation: String,
    pub request_id: String,
    pub input_path: String,
    pub output_path: String,
    pub expected_sample_rate_hz: u32,
    pub expected_channels: u16,
    pub expected_bits_per_sample: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConvertFlacToWavResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub status: String,
    pub sample_rate_hz: Option<u32>,
    pub channels: Option<u16>,
    pub bits_per_sample: Option<u16>,
    pub sample_count: Option<u64>,
    pub payload_length: Option<u64>,
    pub error_code: Option<String>,
}

pub fn handle_convert_flac_to_wav(request: &ConvertFlacToWavRequest) -> ConvertFlacToWavResponse {
    let response = ConvertFlacToWavResponse {
        protocol_version: 1,
        request_id: request.request_id.clone(),
        status: "rejected".to_owned(),
        sample_rate_hz: None,
        channels: None,
        bits_per_sample: None,
        sample_count: None,
        payload_length: None,
        error_code: None,
    };

    if request.protocol_version != 1 {
        return ConvertFlacToWavResponse {
            error_code: Some("unsupported_protocol_version".to_owned()),
            ..response
        };
    }
    if request.operation != OPERATION_CONVERT_FLAC_TO_WAV {
        return ConvertFlacToWavResponse {
            error_code: Some("unsupported_operation".to_owned()),
            ..response
        };
    }
    if request.input_path.trim().is_empty() || request.output_path.trim().is_empty() {
        return ConvertFlacToWavResponse {
            error_code: Some("invalid_path".to_owned()),
            ..response
        };
    }

    match convert_flac_to_wav(
        Path::new(&request.input_path),
        Path::new(&request.output_path),
        request.expected_sample_rate_hz,
        request.expected_channels,
        request.expected_bits_per_sample,
    ) {
        Ok(result) => ConvertFlacToWavResponse {
            protocol_version: 1,
            request_id: request.request_id.clone(),
            status: "converted".to_owned(),
            sample_rate_hz: Some(result.sample_rate_hz),
            channels: Some(result.channels),
            bits_per_sample: Some(result.bits_per_sample),
            sample_count: Some(result.sample_count),
            payload_length: Some(result.payload_length),
            error_code: None,
        },
        Err(error) => ConvertFlacToWavResponse {
            error_code: Some(error),
            ..response
        },
    }
}

pub const V1_BITS_PER_SAMPLE: u16 = 24;
pub const V1_CHANNELS: u16 = 1;
pub const V1_SAMPLE_RATE: u32 = 48_000;
pub const V1_FALLBACK_SAMPLE_RATE: u32 = 44_100;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlacToWavResult {
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub sample_count: u64,
    pub payload_length: u64,
}

pub fn convert_flac_to_wav(
    input_path: &Path,
    output_path: &Path,
    expected_sample_rate_hz: u32,
    expected_channels: u16,
    expected_bits_per_sample: u16,
) -> Result<FlacToWavResult, String> {
    if !input_path.is_file() {
        return Err("flac_input_missing".to_owned());
    }
    if expected_channels != V1_CHANNELS || expected_bits_per_sample != V1_BITS_PER_SAMPLE {
        return Err("flac_output_profile_unsupported".to_owned());
    }
    if expected_sample_rate_hz != 0
        && !matches!(
            expected_sample_rate_hz,
            V1_SAMPLE_RATE | V1_FALLBACK_SAMPLE_RATE
        )
    {
        return Err("flac_sample_rate_unsupported".to_owned());
    }

    let mut reader =
        FlacReader::open(input_path).map_err(|error| format!("flac_decode_open:{error}"))?;
    let info = reader.streaminfo();
    let channels =
        u16::try_from(info.channels).map_err(|_| "flac_channel_count_invalid".to_string())?;
    let bits_per_sample =
        u16::try_from(info.bits_per_sample).map_err(|_| "flac_bit_depth_invalid".to_string())?;
    if channels != expected_channels
        || bits_per_sample != expected_bits_per_sample
        || !matches!(info.sample_rate, V1_SAMPLE_RATE | V1_FALLBACK_SAMPLE_RATE)
        || (expected_sample_rate_hz != 0 && info.sample_rate != expected_sample_rate_hz)
    {
        return Err("flac_streaminfo_mismatch".to_owned());
    }

    let output = OpenOptions::new()
        .create(true)
        .truncate(true)
        .read(true)
        .write(true)
        .open(output_path)
        .map_err(|error| format!("wav_output_open:{error}"))?;
    let mut output = BufWriter::new(output);
    write_wav_header(
        &mut output,
        info.sample_rate,
        V1_CHANNELS,
        V1_BITS_PER_SAMPLE,
        0,
    )
    .map_err(|error| format!("wav_header_write:{error}"))?;

    let mut sample_count = 0_u64;
    let mut pcm_bytes = 0_u64;
    let mut buffer = Vec::with_capacity(64 * 1024);

    for sample in reader.samples() {
        let sample = sample.map_err(|error| format!("flac_decode_sample:{error}"))?;
        if !(-8_388_608..=8_388_607).contains(&sample) {
            return cleanup_failed_output(output, output_path, "flac_sample_out_of_range");
        }

        let value = sample as u32;
        buffer.push((value & 0xff) as u8);
        buffer.push(((value >> 8) & 0xff) as u8);
        buffer.push(((value >> 16) & 0xff) as u8);
        sample_count = sample_count
            .checked_add(1)
            .ok_or_else(|| "wav_sample_count_overflow".to_owned())?;
        pcm_bytes = pcm_bytes
            .checked_add(3)
            .ok_or_else(|| "wav_payload_size_overflow".to_owned())?;

        if buffer.len() >= 64 * 1024 {
            output
                .write_all(&buffer)
                .map_err(|error| format!("wav_payload_write:{error}"))?;
            buffer.clear();
        }
    }

    if !buffer.is_empty() {
        output
            .write_all(&buffer)
            .map_err(|error| format!("wav_payload_write:{error}"))?;
    }

    let payload_length = 44_u64
        .checked_add(pcm_bytes)
        .ok_or_else(|| "wav_file_size_overflow".to_owned())?;
    if payload_length > u64::from(u32::MAX) + 8 {
        return cleanup_failed_output(output, output_path, "wav_file_too_large");
    }

    output
        .flush()
        .map_err(|error| format!("wav_flush:{error}"))?;
    output
        .seek(SeekFrom::Start(0))
        .map_err(|error| format!("wav_seek:{error}"))?;
    write_wav_header(
        &mut output,
        info.sample_rate,
        V1_CHANNELS,
        V1_BITS_PER_SAMPLE,
        pcm_bytes,
    )
    .map_err(|error| format!("wav_header_rewrite:{error}"))?;
    output
        .flush()
        .map_err(|error| format!("wav_final_flush:{error}"))?;
    let mut file = output
        .into_inner()
        .map_err(|error| format!("wav_finalize:{error}"))?;
    let actual_length = file
        .seek(SeekFrom::End(0))
        .map_err(|error| format!("wav_size:{error}"))?;

    if actual_length != payload_length {
        let error = format!("wav_size_mismatch:{actual_length}:{payload_length}");
        let _ = std::fs::remove_file(output_path);
        return Err(error);
    }

    Ok(FlacToWavResult {
        sample_rate_hz: info.sample_rate,
        channels,
        bits_per_sample,
        sample_count,
        payload_length,
    })
}

fn cleanup_failed_output(
    mut output: BufWriter<std::fs::File>,
    output_path: &Path,
    error: &str,
) -> Result<FlacToWavResult, String> {
    let _ = output.flush();
    let _ = std::fs::remove_file(output_path);
    Err(error.to_owned())
}

fn write_wav_header<W: Write>(
    output: &mut W,
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    data_length: u64,
) -> io::Result<()> {
    let block_align = channels
        .checked_mul(bits_per_sample / 8)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "block align overflow"))?;
    let byte_rate = sample_rate
        .checked_mul(u32::from(block_align))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "byte rate overflow"))?;
    let riff_length = 36_u64
        .checked_add(data_length)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "RIFF length overflow"))?;
    let data_length_u32 = u32::try_from(data_length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "data length overflow"))?;
    let riff_length_u32 = u32::try_from(riff_length)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "RIFF length overflow"))?;

    output.write_all(b"RIFF")?;
    output.write_all(&riff_length_u32.to_le_bytes())?;
    output.write_all(b"WAVEfmt ")?;
    output.write_all(&16_u32.to_le_bytes())?;
    output.write_all(&1_u16.to_le_bytes())?;
    output.write_all(&channels.to_le_bytes())?;
    output.write_all(&sample_rate.to_le_bytes())?;
    output.write_all(&byte_rate.to_le_bytes())?;
    output.write_all(&block_align.to_le_bytes())?;
    output.write_all(&bits_per_sample.to_le_bytes())?;
    output.write_all(b"data")?;
    output.write_all(&data_length_u32.to_le_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_real_v1_flac_fixture_to_canonical_wav() {
        let root =
            std::env::temp_dir().join(format!("pore-flac-test-{}", std::process::id()));
        std::fs::create_dir_all(&root).unwrap();
        let input = root.join("fixture.flac");
        let output = root.join("fixture.wav");
        let fixture = include_bytes!("../tests/fixtures/v1-24bit-mono-48khz.flac");
        std::fs::write(&input, fixture).unwrap();

        let result = convert_flac_to_wav(&input, &output, 48_000, 1, 24).unwrap();
        assert_eq!(result.sample_rate_hz, 48_000);
        assert_eq!(result.channels, 1);
        assert_eq!(result.bits_per_sample, 24);
        assert_eq!(result.sample_count, 3);
        assert_eq!(result.payload_length, 53);

        let bytes = std::fs::read(&output).unwrap();
        assert_eq!(bytes.len(), 53);
        assert_eq!(&bytes[44..], &[0, 0, 0, 0xff, 0xff, 0x7f, 0, 0, 0x80]);

        let _ = std::fs::remove_file(&input);
        let _ = std::fs::remove_file(&output);
        let _ = std::fs::remove_dir(&root);
    }

    #[test]
    fn wav_header_matches_pore_24_bit_mono_shape() {
        let mut bytes = Vec::new();
        write_wav_header(&mut bytes, 48_000, 1, 24, 9).unwrap();
        assert_eq!(bytes.len(), 44);
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(u32::from_le_bytes(bytes[16..20].try_into().unwrap()), 16);
        assert_eq!(u16::from_le_bytes(bytes[20..22].try_into().unwrap()), 1);
        assert_eq!(u16::from_le_bytes(bytes[22..24].try_into().unwrap()), 1);
        assert_eq!(
            u32::from_le_bytes(bytes[24..28].try_into().unwrap()),
            48_000
        );
        assert_eq!(
            u32::from_le_bytes(bytes[28..32].try_into().unwrap()),
            144_000
        );
        assert_eq!(u16::from_le_bytes(bytes[32..34].try_into().unwrap()), 3);
        assert_eq!(u16::from_le_bytes(bytes[34..36].try_into().unwrap()), 24);
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 9);
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 45);
    }
}
