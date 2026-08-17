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
    pub fn new(sequence: u32) -> Self {
        Self { sequence, payload: Vec::new() }
    }

    pub fn with_payload(sequence: u32, payload: impl Into<Vec<u8>>) -> Self {
        Self { sequence, payload: payload.into() }
    }

    pub fn payload(&self) -> &[u8] { &self.payload }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTrackId(String);

impl CaptureTrackId {
    pub fn new(value: impl Into<String>) -> Self { Self(value.into()) }
    pub fn value(&self) -> &str { &self.0 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTrack {
    pub id: CaptureTrackId,
    configuration: Option<RecordingConfiguration>,
    chunks: Vec<CaptureChunk>,
}

impl CaptureTrack {
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: CaptureTrackId::new(id), configuration: None, chunks: Vec::new() }
    }

    pub fn with_configuration(id: impl Into<String>, configuration: RecordingConfiguration) -> Self {
        Self { id: CaptureTrackId::new(id), configuration: Some(configuration), chunks: Vec::new() }
    }

    pub const fn configuration(&self) -> Option<RecordingConfiguration> { self.configuration }
    pub fn add_chunk(&mut self, chunk: CaptureChunk) { self.chunks.push(chunk); }
    pub fn chunks(&self) -> &[CaptureChunk] { &self.chunks }
}

/// Technical outcome of a capture operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureStatus {
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureResult {
    id: String,
    status: CaptureStatus,
    tracks: Vec<CaptureTrack>,
}

impl CaptureResult {
    /// Creates a successfully completed capture result.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), status: CaptureStatus::Completed, tracks: Vec::new() }
    }

    /// Creates a capture result representing a technical capture failure.
    pub fn failed(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self { id: id.into(), status: CaptureStatus::Failed(error.into()), tracks: Vec::new() }
    }

    pub fn id(&self) -> &str { &self.id }
    pub fn status(&self) -> &CaptureStatus { &self.status }
    pub fn add_track(&mut self, track: CaptureTrack) { self.tracks.push(track); }
    pub fn tracks(&self) -> &[CaptureTrack] { &self.tracks }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::audio::SampleFormat;

    #[test]
    fn capture_result_can_contain_tracks() {
        let mut result = CaptureResult::new("capture-001");
        result.add_track(CaptureTrack::new("track-host"));
        result.add_track(CaptureTrack::new("track-guest"));
        assert_eq!(result.tracks().len(), 2);
        assert_eq!(result.tracks()[0].id.value(), "track-host");
        assert_eq!(result.tracks()[1].id.value(), "track-guest");
    }

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
        assert_eq!(result.tracks()[1].chunks().len(), 3);
    }

    #[test]
    fn capture_chunk_can_carry_payload_bytes() {
        let chunk = CaptureChunk::with_payload(1, vec![1, 2, 3, 4]);
        assert_eq!(chunk.sequence, 1);
        assert_eq!(chunk.payload(), &[1, 2, 3, 4]);
    }

    #[test]
    fn capture_track_preserves_recording_configuration() {
        let configuration = RecordingConfiguration::new(44_100, 2, SampleFormat::F32);
        let track = CaptureTrack::with_configuration("track-host", configuration);
        assert_eq!(track.configuration(), Some(configuration));
    }

    #[test]
    fn capture_result_defaults_to_completed() {
        let result = CaptureResult::new("capture-001");
        assert_eq!(result.status(), &CaptureStatus::Completed);
    }

    #[test]
    fn capture_result_can_represent_technical_failure() {
        let result = CaptureResult::failed("capture-001", "input stream failed");
        assert_eq!(result.status(), &CaptureStatus::Failed("input stream failed".to_string()));
        assert!(result.tracks().is_empty());
    }
}

#[cfg(test)]
mod capture_track_id_tests {
    use super::*;

    #[test]
    fn capture_track_id_preserves_value() {
        let id = CaptureTrackId::new("track-001");
        assert_eq!(id.value(), "track-001");
    }
}
