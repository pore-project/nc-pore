//! Recording configuration for the technical capture boundary.
//!
//! This module describes the recording parameters requested by NC-PoRe.
//! It does not describe the capabilities of a concrete audio device or
//! backend.
//!
//! See:
//! - ADR-002 Audio Format and Track Concept
//! - ADR-061 Configurable Recording Configuration

/// Sample representation requested for a recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleFormat {
    /// Signed 24-bit PCM samples.
    Pcm24,
    /// Signed 32-bit floating-point samples.
    F32,
}

/// Recording parameters requested from a capture implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordingConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: SampleFormat,
}

impl RecordingConfiguration {
    /// Creates a recording configuration with explicit parameters.
    pub const fn new(sample_rate_hz: u32, channels: u16, sample_format: SampleFormat) -> Self {
        Self {
            sample_rate_hz,
            channels,
            sample_format,
        }
    }

    /// Returns the preferred NC-PoRe recording configuration from ADR-002.
    pub const fn default() -> Self {
        Self::new(48_000, 1, SampleFormat::Pcm24)
    }

    pub const fn sample_rate_hz(&self) -> u32 {
        self.sample_rate_hz
    }

    pub const fn channels(&self) -> u16 {
        self.channels
    }

    pub const fn sample_format(&self) -> SampleFormat {
        self.sample_format
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01
    // Verify: The default recording configuration reflects ADR-002.
    #[test]
    fn default_configuration_matches_adr_002() {
        let configuration = RecordingConfiguration::default();

        assert_eq!(configuration.sample_rate_hz(), 48_000);
        assert_eq!(configuration.channels(), 1);
        assert_eq!(configuration.sample_format(), SampleFormat::Pcm24);
    }

    // TEST-02
    // Verify: User-requested recording parameters are represented
    // without being coupled to a concrete audio backend.
    #[test]
    fn configuration_preserves_explicit_parameters() {
        let configuration = RecordingConfiguration::new(44_100, 2, SampleFormat::F32);

        assert_eq!(configuration.sample_rate_hz(), 44_100);
        assert_eq!(configuration.channels(), 2);
        assert_eq!(configuration.sample_format(), SampleFormat::F32);
    }
}
