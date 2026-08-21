//! NC-PoRe synchronization signet model.
//!
//! The signet is a short, deliberately recognizable audio event used as a
//! shared reference point in recordings. This module models the signal as a
//! sequence of timed broadband events without coupling it to a concrete audio
//! backend.
//!
//! See ADR-068 Recording Start and Audio Synchronization Signet.

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
/// structure needed by the recorder workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncSignet {
    events: [SignetEvent; 3],
}

impl SyncSignet {
    /// Returns the initial three-event signet pattern defined by ADR-068.
    pub const fn opening() -> Self {
        Self {
            events: [
                SignetEvent::new(0, 40),
                SignetEvent::new(120, 40),
                SignetEvent::new(240, 40),
            ],
        }
    }

    pub const fn events(self) -> [SignetEvent; 3] {
        self.events
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
}
