//! Application boundary for browser-finalized recording artifacts.
//!
//! The browser is a capture client. It hands one locally finalized payload to
//! this boundary together with the authoritative Core identities and the
//! recorder-side technical identities. The boundary adapts that payload to the
//! existing RecordingArtifactProcessor; it does not introduce a second
//! persistence or synchronization path.

use nc_pore_core::identity::ProductionId;
use recorder::artifact::coordination::ArtifactCoordinator;
use recorder::artifact::processing::RecordingArtifactProcessor;
use recorder::artifact::{RecordingArtifact, RecordingArtifactAssociation};
use recorder::audio::{
    CaptureChunk, CaptureResult, CaptureTrack, RecordingConfiguration, SampleFormat,
};
use recorder::persistence::{PersistenceProvider, PersistenceStoreError};
use recorder::session::RecordingSessionId;

/// One browser-local finalized artifact handed to the application boundary.
///
/// `capture_id` and `recording_session_id` are technical identities and must
/// not be confused with `ProductionId` or the domain `RecordingId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserRecordingArtifact {
    capture_id: String,
    recording_session_id: String,
    production_id: ProductionId,
    recording_id: String,
    track_id: String,
    sample_rate_hz: u32,
    channels: u16,
    payload: Vec<u8>,
}

impl BrowserRecordingArtifact {
    pub fn new(
        capture_id: impl Into<String>,
        recording_session_id: impl Into<String>,
        production_id: ProductionId,
        recording_id: impl Into<String>,
        track_id: impl Into<String>,
        sample_rate_hz: u32,
        channels: u16,
        payload: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            capture_id: capture_id.into(),
            recording_session_id: recording_session_id.into(),
            production_id,
            recording_id: recording_id.into(),
            track_id: track_id.into(),
            sample_rate_hz,
            channels,
            payload: payload.into(),
        }
    }

    pub fn capture_id(&self) -> &str { &self.capture_id }
    pub fn recording_session_id(&self) -> &str { &self.recording_session_id }
    pub fn production_id(&self) -> &ProductionId { &self.production_id }
    pub fn recording_id(&self) -> &str { &self.recording_id }
    pub fn track_id(&self) -> &str { &self.track_id }
    pub fn sample_rate_hz(&self) -> u32 { self.sample_rate_hz }
    pub fn channels(&self) -> u16 { self.channels }
    pub fn payload(&self) -> &[u8] { &self.payload }

    /// Adapts the browser payload to the existing recorder capture boundary.
    ///
    /// The browser V1 recorder emits one finalized WAV payload, so this
    /// boundary represents it as one technical capture track with one chunk.
    /// No Core identity is reused as a technical artifact/session identity.
    pub fn into_capture_result(&self) -> CaptureResult {
        let mut result = CaptureResult::new(self.capture_id.clone());
        let configuration = RecordingConfiguration::new(
            self.sample_rate_hz,
            self.channels,
            SampleFormat::Pcm24,
        );
        let mut track = CaptureTrack::with_configuration(self.track_id.clone(), configuration);
        track.add_chunk(CaptureChunk::with_payload(1, self.payload.clone()));
        result.add_track(track);
        result
    }
}

/// Persists a browser-finalized artifact through the existing recorder path.
///
/// This is deliberately a thin application adapter:
/// Browser artifact -> CaptureResult -> RecordingArtifactProcessor ->
/// PersistenceProvider. Synchronization remains downstream of persistence.
pub fn persist_browser_recording_artifact<P>(
    processor: &mut RecordingArtifactProcessor<P>,
    artifact: BrowserRecordingArtifact,
) -> Result<RecordingArtifact, PersistenceStoreError>
where
    P: PersistenceProvider,
{
    let association = RecordingArtifactAssociation::new(
        artifact.production_id().value(),
        artifact.recording_id(),
    );
    let recording_session_id = RecordingSessionId::new(artifact.recording_session_id());
    let capture_result = artifact.into_capture_result();

    processor.process(capture_result, recording_session_id, association)
}

/// Convenience constructor for the processor used by concrete application
/// composition roots. It does not create a new persistence abstraction.
pub fn browser_artifact_processor<P>(persistence: P) -> RecordingArtifactProcessor<P>
where
    P: PersistenceProvider,
{
    RecordingArtifactProcessor::new(ArtifactCoordinator::new(persistence))
}

#[cfg(test)]
mod tests {
    use super::*;
    use recorder::artifact::ArtifactStatus;
    use recorder::persistence::InMemoryPersistenceProvider;

    // TEST-41
    #[test]
    fn browser_artifact_maps_technical_and_domain_identities_without_aliasing_them() {
        let artifact = BrowserRecordingArtifact::new(
            "capture-browser-001",
            "recorder-session-001",
            ProductionId::new("production-001"),
            "recording-001",
            "browser-track-001",
            48_000,
            1,
            vec![1, 2, 3, 4],
        );
        let production_id = artifact.production_id().clone();
        let recording_id = artifact.recording_id().to_owned();
        let recording_session_id = artifact.recording_session_id().to_owned();
        let capture_id = artifact.capture_id().to_owned();
        let capture = artifact.into_capture_result();

        assert_eq!(capture.id(), "capture-browser-001");
        assert_eq!(production_id.value(), "production-001");
        assert_eq!(recording_id, "recording-001");
        assert_eq!(recording_session_id, "recorder-session-001");
        assert_ne!(capture_id, recording_id);
        assert_eq!(capture.tracks()[0].chunks()[0].payload(), &[1, 2, 3, 4]);
    }

    // TEST-42
    #[test]
    fn browser_artifact_uses_existing_persistence_boundary() {
        let persistence = InMemoryPersistenceProvider::new();
        let mut processor = browser_artifact_processor(persistence);
        let submission = BrowserRecordingArtifact::new(
            "capture-browser-002",
            "recorder-session-002",
            ProductionId::new("production-002"),
            "recording-002",
            "browser-track-002",
            48_000,
            1,
            vec![10, 20, 30],
        );

        let stored = persist_browser_recording_artifact(&mut processor, submission)
            .expect("browser artifact should use the existing persistence path");

        assert_eq!(stored.id.value(), "capture-browser-002");
        assert_eq!(stored.status(), &ArtifactStatus::Stored);
        assert_eq!(stored.production_id(), Some("production-002"));
        assert_eq!(stored.recording_id(), Some("recording-002"));
    }
}
