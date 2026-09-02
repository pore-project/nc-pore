//! Recording Artifact factory.
//!
//! This module creates RecordingArtifact instances
//! from preserved local captures.
//!
//! It intentionally does not contain:
//! - workflow coordination
//! - capture logic
//! - preservation logic
//! - persistence logic
//! - registry management
//!
//! See:
//! - ADR-050 Recording Artifact Factory
//! - ADR-058 Recording Payload Representation

use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use crate::preservation::PreservedCapture;
use crate::session::RecordingSessionId;

/// Creates RecordingArtifact instances from preserved captures.
pub struct RecordingArtifactFactory;

impl RecordingArtifactFactory {
    /// Creates a new RecordingArtifact from a preserved local capture.
    pub fn create(
        preserved_capture: PreservedCapture,
        recording_session_id: RecordingSessionId,
    ) -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new(preserved_capture.id(), recording_session_id);

        for capture_track in preserved_capture.tracks() {
            let mut recording_track = match capture_track.configuration() {
                Some(configuration) => {
                    RecordingTrack::with_configuration(capture_track.id.value(), configuration)
                }
                None => RecordingTrack::new(capture_track.id.value()),
            };

            if let Some(provenance) = capture_track.source_provenance() {
                recording_track.set_source_provenance(provenance.clone());
            }

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
    use crate::audio::{
        CaptureChunk, CaptureResult, CaptureSourceProvenance, CaptureTrack, RecordingConfiguration,
        SampleFormat,
    };
    use crate::preservation::CapturePreserver;

    // TEST-22
    //
    // Protects ADR-050:
    // Artifact creation is encapsulated in the factory.
    #[test]
    fn factory_creates_artifact_from_preserved_capture() {
        let capture_result = CaptureResult::new("capture-001");
        let preserved = CapturePreserver::preserve(capture_result);

        let artifact =
            RecordingArtifactFactory::create(preserved, RecordingSessionId::new("session-001"));

        assert_eq!(artifact.id.value(), "capture-001");
        assert_eq!(artifact.recording_session_id.value(), "session-001");
    }

    // TEST-23
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
        let preserved = CapturePreserver::preserve(capture);

        let artifact =
            RecordingArtifactFactory::create(preserved, RecordingSessionId::new("session-001"));

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
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::F32);
        let mut track = CaptureTrack::with_configuration("track-host", configuration);
        track.add_chunk(CaptureChunk::with_payload(1, vec![10, 20, 30]));
        capture.add_track(track);
        let preserved = CapturePreserver::preserve(capture);

        let artifact =
            RecordingArtifactFactory::create(preserved, RecordingSessionId::new("session-001"));

        let recording_track = &artifact.tracks()[0];
        assert_eq!(recording_track.configuration(), Some(configuration));

        let chunk = &recording_track.chunks()[0];
        assert_eq!(
            chunk.payload().reference().value(),
            "track-host/chunk-000001"
        );
        assert_eq!(chunk.payload().data(), &[10, 20, 30]);
        assert_eq!(chunk.payload().size_bytes(), 3);
    }

    // TEST-41
    //
    // Protects the host-neutral capture boundary:
    // source provenance is transferred unchanged from capture to artifact.
    #[test]
    fn factory_transfers_source_provenance() {
        let provenance = CaptureSourceProvenance::new("device-1", 1_762_000_000_000)
            .with_label("Microphone")
            .ended_at(1_762_000_005_000);
        let mut capture = CaptureResult::new("capture-001");
        let mut track = CaptureTrack::new("track-host");
        track.set_source_provenance(provenance.clone());
        capture.add_track(track);
        let preserved = CapturePreserver::preserve(capture);

        let artifact =
            RecordingArtifactFactory::create(preserved, RecordingSessionId::new("session-001"));

        assert_eq!(artifact.tracks()[0].source_provenance(), Some(&provenance));
    }
}
