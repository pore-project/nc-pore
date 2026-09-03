//! Domain model for the two audio synchronization anchors defined by ADR-068.

/// Whether emitting a synchronization signet is required or optional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingSyncSignetRequirement {
    Required,
    Optional,
}

/// The logical synchronization anchor represented in a recording's audio.
///
/// This domain value describes the lifecycle event only. It deliberately does
/// not prescribe transport, waveform, codec, or how the signet is captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingSyncSignet {
    /// Marks the logical beginning of a recording and is required.
    Opening,
    /// Marks the logical end of a recording and is optional.
    Closing,
}

impl RecordingSyncSignet {
    pub const fn is_opening(self) -> bool {
        matches!(self, Self::Opening)
    }

    pub const fn is_closing(self) -> bool {
        matches!(self, Self::Closing)
    }

    /// Returns the ADR-068 requirement for this synchronization anchor.
    pub const fn requirement(self) -> RecordingSyncSignetRequirement {
        match self {
            Self::Opening => RecordingSyncSignetRequirement::Required,
            Self::Closing => RecordingSyncSignetRequirement::Optional,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{RecordingSyncSignet, RecordingSyncSignetRequirement};

    // TEST-01
    #[test]
    fn opening_and_closing_are_distinct() {
        assert_ne!(RecordingSyncSignet::Opening, RecordingSyncSignet::Closing);
        assert!(RecordingSyncSignet::Opening.is_opening());
        assert!(!RecordingSyncSignet::Opening.is_closing());
        assert!(RecordingSyncSignet::Closing.is_closing());
        assert!(!RecordingSyncSignet::Closing.is_opening());
    }

    // TEST-02
    #[test]
    fn opening_is_required_and_closing_is_optional() {
        assert_eq!(
            RecordingSyncSignet::Opening.requirement(),
            RecordingSyncSignetRequirement::Required
        );
        assert_eq!(
            RecordingSyncSignet::Closing.requirement(),
            RecordingSyncSignetRequirement::Optional
        );
    }
}
