//! Recording configuration for the technical capture boundary.
//!
//! This module describes the recording parameters requested by NC-PoRe.
//! It does not describe the capabilities of a concrete audio device or
//! backend.
//!
//! See:
//! - ADR-002 Audio Format and Track Concept
//! - ADR-061 Configurable Recording Configuration
//! - ADR-068 Recording Start and Audio Synchronization Signet

use super::signet::SyncSignetConfiguration;

/// Sample representation requested for a recording.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SampleFormat {
    /// Signed 24-bit PCM samples.
    Pcm24,
    /// Signed 32-bit floating-point samples.
    F32,
}

/// Supported recording chunk durations.
///
/// The set is intentionally represented as named values rather than an
/// arbitrary duration so user-facing configuration can offer controlled
/// choices while remaining extensible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordingChunkDuration {
    TenSeconds,
    ThirtySeconds,
    OneMinute,
    TwoMinutes,
    FiveMinutes,
    TenMinutes,
}

impl RecordingChunkDuration {
    /// Returns the duration in seconds.
    pub const fn seconds(self) -> u32 {
        match self {
            Self::TenSeconds => 10,
            Self::ThirtySeconds => 30,
            Self::OneMinute => 60,
            Self::TwoMinutes => 120,
            Self::FiveMinutes => 300,
            Self::TenMinutes => 600,
        }
    }
}

impl Default for RecordingChunkDuration {
    fn default() -> Self {
        Self::OneMinute
    }
}

/// Recording parameters requested from a capture implementation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordingConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: SampleFormat,
    chunk_duration: RecordingChunkDuration,
    signets: SyncSignetConfiguration,
}

impl RecordingConfiguration {
    /// Creates a recording configuration with the default chunk duration
    /// and default Opening/Closing synchronization signets.
    pub const fn new(sample_rate_hz: u32, channels: u16, sample_format: SampleFormat) -> Self {
        Self {
            sample_rate_hz,
            channels,
            sample_format,
            chunk_duration: RecordingChunkDuration::OneMinute,
            signets: SyncSignetConfiguration::new(
                super::signet::SyncSignet::opening(),
                Some(super::signet::SyncSignet::closing()),
            ),
        }
    }

    /// Creates a recording configuration with an explicit chunk duration.
    pub const fn with_chunk_duration(
        sample_rate_hz: u32,
        channels: u16,
        sample_format: SampleFormat,
        chunk_duration: RecordingChunkDuration,
    ) -> Self {
        Self {
            sample_rate_hz,
            channels,
            sample_format,
            chunk_duration,
            signets: SyncSignetConfiguration::new(
                super::signet::SyncSignet::opening(),
                Some(super::signet::SyncSignet::closing()),
            ),
        }
    }

    /// Creates a recording configuration with an explicit synchronization
    /// signet configuration.
    pub const fn with_signets(
        sample_rate_hz: u32,
        channels: u16,
        sample_format: SampleFormat,
        chunk_duration: RecordingChunkDuration,
        signets: SyncSignetConfiguration,
    ) -> Self {
        Self {
            sample_rate_hz,
            channels,
            sample_format,
            chunk_duration,
            signets,
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

    pub const fn chunk_duration(&self) -> RecordingChunkDuration {
        self.chunk_duration
    }

    pub const fn signets(&self) -> SyncSignetConfiguration {
        self.signets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{SignetEvent, SyncSignet, SyncSignetKind};

    // TEST-01
    // Verify: The default recording configuration reflects ADR-002 and
    // uses the defined default chunk duration and signet configuration.
    #[test]
    fn default_configuration_matches_adr_002() {
        let configuration = RecordingConfiguration::default();

        assert_eq!(configuration.sample_rate_hz(), 48_000);
        assert_eq!(configuration.channels(), 1);
        assert_eq!(configuration.sample_format(), SampleFormat::Pcm24);
        assert_eq!(
            configuration.chunk_duration(),
            RecordingChunkDuration::OneMinute
        );
        assert_eq!(
            configuration.signets().opening().kind(),
            SyncSignetKind::Opening
        );
    }

    // TEST-02
    // Verify: User-requested recording parameters are represented
    // without being coupled to a concrete audio backend.
    #[test]
    fn configuration_preserves_explicit_parameters() {
        let configuration = RecordingConfiguration::with_chunk_duration(
            44_100,
            2,
            SampleFormat::F32,
            RecordingChunkDuration::FiveMinutes,
        );

        assert_eq!(configuration.sample_rate_hz(), 44_100);
        assert_eq!(configuration.channels(), 2);
        assert_eq!(configuration.sample_format(), SampleFormat::F32);
        assert_eq!(
            configuration.chunk_duration(),
            RecordingChunkDuration::FiveMinutes
        );
    }

    // TEST-03
    // Verify: The supported chunk durations remain explicit and stable.
    #[test]
    fn chunk_duration_values_are_explicit() {
        assert_eq!(RecordingChunkDuration::TenSeconds.seconds(), 10);
        assert_eq!(RecordingChunkDuration::ThirtySeconds.seconds(), 30);
        assert_eq!(RecordingChunkDuration::OneMinute.seconds(), 60);
        assert_eq!(RecordingChunkDuration::TwoMinutes.seconds(), 120);
        assert_eq!(RecordingChunkDuration::FiveMinutes.seconds(), 300);
        assert_eq!(RecordingChunkDuration::TenMinutes.seconds(), 600);
    }

    // TEST-04
    // Verify: An explicit signet configuration can replace the default pattern.
    #[test]
    fn configuration_accepts_custom_signets() {
        let opening = SyncSignet::new(
            SyncSignetKind::Opening,
            [
                SignetEvent::new(0, 20),
                SignetEvent::new(80, 20),
                SignetEvent::new(160, 20),
            ],
            0.08,
            42,
        );
        let signets = SyncSignetConfiguration::new(opening, None);
        let configuration = RecordingConfiguration::with_signets(
            48_000,
            1,
            SampleFormat::Pcm24,
            RecordingChunkDuration::OneMinute,
            signets,
        );

        assert_eq!(configuration.signets().opening(), opening);
        assert!(configuration.signets().closing().is_none());
    }
}
