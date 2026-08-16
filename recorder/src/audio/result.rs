#![allow(dead_code)]

//! Result of a completed audio capture operation.
//!
//! CaptureResult represents the technical outcome
//! of a capture operation.
//!
//! It intentionally does not contain:
//! - artifact lifecycle rules
//! - persistence logic
//! - synchronization logic
//! - artifact model types
//!
//! See:
//! - ADR-039 Recording Architecture and Capture Boundary
//! - ADR-056 Capture Result and Recording Artifact Data Boundary

use super::RecordingConfiguration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureChunk {
    pub sequence: u32,
    payload: Vec<u8>,
}

impl CaptureChunk {
    /// Creates a capture chunk without payload data.
    pub fn new(sequence: u32) -> Self {
        Self {
            sequence,
            payload: Vec::new(),
        }
    }

    /// Creates a capture chunk with the technical payload bytes produced by capture.
    pub fn with_payload(sequence: u32, payload: impl Into<Vec<u8>>) -> Self {
        Self {
            sequence,
            payload: payload.into(),
        }
    }

    /// Returns the payload bytes produced by the capture layer.
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// Capture Track identity.
///
/// CaptureTrackId represents the technical identity
/// of a CaptureTrack and is intentionally distinct
/// from RecordingTrackId.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTrackId(String);

impl CaptureTrackId {
    /// Creates a new capture track identifier.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the raw identifier value.
    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Technical recording track produced by the capture layer.
///
/// A capture track represents one technical audio stream.
/// It does not represent a domain participant or role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTrack {
    pub id: CaptureTrackId,
    configuration: Option<RecordingConfiguration>,
    chunks: Vec<CaptureChunk>,
}

impl CaptureTrack {
    /// Creates an empty capture track without configuration metadata.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: CaptureTrackId::new(id),
            configuration: None,
            chunks: Vec::new(),
        }
    }

    /// Creates an empty capture track with the supplied recording configuration.
    pub fn with_configuration(
        id: impl Into<String>,
        configuration: RecordingConfiguration,
    ) -> Self {
        Self {
            id: CaptureTrackId::new(id),
            configuration: Some(configuration),
            chunks: Vec::new(),
        }
    }

    /// Returns the recording configuration used for this capture track, if known.
    pub const fn configuration(&self) -> Option<RecordingConfiguration> {
        self.configuration
    }

    /// Adds a technical capture chunk.
    pub fn add_chunk(&mut self, chunk: CaptureChunk) {
        self.chunks.push(chunk);
    }

    /// Returns the chunks belonging to the capture track.
    pub fn chunks(&self) -> &[CaptureChunk] {
        &self.chunks
    }
}

/// Result of a completed audio capture operation.
///
/// CaptureResult contains technical recording data produced by
/// the capture layer. It does not represent a RecordingArtifact.
///
/// The capture-side data types are intentionally separate from
/// the artifact-side RecordingTrack and RecordingChunk types.
///
/// See ADR-056.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureResult {
    id: String,
    tracks: Vec<CaptureTrack>,
}

impl CaptureResult {
    /// Creates a new capture result.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            tracks: Vec::new(),
        }
    }

    /// Returns the identifier of the capture result.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Adds a technical capture track.
    pub fn add_track(&mut self, track: CaptureTrack) {
        self.tracks.push(track);
    }

    /// Returns the technical capture tracks.
    pub fn tracks(&self) -> &[CaptureTrack] {
        &self.tracks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SampleFormat;

    // TEST-31
    //
    // Protects ADR-056:
    // CaptureResult can represent technical recording tracks
    // without depending on artifact model types.
    #[test]
    fn capture_result_can_contain_tracks() {
        let mut result = CaptureResult::new("capture-001");

        result.add_track(CaptureTrack::new("track-host"));
        result.add_track(CaptureTrack::new("track-guest"));

        assert_eq!(result.tracks().len(), 2);
        assert_eq!(result.tracks()[0].id.value(), "track-host");
        assert_eq!(result.tracks()[1].id.value(), "track-guest");
    }

    // TEST-32
    //
    // Protects ADR-056:
    // A capture track can contain multiple ordered chunks.
    #[test]
    fn capture_track_can_contain_ordered_chunks() {
        let mut track = CaptureTrack::new("track-host");

        track.add_chunk(CaptureChunk::new(1));
        track.add_chunk(CaptureChunk::new(2));
        track.add_chunk(CaptureChunk::new(3));

        assert_eq!(track.chunks().len(), 3);
        assert_eq!(track.chunks()[0].sequence, 1);
        assert_eq!(track.chunks()[1].sequence, 2);
        assert_eq!(track.chunks()[2].sequence, 3);
    }

    // TEST-33
    //
    // Protects ADR-056:
    // Capture tracks remain independent technical structures.
    #[test]
    fn capture_result_preserves_independent_track_data() {
        let mut result = CaptureResult::new("capture-001");

        let mut host_track = CaptureTrack::new("track-host");
        host_track.add_chunk(CaptureChunk::new(1));
        host_track.add_chunk(CaptureChunk::new(2));

        let mut guest_track = CaptureTrack::new("track-guest");
        guest_track.add_chunk(CaptureChunk::new(1));
        guest_track.add_chunk(CaptureChunk::new(2));
        guest_track.add_chunk(CaptureChunk::new(3));

        result.add_track(host_track);
        result.add_track(guest_track);

        assert_eq!(result.tracks().len(), 2);

        assert_eq!(result.tracks()[0].chunks().len(), 2);
        assert_eq!(result.tracks()[0].chunks()[0].sequence, 1);
        assert_eq!(result.tracks()[0].chunks()[1].sequence, 2);

        assert_eq!(result.tracks()[1].chunks().len(), 3);
        assert_eq!(result.tracks()[1].chunks()[0].sequence, 1);
        assert_eq!(result.tracks()[1].chunks()[1].sequence, 2);
        assert_eq!(result.tracks()[1].chunks()[2].sequence, 3);
    }

    // TEST-34
    //
    // Protects the payload part of the capture boundary:
    // CaptureChunk can carry technical payload bytes without
    // depending on artifact model types.
    #[test]
    fn capture_chunk_can_carry_payload_bytes() {
        let chunk = CaptureChunk::with_payload(1, vec![1, 2, 3, 4]);

        assert_eq!(chunk.sequence, 1);
        assert_eq!(chunk.payload(), &[1, 2, 3, 4]);
    }

    // TEST-35
    //
    // Protects the capture boundary:
    // The technical configuration used for a capture track remains
    // attached to the resulting track.
    #[test]
    fn capture_track_preserves_recording_configuration() {
        let configuration = RecordingConfiguration::new(44_100, 2, SampleFormat::F32);
        let track = CaptureTrack::with_configuration("track-host", configuration);

        assert_eq!(track.configuration(), Some(configuration));
    }
}

#[cfg(test)]
mod capture_track_id_tests {
    use super::*;

    // TEST-36
    #[test]
    fn capture_track_id_preserves_value() {
        let id = CaptureTrackId::new("track-001");

        assert_eq!(id.value(), "track-001");
    }
}
