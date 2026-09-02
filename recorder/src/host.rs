//! Host integration boundary for local recording.
//!
//! A host connector observes host-specific state and translates it into
//! host-neutral recording facts. It never supplies the PoRE audio payload.
//!
//! This boundary deliberately keeps Nextcloud Talk, BigBlueButton, Jitsi,
//! and future host integrations out of the capture, preservation, and
//! transport layers.
//!
//! See ADR-078i: Host-neutral Capture, Preservation and Transport.

/// A host-observed local capture source.
///
/// The identifier is intentionally opaque to the recorder core. A connector
/// may use a browser device id, a host-specific source identifier, or another
/// stable local identifier. The recorder must not infer host semantics from it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSource {
    id: String,
    label: Option<String>,
}

impl CaptureSource {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: None,
        }
    }

    pub fn with_label(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: Some(label.into()),
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }
}

/// Host-neutral event describing a change of the local capture source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSourceChange {
    occurred_at_unix_ms: u64,
    previous: Option<CaptureSource>,
    current: CaptureSource,
}

impl CaptureSourceChange {
    /// Creates a source-change event using Unix epoch milliseconds.
    pub fn new(
        occurred_at_unix_ms: u64,
        previous: Option<CaptureSource>,
        current: CaptureSource,
    ) -> Self {
        Self {
            occurred_at_unix_ms,
            previous,
            current,
        }
    }

    pub fn occurred_at_unix_ms(&self) -> u64 {
        self.occurred_at_unix_ms
    }

    pub fn previous(&self) -> Option<&CaptureSource> {
        self.previous.as_ref()
    }

    pub fn current(&self) -> &CaptureSource {
        &self.current
    }
}

/// Neutral interface implemented by host-specific connectors.
///
/// A connector reports host observations only. Audio data, capture
/// configuration, preservation, and transport remain owned by PoRE.
pub trait HostConnector {
    type Error;

    /// Returns the source currently selected by the host, if known.
    fn selected_capture_source(&self) -> Result<Option<CaptureSource>, Self::Error>;

    /// Returns source-change observations since the previous poll.
    fn poll_capture_source_changes(&mut self) -> Result<Vec<CaptureSourceChange>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeHostConnector {
        source: Option<CaptureSource>,
        changes: Vec<CaptureSourceChange>,
    }

    impl HostConnector for FakeHostConnector {
        type Error = &'static str;

        fn selected_capture_source(&self) -> Result<Option<CaptureSource>, Self::Error> {
            Ok(self.source.clone())
        }

        fn poll_capture_source_changes(&mut self) -> Result<Vec<CaptureSourceChange>, Self::Error> {
            Ok(std::mem::take(&mut self.changes))
        }
    }

    // TEST-37
    //
    // Protects ADR-078i: host integrations expose an opaque capture source,
    // not an audio payload or a host-specific recording representation.
    #[test]
    fn connector_exposes_selected_capture_source() {
        let connector = FakeHostConnector {
            source: Some(CaptureSource::with_label("device-1", "Microphone")),
            changes: Vec::new(),
        };

        let source = connector
            .selected_capture_source()
            .expect("source lookup succeeds")
            .expect("source exists");

        assert_eq!(source.id(), "device-1");
        assert_eq!(source.label(), Some("Microphone"));
    }

    // TEST-38
    //
    // Protects ADR-078i: source changes retain timestamp and both sides of
    // the transition so provenance can be attached to the recording tracks.
    #[test]
    fn connector_reports_host_neutral_source_changes() {
        let previous = CaptureSource::new("device-1");
        let current = CaptureSource::new("device-2");
        let change =
            CaptureSourceChange::new(1_762_000_000_000, Some(previous.clone()), current.clone());
        let mut connector = FakeHostConnector {
            source: Some(current),
            changes: vec![change],
        };

        let changes = connector
            .poll_capture_source_changes()
            .expect("change lookup succeeds");

        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].occurred_at_unix_ms(), 1_762_000_000_000);
        assert_eq!(changes[0].previous(), Some(&previous));
        assert_eq!(changes[0].current().id(), "device-2");
        assert!(
            connector
                .poll_capture_source_changes()
                .expect("second poll succeeds")
                .is_empty()
        );
    }
}
