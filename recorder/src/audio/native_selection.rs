//! Native audio capability selection policy.
//!
//! The selection policy is deliberately independent of a concrete audio
//! backend. A backend reports what it can actually provide; this module
//! chooses the best native representation without resampling or
//! up-conversion.

use super::SampleFormat;

/// Sample representation exposed by a native capture backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeSampleFormat {
    Pcm16,
    Pcm24,
    F32,
}

impl NativeSampleFormat {
    pub const fn quality_rank(self) -> u8 {
        match self {
            Self::Pcm16 => 0,
            Self::Pcm24 => 2,
            Self::F32 => 1,
        }
    }

    pub const fn as_recording_format(self) -> Option<SampleFormat> {
        match self {
            Self::Pcm24 => Some(SampleFormat::Pcm24),
            Self::F32 => Some(SampleFormat::F32),
            Self::Pcm16 => None,
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
}

impl NativeCaptureConfiguration {
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
}

/// Select the best native capability for a preferred recording profile.
///
/// The policy is intentionally conservative:
///
/// * exact channel count is preferred;
/// * the requested sample representation is preferred;
/// * the closest actually supported sample rate is selected;
/// * a higher native rate wins ties over a lower native rate;
/// * native Pcm16 is retained as a real capability, never represented as
///   synthetic 24-bit audio.
///
/// This function only chooses a native configuration. It never performs
/// conversion, resampling, channel mixing, or bit-depth expansion.
pub fn select_best_native_capture(
    preferred_sample_rate_hz: u32,
    preferred_channels: u16,
    preferred_format: NativeSampleFormat,
    capabilities: &[NativeAudioCapability],
) -> Option<NativeCaptureConfiguration> {
    capabilities
        .iter()
        .copied()
        .filter(|capability| capability.min_sample_rate_hz() <= capability.max_sample_rate_hz())
        .map(|capability| NativeCaptureConfiguration {
            capability,
            sample_rate_hz: closest_native_rate(capability, preferred_sample_rate_hz),
        })
        .min_by_key(|selection| {
            (
                channel_penalty(selection.channels(), preferred_channels),
                format_penalty(selection.sample_format(), preferred_format),
                u64::from(selection.sample_rate_hz().abs_diff(preferred_sample_rate_hz)),
                lower_rate_penalty(selection.sample_rate_hz(), preferred_sample_rate_hz),
                // When all other factors tie, retain the higher native
                // channel count rather than throwing information away.
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
    if actual == preferred { 0 } else { 1 }
}

fn format_penalty(actual: NativeSampleFormat, preferred: NativeSampleFormat) -> u8 {
    if actual == preferred { 0 } else { 1 }
}

fn lower_rate_penalty(actual: u32, preferred: u32) -> u8 {
    if actual < preferred { 1 } else { 0 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_native_profile_wins() {
        let capabilities = [
            NativeAudioCapability::new(2, 48_000, 48_000, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];

        let selected = select_best_native_capture(
            48_000,
            1,
            NativeSampleFormat::Pcm24,
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
            48_000,
            1,
            NativeSampleFormat::Pcm24,
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.sample_rate_hz(), 44_100);
    }

    #[test]
    fn native_16_bit_is_not_promoted_to_24_bit() {
        let capability = NativeAudioCapability::new(
            1,
            48_000,
            48_000,
            NativeSampleFormat::Pcm16,
        );

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
            48_000,
            1,
            NativeSampleFormat::Pcm24,
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.channels(), 2);
    }

    #[test]
    fn exact_format_is_preferred_before_rate_distance() {
        let capabilities = [
            NativeAudioCapability::new(1, 44_100, 44_100, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::F32),
        ];

        let selected = select_best_native_capture(
            48_000,
            1,
            NativeSampleFormat::Pcm24,
            &capabilities,
        )
        .unwrap();

        assert_eq!(selected.sample_format(), NativeSampleFormat::Pcm24);
        assert_eq!(selected.sample_rate_hz(), 44_100);
    }
}
