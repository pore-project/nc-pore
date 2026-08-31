//! Native audio capability selection policy.
//!
//! The policy is deliberately independent of a concrete audio backend. A
//! backend reports what it can actually provide; this module chooses the
//! best native representation without resampling or up-conversion.

use super::{RecordingChunkDuration, RecordingConfiguration, SampleFormat};

/// Sample representation exposed by a native capture backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSampleFormat {
    Pcm16,
    Pcm24,
    F32,
}

impl NativeSampleFormat {
    /// Higher means more native sample precision is available.
    pub const fn quality_rank(self) -> u8 {
        match self {
            Self::Pcm16 => 1,
            Self::Pcm24 => 2,
            Self::F32 => 3,
        }
    }

    pub const fn as_recording_format(self) -> Option<SampleFormat> {
        match self {
            Self::Pcm16 => None,
            Self::Pcm24 => Some(SampleFormat::Pcm24),
            Self::F32 => Some(SampleFormat::F32),
        }
    }
}

/// One native capability reported by an audio backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeAudioCapability {
    channels: u16,
    min_sample_rate_hz: u32,
    max_sample_rate_hz: u32,
    sample_format: NativeSampleFormat,
}

impl NativeAudioCapability {
    pub const fn new(
        channels: u16,
        min_sample_rate_hz: u32,
        max_sample_rate_hz: u32,
        sample_format: NativeSampleFormat,
    ) -> Self {
        Self {
            channels,
            min_sample_rate_hz,
            max_sample_rate_hz,
            sample_format,
        }
    }

    pub const fn channels(self) -> u16 {
        self.channels
    }

    pub const fn min_sample_rate_hz(self) -> u32 {
        self.min_sample_rate_hz
    }

    pub const fn max_sample_rate_hz(self) -> u32 {
        self.max_sample_rate_hz
    }

    pub const fn sample_format(self) -> NativeSampleFormat {
        self.sample_format
    }
}

/// The concrete native configuration selected for a recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NativeCaptureConfiguration {
    capability: NativeAudioCapability,
    sample_rate_hz: u32,
    chunk_duration: RecordingChunkDuration,
}

impl NativeCaptureConfiguration {
    pub const fn new(
        capability: NativeAudioCapability,
        sample_rate_hz: u32,
        chunk_duration: RecordingChunkDuration,
    ) -> Self {
        Self {
            capability,
            sample_rate_hz,
            chunk_duration,
        }
    }

    pub const fn capability(self) -> NativeAudioCapability {
        self.capability
    }

