//! Domain model for the two audio synchronization anchors defined by ADR-068.

/// The logical synchronization anchor represented in a recording's audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingSyncSignet {
    /// Marks the logical beginning of a recording.
    Opening,
    /// Marks the logical end of a recording.
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
}

#[cfg(test)]
mod tests {
    use super::RecordingSyncSignet;

    #[test]
    fn opening_and_closing_are_distinct_and_self_describing() {
        assert_ne!(RecordingSyncSignet::Opening, RecordingSyncSignet::Closing);
        assert!(RecordingSyncSignet::Opening.is_opening());
        assert!(!RecordingSyncSignet::Opening.is_closing());
        assert!(RecordingSyncSignet::Closing.is_closing());
        assert!(!RecordingSyncSignet::Closing.is_opening());
    }
}
