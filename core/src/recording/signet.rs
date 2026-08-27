//! Domain model for the two audio synchronization anchors defined by ADR-068.

/// Whether emitting a synchronization signet is required or optional.
///
/// This is a domain-level requirement and deliberately does not prescribe how
/// the signet is transported or physically captured.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingSyncSignetRequirement {
    /// The signet is required for the recording workflow.
    Required,
    /// The signet may be emitted when the implementation supports it.
    Optional,
}

/// The logical synchronization anchor represented in a recording's audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingSyncSignet {
    /// Marks the logical beginning of a recording. This signet is required.
    Opening,
    /// Marks the logical end of a recording. This signet is optional.
    Closing,
}

impl RecordingSyncSignet {
    /// Returns whether this signet marks the logical beginning of the recording.
    pub const fn is_opening(self) -> bool {
        matches!(self, Self::Opening)
    }

    /// Returns whether this signet marks the logical end of the recording.
    pub const fn is_closing(self) -> bool {
        matches!(self, Self::Closing)
    }

    /// Returns the ADR-068 requirement for emitting this signet.
    ///
    /// Opening is a required synchronization event. Closing is an optional
    /// synchronization aid and must not become a prerequisite for completion.
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

    #[test]
    fn opening_and_closing_are_distinct_and_self_describing() {
        assert_ne!(RecordingSyncSignet::Opening, RecordingSyncSignet::Closing);
        assert!(RecordingSyncSignet::Opening.is_opening());
        assert!(!RecordingSyncSignet::Opening.is_closing());
        assert!(RecordingSyncSignet::Closing.is_closing());
        assert!(!RecordingSyncSignet::Closing.is_opening());
    }

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
