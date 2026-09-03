//! Application-level vertical slice for a finalized browser recording.
//!
//! This module deliberately reuses the neutral browser handoff and the
//! existing recorder artifact/persistence boundaries. It is a small executable
//! seam for the browser-facing slice until a real browser runtime supplies the
//! finalized bytes.

use recorder::artifact::RecordingArtifact;
use recorder::persistence::{PersistenceProvider, PersistenceStoreError};
use recorder::session::RecordingSessionId;

use crate::browser_recording_handoff::{
    BrowserRecordingHandoff, BrowserRecordingPersistenceError, BrowserRecordingStopReason,
};

/// Persist one finalized browser recording through the existing recorder
/// artifact and persistence boundaries.
///
/// No browser-runtime or Talk-specific type crosses this function. The
/// finalized bytes are handed to the neutral boundary, converted to the
/// existing `CaptureResult`, turned into a `RecordingArtifact` by the existing
/// factory, and finally persisted by the existing provider.
pub fn persist_finalized_browser_recording<P: PersistenceProvider>(
    recording_id: impl Into<String>,
    track_id: impl Into<String>,
    payload: impl Into<Vec<u8>>,
    format: impl Into<String>,
    recording_session_id: RecordingSessionId,
    provider: &mut P,
) -> Result<RecordingArtifact, BrowserRecordingArtifactSliceError> {
    BrowserRecordingHandoff::new(
        recording_id,
        track_id,
        payload,
        format,
        BrowserRecordingStopReason::UserRequested,
    )
    .persist(recording_session_id, provider)
    .map_err(BrowserRecordingArtifactSliceError::Persistence)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRecordingArtifactSliceError {
    Persistence(BrowserRecordingPersistenceError),
}

impl From<BrowserRecordingPersistenceError> for BrowserRecordingArtifactSliceError {
    fn from(error: BrowserRecordingPersistenceError) -> Self {
        Self::Persistence(error)
    }
}

impl BrowserRecordingArtifactSliceError {
    /// Returns the underlying persistence conflict when one occurred.
    pub fn persistence_error(&self) -> Option<&PersistenceStoreError> {
        match self {
            Self::Persistence(BrowserRecordingPersistenceError::Persistence(error)) => Some(error),
            Self::Persistence(BrowserRecordingPersistenceError::Handoff(_)) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use recorder::artifact::ArtifactStatus;
    use recorder::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

    #[test]
    fn finalized_browser_recording_reaches_stored_artifact() {
        let mut provider = InMemoryPersistenceProvider::new();

        let artifact = persist_finalized_browser_recording(
            "browser-recording-001",
            "browser-track",
            vec![10, 20, 30],
            "audio/webm",
            RecordingSessionId::new("browser-session-001"),
            &mut provider,
        )
        .expect("browser artifact should persist");

        assert!(matches!(artifact.status(), ArtifactStatus::Stored));
        assert_eq!(artifact.id.value(), "browser-recording-001");
        assert_eq!(artifact.recording_session_id.value(), "browser-session-001");
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks()[0].payload().data(), &[10, 20, 30]);
        assert!(matches!(
            provider.load("browser-recording-001"),
            PersistenceLoadResult::Valid(stored) if matches!(stored.status(), ArtifactStatus::Stored)
        ));
    }

    #[test]
    fn browser_artifact_persistence_is_idempotent_for_equivalent_retries() {
        let mut provider = InMemoryPersistenceProvider::new();

        persist_finalized_browser_recording(
            "browser-recording-002",
            "browser-track",
            vec![1, 2, 3],
            "audio/webm",
            RecordingSessionId::new("browser-session-002"),
            &mut provider,
        )
        .expect("first persistence should succeed");

        persist_finalized_browser_recording(
            "browser-recording-002",
            "browser-track",
            vec![1, 2, 3],
            "audio/webm",
            RecordingSessionId::new("browser-session-002"),
            &mut provider,
        )
        .expect("equivalent retry should be idempotent");

        assert_eq!(provider.list().len(), 1);
    }

    #[test]
    fn browser_artifact_persistence_rejects_conflicting_retry() {
        let mut provider = InMemoryPersistenceProvider::new();

        persist_finalized_browser_recording(
            "browser-recording-003",
            "browser-track",
            vec![1, 2, 3],
            "audio/webm",
            RecordingSessionId::new("browser-session-003"),
            &mut provider,
        )
        .expect("first persistence should succeed");

        let result = persist_finalized_browser_recording(
            "browser-recording-003",
            "browser-track",
            vec![9, 9, 9],
            "audio/webm",
            RecordingSessionId::new("browser-session-003"),
            &mut provider,
        );

        assert!(matches!(
            result,
            Err(BrowserRecordingArtifactSliceError::Persistence(
                BrowserRecordingPersistenceError::Persistence(
                    PersistenceStoreError::Conflict { .. }
                )
            ))
        ));
        assert_eq!(provider.list().len(), 1);
    }
}
