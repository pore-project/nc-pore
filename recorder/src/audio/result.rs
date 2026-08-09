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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureChunk {
    pub sequence: u32,
}

impl CaptureChunk {
    /// Creates a capture chunk at the given sequence position.
    pub fn new(sequence: u32) -> Self {
        Self { sequence }
    }
}

/// Technical recording track produced by the capture layer.
///
/// A capture track represents one technical audio stream.
/// It does not represent a domain participant or role.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTrack {
    pub id: String,
    chunks: Vec<CaptureChunk>,
}

impl CaptureTrack {
    /// Creates an empty capture track.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            chunks: Vec::new(),
        }
    }

    /// Adds a chunk to this capture track.
    pub fn add_chunk(&mut self, chunk: CaptureChunk) {
        self.chunks.push(chunk);
    }

    /// Returns the chunks belonging to this capture track.
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
        assert_eq!(result.tracks()[0].id, "track-host");
        assert_eq!(result.tracks()[1].id, "track-guest");
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
}
