//! Neutral handoff from a browser capture producer into the recorder pipeline.
//!
//! The browser-facing layer may produce a finalized container (for example,
//! a browser Blob), but that representation must not be mistaken for a
//! `RecordingArtifact`. This module defines the application boundary between
//! that producer and the existing capture/artifact pipeline.
//!
//! Talk- or browser-runtime-specific types deliberately do not cross this
//! boundary. The handoff contains only technical capture data needed to build
//! the existing `CaptureResult` representation.

use recorder::artifact::factory::RecordingArtifactFactory;
use recorder::artifact::RecordingArtifact;
use recorder::audio::{CaptureResult, CaptureTrack, RecordingConfiguration};
use recorder::persistence::{PersistenceProvider, PersistenceStoreError};
use recorder::session::RecordingSessionId;

/// Technical stop reason supplied by a capture producer.
///
/// The current `CaptureResult` model does not yet persist this value. It is
/// therefore retained at the handoff boundary only and can be promoted to
/// the artifact metadata model when that boundary supports it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRecordingStopReason {
    UserRequested,
    SessionEnded,
    TechnicalFailure,
}

/// Finalized browser recording in neutral technical form.
///
/// `payload` is the finalized recording container as bytes. A successful
/// handoff converts it into the existing `CaptureResult`; no
/// `RecordingArtifact` is created implicitly by this type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserRecordingHandoff {
    recording_id: String,
    track_id: String,
    payload: Vec<u8>,
    format: String,
    recording_started_at: Option<String>,
    recording_stopped_at: Option<String>,
    stop_reason: BrowserRecordingStopReason,
    configuration: Option<RecordingConfiguration>,
}

impl BrowserRecordingHandoff {
    /// Creates a finalized browser recording handoff.
    pub fn new(
        recording_id: impl Into<String>,
        track_id: impl Into<String>,
        payload: impl Into<Vec<u8>>,
        format: impl Into<String>,
        stop_reason: BrowserRecordingStopReason,
    ) -> Self {
        Self {
            recording_id: recording_id.into(),
            track_id: track_id.into(),
            payload: payload.into(),
            format: format.into(),
            recording_started_at: None,
            recording_stopped_at: None,
            stop_reason,
            configuration: None,
        }
    }

    pub fn with_timestamps(
        mut self,
        recording_started_at: impl Into<String>,
        recording_stopped_at: impl Into<String>,
    ) -> Self {
        self.recording_started_at = Some(recording_started_at.into());
        self.recording_stopped_at = Some(recording_stopped_at.into());
        self
    }

    pub fn with_configuration(mut self, configuration: RecordingConfiguration) -> Self {
        self.configuration = Some(configuration);
        self
    }

    pub fn recording_id(&self) -> &str {
        &self.recording_id
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn format(&self) -> &str {
        &self.format
    }

    pub fn recording_started_at(&self) -> Option<&str> {
        self.recording_started_at.as_deref()
    }

    pub fn recording_stopped_at(&self) -> Option<&str> {
        self.recording_stopped_at.as_deref()
    }

    pub fn stop_reason(&self) -> &BrowserRecordingStopReason {
        &self.stop_reason
    }

    /// Converts the finalized recording into the existing capture boundary.
    ///
    /// An empty finalized payload is rejected: there is no meaningful
    /// recording to hand to the artifact factory in that case.
    pub fn into_capture_result(self) -> Result<CaptureResult, BrowserRecordingHandoffError> {
        if self.payload.is_empty() {
            return Err(BrowserRecordingHandoffError::EmptyPayload);
        }

        let mut result = CaptureResult::new(self.recording_id);
        let mut track = match self.configuration {
            Some(configuration) => CaptureTrack::with_configuration(self.track_id, configuration),
            None => CaptureTrack::new(self.track_id),
        };
        track.add_chunk(recorder::audio::CaptureChunk::with_payload(1, self.payload));
        result.add_track(track);
        Ok(result)
    }

    /// Completes the neutral handoff through the existing artifact factory.
    ///
    /// This is deliberately the last responsibility of the browser boundary:
    /// persistence remains owned by the existing persistence provider.
    pub fn into_recording_artifact(
        self,
        recording_session_id: RecordingSessionId,
    ) -> Result<RecordingArtifact, BrowserRecordingHandoffError> {
        let capture_result = self.into_capture_result()?;
        Ok(RecordingArtifactFactory::create(
            capture_result,
            recording_session_id,
        ))
    }

    /// Bridges a finalized browser recording into the existing persistence
    /// boundary. The browser handoff does not own storage; it only supplies
    /// the artifact that the existing provider persists.
    pub fn persist<P: PersistenceProvider>(
        self,
        recording_session_id: RecordingSessionId,
        provider: &mut P,
    ) -> Result<RecordingArtifact, BrowserRecordingPersistenceError> {
        let artifact = self
            .into_recording_artifact(recording_session_id)
            .map_err(BrowserRecordingPersistenceError::Handoff)?;

        provider
            .store_checked(artifact)
            .map_err(BrowserRecordingPersistenceError::Persistence)
    }
}

/// Failure while crossing the browser-to-application handoff boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRecordingHandoffError {
    EmptyPayload,
}

