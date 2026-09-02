//! Backend-independent native capture selection.

use crate::configuration::SampleFormat;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSampleFormat {
    Pcm16,
    Pcm24,
    F32,
}

impl NativeSampleFormat {
    pub fn quality_rank(self) -> u8 {
        match self {
            Self::Pcm16 => 1,
            Self::Pcm24 => 2,
            Self::F32 => 3,
        }
    }

    pub fn as_recording_format(self) -> SampleFormat {
        match self {
            Self::Pcm16 => SampleFormat::Pcm16,
            Self::Pcm24 => SampleFormat::Pcm24,
            Self::F32 => SampleFormat::F32,
        }
    }

    pub fn supports_lossless_flac_transport(self) -> bool {
        matches!(self, Self::Pcm16 | Self::Pcm24)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn native_sample_rate_distance(self, requested: u32) -> u64 {
        if requested < self.min_sample_rate_hz {
            u64::from(self.min_sample_rate_hz - requested)
        } else if requested > self.max_sample_rate_hz {
            u64::from(requested - self.max_sample_rate_hz)
        } else {
            0
        }
    }

    fn concrete_sample_rate(self, requested: u32) -> u32 {
        requested.clamp(self.min_sample_rate_hz, self.max_sample_rate_hz)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NativeCaptureConfiguration {
    channels: u16,
    sample_rate_hz: u32,
    sample_format: NativeSampleFormat,
}

impl NativeCaptureConfiguration {
    pub const fn channels(self) -> u16 {
        self.channels
    }

    pub const fn sample_rate_hz(self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn sample_format(self) -> NativeSampleFormat {
        self.sample_format
    }
}

pub fn select_best_native_capture(
    requested: &crate::configuration::RecordingConfiguration,
    capabilities: &[NativeAudioCapability],
) -> Option<NativeCaptureConfiguration> {
    capabilities
        .iter()
        .copied()
        .min_by_key(|capability| {
            let channel_penalty = if capability.channels == requested.channels() {
                0
            } else {
                1
            };
            let format_penalty = if capability.sample_format.as_recording_format()
                == requested.sample_format()
            {
                0
            } else {
                1
            };
            let rate_distance = capability.native_sample_rate_distance(requested.sample_rate_hz());
            let lower_rate_penalty = u64::from(
                capability.concrete_sample_rate(requested.sample_rate_hz())
                    < requested.sample_rate_hz(),
            );

            (
                channel_penalty,
                format_penalty,
                rate_distance,
                lower_rate_penalty,
                u8::MAX - capability.sample_format.quality_rank(),
                u16::MAX - capability.channels,
                u32::MAX - capability.concrete_sample_rate(requested.sample_rate_hz()),
            )
        })
        .map(|capability| NativeCaptureConfiguration {
            channels: capability.channels,
            sample_rate_hz: capability.concrete_sample_rate(requested.sample_rate_hz()),
            sample_format: capability.sample_format,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn requested(
        sample_rate_hz: u32,
        channels: u16,
        sample_format: SampleFormat,
    ) -> crate::configuration::RecordingConfiguration {
        crate::configuration::RecordingConfiguration::new(sample_rate_hz, channels, sample_format)
    }

    #[test]
    fn exact_native_profile_wins() {
        let capabilities = [
            NativeAudioCapability::new(2, 44_100, 44_100, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];
        let selected =
            select_best_native_capture(&requested(48_000, 1, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.channels(), 1);
        assert_eq!(selected.sample_rate_hz(), 48_000);
        assert_eq!(selected.sample_format(), NativeSampleFormat::Pcm24);
    }

    #[test]
    fn native_rate_is_selected_without_resampling() {
        let capabilities = [NativeAudioCapability::new(
            1,
            44_100,
            48_000,
            NativeSampleFormat::Pcm24,
        )];
        let selected =
            select_best_native_capture(&requested(47_000, 1, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.sample_rate_hz(), 47_000);
    }

    #[test]
    fn out_of_range_rate_uses_nearest_native_boundary() {
        let capabilities = [NativeAudioCapability::new(
            1,
            44_100,
            44_100,
            NativeSampleFormat::Pcm24,
        )];
        let selected =
            select_best_native_capture(&requested(48_000, 1, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.sample_rate_hz(), 44_100);
    }

    #[test]
    fn native_16_bit_is_preserved_as_a_real_recording_format() {
        let capability =
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm16);
        assert_eq!(
            capability.sample_format().as_recording_format(),
            SampleFormat::Pcm16
        );
        assert!(
            capability
                .sample_format()
                .supports_lossless_flac_transport()
        );
        assert_eq!(capability.sample_format().quality_rank(), 1);
    }

    #[test]
    fn native_float_remains_selectable_at_the_capture_boundary() {
        let capabilities = [NativeAudioCapability::new(
            1,
            48_000,
            48_000,
            NativeSampleFormat::F32,
        )];
        let selected =
            select_best_native_capture(&requested(48_000, 1, SampleFormat::F32), &capabilities)
                .unwrap();
        assert_eq!(selected.sample_format(), NativeSampleFormat::F32);
        assert!(!selected.sample_format().supports_lossless_flac_transport());
    }

    #[test]
    fn requested_stereo_is_preferred_over_mono() {
        let capabilities = [
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(2, 44_100, 44_100, NativeSampleFormat::Pcm24),
        ];
        let selected =
            select_best_native_capture(&requested(48_000, 2, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.channels(), 2);
    }

    #[test]
    fn pcm24_is_preferred_over_pcm16_when_requested() {
        let capabilities = [
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm16),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];
        let selected =
            select_best_native_capture(&requested(48_000, 1, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.sample_format(), NativeSampleFormat::Pcm24);
    }

    #[test]
    fn equal_rate_distance_prefers_higher_rate() {
        let capabilities = [
            NativeAudioCapability::new(1, 44_100, 44_100, NativeSampleFormat::Pcm24),
            NativeAudioCapability::new(1, 48_000, 48_000, NativeSampleFormat::Pcm24),
        ];
        let selected =
            select_best_native_capture(&requested(46_050, 1, SampleFormat::Pcm24), &capabilities)
                .unwrap();
        assert_eq!(selected.sample_rate_hz(), 48_000);
    }
}
