//! NC-PoRe synchronization signet model.
//!
//! This module defines the provider-neutral description of a synchronization
//! signet. The concrete audio backend decides how the description is rendered.
//!
//! See ADR-068 Recording Start and Audio Synchronization Signet.

/// Identifies the logical role of a synchronization signet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncSignetKind {
    Opening,
    Closing,
}

/// One timed event within a synchronization signet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignetEvent {
    start_ms: u32,
    duration_ms: u32,
}

impl SignetEvent {
    /// Creates one signet event at the given offset.
    pub const fn new(start_ms: u32, duration_ms: u32) -> Self {
        Self {
            start_ms,
            duration_ms,
        }
    }

    pub const fn start_ms(self) -> u32 {
        self.start_ms
    }

    pub const fn duration_ms(self) -> u32 {
        self.duration_ms
    }
}

/// Configurable provider-neutral signet description.
///
/// The temporal event pattern, amplitude and renderer seed are configuration
/// data rather than fixed recorder policy. Amplitude is stored as a millionth
/// of full scale so the configuration remains exactly comparable. A concrete
/// capture provider may render this description according to its audio
/// technology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncSignet {
    kind: SyncSignetKind,
    events: [SignetEvent; 3],
    amplitude_ppm: u32,
    seed: u32,
}

impl SyncSignet {
    /// Creates a signet from its configured temporal and rendering parameters.
    ///
    /// `amplitude` is the linear amplitude in the range 0.0..=1.0. It is
    /// converted to parts per million internally to keep the value exactly
    /// comparable. For example, `0.12` represents 0.12 full scale.
    pub const fn new(
        kind: SyncSignetKind,
        events: [SignetEvent; 3],
        amplitude: f32,
        seed: u32,
    ) -> Self {
        Self {
            kind,
            events,
            amplitude_ppm: (amplitude * 1_000_000.0) as u32,
            seed,
        }
    }

    /// Returns the default opening signet configuration.
    pub const fn opening() -> Self {
        Self::default_for(SyncSignetKind::Opening, 0x1357_9bdf)
    }

    /// Returns the default closing signet configuration.
    pub const fn closing() -> Self {
        Self::default_for(SyncSignetKind::Closing, 0x2468_ace1)
    }

    const fn default_for(kind: SyncSignetKind, seed: u32) -> Self {
        Self::new(
            kind,
            [
                SignetEvent::new(0, 40),
                SignetEvent::new(120, 40),
                SignetEvent::new(240, 40),
            ],
            0.12,
            seed,
        )
    }

    pub const fn kind(self) -> SyncSignetKind {
        self.kind
    }

    pub const fn events(self) -> [SignetEvent; 3] {
        self.events
    }

    pub const fn amplitude_ppm(self) -> u32 {
        self.amplitude_ppm
    }

    pub const fn seed(self) -> u32 {
        self.seed
    }

    /// Returns the total temporal extent of the signet.
    pub const fn duration_ms(self) -> u32 {
        let last = self.events[2];
        last.start_ms() + last.duration_ms()
    }
}

/// Configures which concrete signet descriptions the recorder lifecycle uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncSignetConfiguration {
    opening: SyncSignet,
    closing: Option<SyncSignet>,
}

impl SyncSignetConfiguration {
    /// Creates a configuration with a required opening signet and an optional
    /// closing signet.
    pub const fn new(opening: SyncSignet, closing: Option<SyncSignet>) -> Self {
        Self { opening, closing }
    }

    pub const fn opening(self) -> SyncSignet {
        self.opening
    }

    pub const fn closing(self) -> Option<SyncSignet> {
        self.closing
    }
}

impl Default for SyncSignetConfiguration {
    fn default() -> Self {
        Self::new(SyncSignet::opening(), Some(SyncSignet::closing()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01
    // Verify: A signet preserves caller-supplied timing and rendering parameters.
    #[test]
    fn signet_preserves_configuration() {
        let signet = SyncSignet::new(
            SyncSignetKind::Opening,
            [
                SignetEvent::new(0, 25),
                SignetEvent::new(75, 30),
                SignetEvent::new(180, 50),
            ],
            0.2,
            1234,
        );

        assert_eq!(signet.events()[1], SignetEvent::new(75, 30));
        assert_eq!(signet.amplitude_ppm(), 200_000);
        assert_eq!(signet.seed(), 1234);
        assert_eq!(signet.duration_ms(), 230);
    }

    // TEST-02
    // Verify: Opening and Closing remain distinct logical anchors.
    #[test]
    fn opening_and_closing_are_distinct() {
        assert_eq!(SyncSignet::opening().kind(), SyncSignetKind::Opening);
        assert_eq!(SyncSignet::closing().kind(), SyncSignetKind::Closing);
        assert_ne!(SyncSignet::opening(), SyncSignet::closing());
    }

    // TEST-03
    // Verify: The default configuration requires Opening and permits Closing.
    #[test]
    fn default_configuration_contains_required_opening_and_optional_closing() {
        let configuration = SyncSignetConfiguration::default();

        assert_eq!(configuration.opening().kind(), SyncSignetKind::Opening);
        assert_eq!(
            configuration.closing().unwrap().kind(),
            SyncSignetKind::Closing
        );
    }

    // TEST-04
    // Verify: Closing can be disabled without affecting the required Opening.
    #[test]
    fn configuration_can_omit_closing() {
        let configuration = SyncSignetConfiguration::new(SyncSignet::opening(), None);

        assert_eq!(configuration.opening().kind(), SyncSignetKind::Opening);
        assert!(configuration.closing().is_none());
    }
}