    pub const fn sample_rate_hz(self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn channels(self) -> u16 {
        self.capability.channels()
    }

    pub const fn sample_format(self) -> NativeSampleFormat {
        self.capability.sample_format()
    }

    pub const fn chunk_duration(self) -> RecordingChunkDuration {
        self.chunk_duration
    }
}

/// Select the best native capability for a preferred recording profile.
///
/// Selection is deliberately conservative:
///
/// * exact channel count is preferred;
/// * the preferred native sample representation is preferred;
/// * the closest actually supported sample rate is selected;
/// * a higher native rate wins ties over a lower native rate;
/// * higher native sample precision wins when all earlier criteria tie;
/// * native Pcm16 remains a real capability and is never represented as
///   synthetic 24-bit audio.
///
/// The function only chooses a native configuration. It never performs
/// conversion, resampling, channel mixing, or bit-depth expansion.
pub fn select_best_native_capture(
    requested: &RecordingConfiguration,
    capabilities: &[NativeAudioCapability],
) -> Option<NativeCaptureConfiguration> {
    capabilities
        .iter()
        .copied()
        .filter(|capability| capability.min_sample_rate_hz() <= capability.max_sample_rate_hz())
        .map(|capability| NativeCaptureConfiguration::new(
            capability,
            closest_native_rate(capability, requested.sample_rate_hz()),
            requested.chunk_duration(),
        ))
        .min_by_key(|selection| {
            (
                channel_penalty(selection.channels(), requested.channels()),
                format_penalty(selection.sample_format(), requested.sample_format()),
                u64::from(
                    selection
                        .sample_rate_hz()
                        .abs_diff(requested.sample_rate_hz()),
                ),
                lower_rate_penalty(selection.sample_rate_hz(), requested.sample_rate_hz()),
                // If the requested profile cannot be reproduced exactly,
                // retain the more capable native sample representation.
                std::cmp::Reverse(selection.sample_format().quality_rank()),
                // If still tied, do not throw away channels.
                std::cmp::Reverse(selection.channels()),
                std::cmp::Reverse(selection.sample_rate_hz()),
            )
        })
}

fn closest_native_rate(capability: NativeAudioCapability, preferred: u32) -> u32 {
    preferred.clamp(
        capability.min_sample_rate_hz(),
        capability.max_sample_rate_hz(),
    )
}

fn channel_penalty(actual: u16, preferred: u16) -> u8 {
    u8::from(actual != preferred)
}

fn format_penalty(actual: NativeSampleFormat, preferred: SampleFormat) -> u8 {
    match (actual, preferred) {
        (NativeSampleFormat::Pcm24, SampleFormat::Pcm24)
        | (NativeSampleFormat::F32, SampleFormat::F32) => 0,
        // A native format that can preserve at least as much precision as the
        // requested representation is preferable to falling back to a lower
        // precision representation.
        (NativeSampleFormat::F32, SampleFormat::Pcm24) => 1,
        (NativeSampleFormat::Pcm24, SampleFormat::F32) => 1,
        (NativeSampleFormat::Pcm16, _) => 2,
    }
}

fn lower_rate_penalty(actual: u32, preferred: u32) -> u8 {
    u8::from(actual < preferred)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requested(rate: u32, channels: u16, format: SampleFormat) -> RecordingConfiguration {
        RecordingConfiguration::with_chunk_duration(
            rate,
            channels,
            format,
            RecordingChunkDuration::OneMinute,
        )
    }

    #[test]
    fn exact_native_profile_wins() {
        let capabilities = [
            NativeAudioCapability::new(2, 48_000, 48_000, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];

        let selected = select_best_native_capture(
            &requested(48_000, 1, SampleFormat::Pcm24),
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.channels(), 1);
        assert_eq!(selected.sample_rate_hz(), 48_000);
        assert_eq!(selected.sample_format(), NativeSampleFormat::Pcm24);
    }

    #[test]
    fn native_lower_rate_is_used_instead_of_resampling() {
        let capabilities = [NativeAudioCapability::new(
            1,
            44_100,
            44_100,
            NativeSampleFormat::Pcm24,
        )];

        let selected = select_best_native_capture(
            &requested(48_000, 1, SampleFormat::Pcm24),
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.sample_rate_hz(), 44_100);
    }

    #[test]
    fn native_16_bit_is_preserved_as_a_distinct_capability() {
        let capability = NativeAudioCapability::new(
            1,
            48_000,
            48_000,
            NativeSampleFormat::Pcm16,
        );

        assert_eq!(capability.sample_format(), NativeSampleFormat::Pcm16);
        assert_eq!(capability.sample_format().as_recording_format(), None);
    }

    #[test]
    fn native_stereo_is_retained_when_mono_is_unavailable() {
        let capabilities = [NativeAudioCapability::new(
            2,
            48_000,
            48_000,
            NativeSampleFormat::Pcm24,
        )];

        let selected = select_best_native_capture(
            &requested(48_000, 1, SampleFormat::Pcm24),
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.channels(), 2);
    }

    #[test]
    fn native_24_bit_is_preferred_to_16_bit_when_profile_is_24_bit() {
        let capabilities = [
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm16),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];

        let selected = select_best_native_capture(
            &requested(48_000, 1, SampleFormat::Pcm24),
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.sample_format(), NativeSampleFormat::Pcm24);
    }

    #[test]
    fn higher_native_rate_wins_equal_distance() {
        let capabilities = [
            NativeAudioCapability::new(1, 44_000, 44_000, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 52_000, 52_000, NativeSampleFormat::Pcm24),
        ];

        let selected = select_best_native_capture(
            &requested(48_000, 1, SampleFormat::Pcm24),
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.sample_rate_hz(), 52_000);
    }
}
