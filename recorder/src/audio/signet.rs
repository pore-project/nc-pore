//! NC-PoRe synchronization signet model.
//!
//! The signet is a short, deliberately recognizable audio event used as a
//! shared reference point in recordings. This module models the signal as a
//! sequence of timed broadband events without coupling it to a concrete audio
//! backend.
//!
//! See ADR-068 Recording Start and Audio Synchronization Signet.

/// Identifies the logical role of a synchronization signet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncSignetKind {
    Opening,
    Closing,
}

/// One broadband event within a synchronization signet.
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

/// The currently selected NC-PoRe synchronization signet pattern.
///
/// ADR-068 deliberately leaves the concrete waveform, spectrum and loudness
/// open. The first implementation step therefore defines only the temporal
/// structure needed by the recorder workflow. Opening and Closing are kept as
/// distinct logical signet kinds even while they share the same temporal
/// structure in this backend-independent model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncSignet {
    kind: SyncSignetKind,
    events: [SignetEvent; 3],
}

impl SyncSignet {
    /// Returns the initial three-event opening signet pattern defined by ADR-068.
    pub const fn opening() -> Self {
        Self {
            kind: SyncSignetKind::Opening,
            events: Self::events(),
        }
    }

    /// Returns the initial three-event closing signet pattern defined by ADR-068.
    ///
    /// The concrete waveform may later distinguish the closing signet from the
    /// opening signet by signal direction or spectrum. The logical distinction
    /// is already explicit at this layer.
    pub const fn closing() -> Self {
        Self {
            kind: SyncSignetKind::Closing,
            events: Self::events(),
        }
    }

    pub const fn kind(self) -> SyncSignetKind {
        self.kind
    }

    pub const fn events(self) -> [SignetEvent; 3] {
        self.events
    }

    /// Returns the shared temporal structure used by the current signet family.
    const fn events() -> [SignetEvent; 3] {
        [
            SignetEvent::new(0, 40),
            SignetEvent::new(120, 40),
            SignetEvent::new(240, 40),
        ]
    }

    /// Returns the total temporal extent of the signet.
    pub const fn duration_ms(self) -> u32 {
        let last = self.events[2];
        last.start_ms() + last.duration_ms()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01 / CUE30
    // Verify: The opening signet consists of exactly three timed events.
    #[test]
    fn opening_signet_has_three_events() {
        let signet = SyncSignet::opening();

        assert_eq!(signet.events().len(), 3);
        assert_eq!(signet.kind(), SyncSignetKind::Opening);
    }

    // TEST-02 / CUE30
    // Verify: The opening signet events are ordered and equally spaced.
    #[test]
    fn opening_signet_events_are_evenly_spaced() {
        let events = SyncSignet::opening().events();

        assert_eq!(events[0].start_ms(), 0);
        assert_eq!(events[1].start_ms() - events[0].start_ms(), 120);
        assert_eq!(events[2].start_ms() - events[1].start_ms(), 120);
    }

    // TEST-03 / CUE30
    // Verify: The temporal extent includes the duration of the final event.
    #[test]
    fn opening_signet_duration_includes_final_event() {
        assert_eq!(SyncSignet::opening().duration_ms(), 280);
    }

    // TEST-04 / CUE30
    // Verify: Opening and Closing are distinct logical anchors in the signet family.
    #[test]
    fn closing_signet_has_distinct_logical_kind() {
        assert_eq!(SyncSignet::closing().kind(), SyncSignetKind::Closing);
        assert_ne!(SyncSignet::opening(), SyncSignet::closing());
    }

    // TEST-05 / CUE30
    // Verify: Opening and Closing share the current backend-independent timing structure.
    #[test]
    fn opening_and_closing_share_temporal_structure() {
        assert_eq!(SyncSignet::opening().events(), SyncSignet::closing().events());
        assert_eq!(SyncSignet::opening().duration_ms(), SyncSignet::closing().duration_ms());
    }
}
