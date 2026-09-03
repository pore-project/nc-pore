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

/// Provenance of the local capture source used for one technical track.
///
/// The source identifier is intentionally opaque. It may identify a browser
/// device, an operating-system input, or another local source without making
/// the capture model depend on a particular host integration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureSourceProvenance {
    source_id: String,
    label: Option<String>,
    started_at_unix_ms: u64,
    ended_at_unix_ms: Option<u64>,
}

impl CaptureSourceProvenance {
    /// Creates provenance for a source starting at the supplied Unix timestamp.
    pub fn new(source_id: impl Into<String>, started_at_unix_ms: u64) -> Self {
        Self {
            source_id: source_id.into(),
            label: None,
            started_at_unix_ms,
            ended_at_unix_ms: None,
        }
    }

    /// Adds an optional human-readable source label.
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Marks the end of the source interval.
    pub fn ended_at(mut self, ended_at_unix_ms: u64) -> Self {
        self.ended_at_unix_ms = Some(ended_at_unix_ms);
        self
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn label(&self) -> Option<&str> {
        self.label.as_deref()
    }

    pub const fn started_at_unix_ms(&self) -> u64 {
        self.started_at_unix_ms
    }

    pub const fn ended_at_unix_ms(&self) -> Option<u64> {
        self.ended_at_unix_ms
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
    source_provenance: Option<CaptureSourceProvenance>,
    chunks: Vec<CaptureChunk>,
}

impl CaptureTrack {
    /// Creates an empty capture track without configuration metadata.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: CaptureTrackId::new(id),
            configuration: None,
            source_provenance: None,
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
            source_provenance: None,
            chunks: Vec::new(),
        }
    }

    /// Returns the recording configuration used for this capture track, if known.
    pub const fn configuration(&self) -> Option<RecordingConfiguration> {
        self.configuration
    }

    /// Attaches local capture-source provenance to this technical track.
    pub fn set_source_provenance(&mut self, provenance: CaptureSourceProvenance) {
        self.source_provenance = Some(provenance);
    }

    /// Returns local capture-source provenance, if known.
    pub fn source_provenance(&self) -> Option<&CaptureSourceProvenance> {
        self.source_provenance.as_ref()
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

/// Technical outcome of a capture operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CaptureStatus {
    Completed,
    Failed(String),
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
    status: CaptureStatus,
    tracks: Vec<CaptureTrack>,
}

impl CaptureResult {
    /// Creates a successfully completed capture result.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: CaptureStatus::Completed,
            tracks: Vec::new(),
        }
    }

    /// Creates a capture result representing a technical capture failure.
    pub fn failed(id: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            status: CaptureStatus::Failed(error.into()),
            tracks: Vec::new(),
        }
    }

    /// Returns the identifier of the capture result.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the technical outcome of the capture operation.
    pub fn status(&self) -> &CaptureStatus {
        &self.status
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

    // TEST-39
    //
    // Protects the host-neutral capture boundary:
    // Source provenance remains opaque and preserves the source interval.
    #[test]
    fn capture_track_preserves_source_provenance() {
        let provenance = CaptureSourceProvenance::new("device-1", 1_762_000_000_000)
            .with_label("Microphone")
            .ended_at(1_762_000_005_000);
        let mut track = CaptureTrack::new("track-host");
        track.set_source_provenance(provenance.clone());

        assert_eq!(track.source_provenance(), Some(&provenance));
        assert_eq!(provenance.source_id(), "device-1");
        assert_eq!(provenance.label(), Some("Microphone"));
        assert_eq!(provenance.started_at_unix_ms(), 1_762_000_000_000);
        assert_eq!(provenance.ended_at_unix_ms(), Some(1_762_000_005_000));
    }

    #[test]
    fn capture_result_defaults_to_completed() {
        let result = CaptureResult::new("capture-001");
        assert_eq!(result.status(), &CaptureStatus::Completed);
    }

    #[test]
    fn capture_result_can_represent_technical_failure() {
        let result = CaptureResult::failed("capture-001", "input stream failed");
        assert_eq!(
            result.status(),
            &CaptureStatus::Failed("input stream failed".to_string())
        );
        assert!(result.tracks().is_empty());
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
