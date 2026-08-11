//! Recording Artifact factory.
//!
//! This module creates RecordingArtifact instances
//! from completed capture results.
//!
//! It intentionally does not contain:
//! - workflow coordination
//! - capture logic
//! - persistence logic
//! - registry management
//!
//! See:
//! - ADR-050 Recording Artifact Factory

use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use crate::session::RecordingSessionId;
use crate::audio::CaptureResult;

/// Creates RecordingArtifact instances.
///
/// The factory encapsulates artifact construction
/// and keeps creation logic separate from workflow coordination.
pub struct RecordingArtifactFactory;

impl RecordingArtifactFactory {
    /// Creates a new RecordingArtifact from a capture result.
    pub fn create(
        capture_result: CaptureResult,
        recording_session_id: RecordingSessionId,
    ) -> RecordingArtifact {
        let mut artifact = RecordingArtifact::new(capture_result.id(), recording_session_id);

        for capture_track in capture_result.tracks() {
            let mut recording_track = RecordingTrack::new(&capture_track.id);

            for capture_chunk in capture_track.chunks() {
                recording_track.add_chunk(RecordingChunk::new(capture_chunk.sequence));
            }

            artifact.add_track(recording_track);
        }

        artifact
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::{CaptureChunk, CaptureResult, CaptureTrack};

    // TEST-22
    //
    // Protects ADR-050:
    // Artifact creation is encapsulated in the factory.
    #[test]
    fn factory_creates_artifact_from_capture_result() {
        let capture_result = CaptureResult::new("capture-001");

        let artifact = RecordingArtifactFactory::create(capture_result, RecordingSessionId::new("session-001"));

        assert_eq!(artifact.id.value(), "capture-001");
        assert_eq!(artifact.recording_session_id.value(), "session-001");
    }
    #[test]
    fn factory_transfers_tracks_and_chunks() {
        let mut capture = CaptureResult::new("capture-001");

        let mut host = CaptureTrack::new("track-host");
        host.add_chunk(CaptureChunk::new(1));
        host.add_chunk(CaptureChunk::new(2));

        let mut guest = CaptureTrack::new("track-guest");
        guest.add_chunk(CaptureChunk::new(1));
        guest.add_chunk(CaptureChunk::new(2));
        guest.add_chunk(CaptureChunk::new(3));

        capture.add_track(host);
        capture.add_track(guest);

        let artifact = RecordingArtifactFactory::create(capture, RecordingSessionId::new("session-001"));

        assert_eq!(artifact.tracks().len(), 2);
        assert_eq!(artifact.tracks()[0].id.value(), "track-host");
        assert_eq!(artifact.tracks()[0].chunks()[0].sequence, 1);
        assert_eq!(artifact.tracks()[0].chunks()[1].sequence, 2);

        assert_eq!(artifact.tracks()[1].id.value(), "track-guest");
        assert_eq!(artifact.tracks()[1].chunks()[0].sequence, 1);
        assert_eq!(artifact.tracks()[1].chunks()[1].sequence, 2);
        assert_eq!(artifact.tracks()[1].chunks()[2].sequence, 3);
    }
}
