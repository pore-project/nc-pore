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
//! - ADR-058 Recording Payload Representation

use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use crate::audio::CaptureResult;
use crate::session::RecordingSessionId;

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
            let mut recording_track = RecordingTrack::new(capture_track.id.value());

            for capture_chunk in capture_track.chunks() {
                // The logical reference remains independent from the concrete
                // filesystem path used later by the persistence provider.
                let reference = format!(
                    "{}/chunk-{sequence:06}",
                    capture_track.id.value(),
                    sequence = capture_chunk.sequence
                );

                recording_track.add_chunk(RecordingChunk::with_payload(
                    capture_chunk.sequence,
                    reference,
                    capture_chunk.payload().to_vec(),
                ));
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

        let artifact = RecordingArtifactFactory::create(
            capture_result,
            RecordingSessionId::new("session-001"),
        );

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

        let artifact =
            RecordingArtifactFactory::create(capture, RecordingSessionId::new("session-001"));

        assert_eq!(artifact.tracks().len(), 2);
        assert_eq!(artifact.tracks()[0].id.value(), "track-host");
        assert_eq!(artifact.tracks()[0].chunks()[0].sequence, 1);
        assert_eq!(artifact.tracks()[0].chunks()[1].sequence, 2);

        assert_eq!(artifact.tracks()[1].id.value(), "track-guest");
        assert_eq!(artifact.tracks()[1].chunks()[0].sequence, 1);
        assert_eq!(artifact.tracks()[1].chunks()[1].sequence, 2);
        assert_eq!(artifact.tracks()[1].chunks()[2].sequence, 3);
    }

    // TEST-36
    //
    // Protects ADR-056 and ADR-058:
    // The factory transfers technical payload data without coupling
    // the capture types to artifact or persistence types.
    #[test]
    fn factory_transfers_chunk_payload_and_assigns_logical_reference() {
        let mut capture = CaptureResult::new("capture-001");
        let mut track = CaptureTrack::new("track-host");
        track.add_chunk(CaptureChunk::with_payload(1, vec![10, 20, 30]));
        capture.add_track(track);

        let artifact =
            RecordingArtifactFactory::create(capture, RecordingSessionId::new("session-001"));

        let chunk = &artifact.tracks()[0].chunks()[0];
        assert_eq!(chunk.payload().reference().value(), "track-host/chunk-000001");
        assert_eq!(chunk.payload().data(), &[10, 20, 30]);
        assert_eq!(chunk.payload().size_bytes(), 3);
    }
}
