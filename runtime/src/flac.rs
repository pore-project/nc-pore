use claxon::FlacReader;
use std::fs::OpenOptions;
use std::io::{self, BufWriter, Seek, SeekFrom, Write};
use std::path::Path;

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
    if !matches!(expected_sample_rate_hz, V1_SAMPLE_RATE | V1_FALLBACK_SAMPLE_RATE) {
        return Err("flac_sample_rate_unsupported".to_owned());
    }

    let mut reader = FlacReader::open(input_path)
        .map_err(|error| format!("flac_decode_open:{error}"))?;
    let info = reader.streaminfo();
    let channels = u16::from(info.channels);
    let bits_per_sample = u16::from(info.bits_per_sample);
    if channels != expected_channels
        || bits_per_sample != expected_bits_per_sample
        || info.sample_rate != expected_sample_rate_hz
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
        assert_eq!(u32::from_le_bytes(bytes[24..28].try_into().unwrap()), 48_000);
        assert_eq!(u32::from_le_bytes(bytes[28..32].try_into().unwrap()), 144_000);
        assert_eq!(u16::from_le_bytes(bytes[32..34].try_into().unwrap()), 3);
        assert_eq!(u16::from_le_bytes(bytes[34..36].try_into().unwrap()), 24);
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(u32::from_le_bytes(bytes[40..44].try_into().unwrap()), 9);
        assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 45);
    }
}