/// Failure while bridging a browser recording into persistence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserRecordingPersistenceError {
    Handoff(BrowserRecordingHandoffError),
    Persistence(PersistenceStoreError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use recorder::persistence::{InMemoryPersistenceProvider, PersistenceLoadResult};

    #[test]
    fn finalized_browser_recording_becomes_capture_result() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-001",
            "browser-track",
            vec![1, 2, 3, 4],
            "audio/webm",
            BrowserRecordingStopReason::UserRequested,
        )
        .with_timestamps("2026-09-03T10:00:00Z", "2026-09-03T10:01:00Z");

        let result = handoff.into_capture_result().expect("handoff succeeds");

        assert_eq!(result.id(), "recording-001");
        assert_eq!(result.tracks().len(), 1);
        assert_eq!(result.tracks()[0].id.value(), "browser-track");
        assert_eq!(result.tracks()[0].chunks()[0].sequence, 1);
        assert_eq!(result.tracks()[0].chunks()[0].payload(), &[1, 2, 3, 4]);
    }

    #[test]
    fn finalized_browser_recording_uses_existing_artifact_factory() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-001",
            "browser-track",
            vec![9, 8, 7],
            "audio/webm",
            BrowserRecordingStopReason::UserRequested,
        );

        let artifact = handoff
            .into_recording_artifact(RecordingSessionId::new("session-001"))
            .expect("handoff succeeds");

        assert_eq!(artifact.id.value(), "recording-001");
        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(
            artifact.tracks()[0].chunks()[0].payload().data(),
            &[9, 8, 7]
        );
        assert_eq!(
            artifact.tracks()[0].chunks()[0]
                .payload()
                .reference()
                .value(),
            "browser-track/chunk-000001"
        );
    }

    #[test]
    fn finalized_browser_recording_can_be_persisted_through_existing_provider() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-001",
            "browser-track",
            vec![5, 6, 7],
            "audio/webm",
            BrowserRecordingStopReason::UserRequested,
        );
        let mut provider = InMemoryPersistenceProvider::new();

        let artifact = handoff
            .persist(RecordingSessionId::new("session-001"), &mut provider)
            .expect("persistence succeeds");

        assert!(matches!(
            artifact.status(),
            recorder::artifact::ArtifactStatus::Stored
        ));
        assert!(matches!(
            provider.load("recording-001"),
            PersistenceLoadResult::Valid(stored) if stored.id.value() == "recording-001"
        ));
        assert_eq!(provider.list().len(), 1);
    }

    #[test]
    fn repeated_equivalent_browser_handoff_is_idempotent_at_persistence_boundary() {
        let mut provider = InMemoryPersistenceProvider::new();

        for _ in 0..2 {
            BrowserRecordingHandoff::new(
                "recording-002",
                "browser-track",
                vec![1, 2, 3],
                "audio/webm",
                BrowserRecordingStopReason::UserRequested,
            )
            .persist(RecordingSessionId::new("session-002"), &mut provider)
            .expect("equivalent persistence succeeds");
        }

        assert_eq!(provider.list().len(), 1);
    }

    #[test]
    fn empty_finalized_recording_is_rejected_before_persistence() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-003",
            "browser-track",
            Vec::<u8>::new(),
            "audio/webm",
            BrowserRecordingStopReason::TechnicalFailure,
        );
        let mut provider = InMemoryPersistenceProvider::new();

        let result = handoff.persist(RecordingSessionId::new("session-003"), &mut provider);

        assert!(matches!(
            result,
            Err(BrowserRecordingPersistenceError::Handoff(
                BrowserRecordingHandoffError::EmptyPayload
            ))
        ));
        assert!(matches!(
            provider.load("recording-003"),
            PersistenceLoadResult::NotFound
        ));
    }

    #[test]
    fn empty_finalized_recording_is_rejected_at_handoff() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-001",
            "browser-track",
            Vec::<u8>::new(),
            "audio/webm",
            BrowserRecordingStopReason::TechnicalFailure,
        );

        assert_eq!(
            handoff.into_capture_result(),
            Err(BrowserRecordingHandoffError::EmptyPayload)
        );
    }

    #[test]
    fn technical_metadata_stays_at_neutral_boundary_when_capture_model_cannot_carry_it() {
        let handoff = BrowserRecordingHandoff::new(
            "recording-001",
            "browser-track",
            vec![7],
            "audio/webm",
            BrowserRecordingStopReason::SessionEnded,
        )
        .with_timestamps("start", "stop");

        assert_eq!(handoff.format(), "audio/webm");
        assert_eq!(handoff.recording_started_at(), Some("start"));
        assert_eq!(handoff.recording_stopped_at(), Some("stop"));
        assert_eq!(
            handoff.stop_reason(),
            &BrowserRecordingStopReason::SessionEnded
        );
    }
}
