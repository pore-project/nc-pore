#![allow(dead_code)]

//! Recording Artifact model.
//!
//! This module represents the technical result of a local recording.
//!
//! A Recording Artifact is intentionally separated from:
//! - production domain objects
//! - storage implementations
//! - synchronization logic
//! - export processing
//!
//! See:
//! - ADR-042 Recording Artifact Model and Lifecycle Boundary
//! - ADR-054 Recording Artifact and Local Recording Data Association
//! - ADR-058 Recording Payload Representation
//! - ADR-038 Reconstructable Capture Artifacts

pub mod coordination;
pub mod factory;
pub mod id;
pub mod integrity;
pub mod processing;
pub mod recovery;
pub mod registry;

pub use id::{ArtifactId, RecordingTrackId};
pub use integrity::{ManifestHash, PayloadHash};

use crate::audio::{RecordingChunkDuration, RecordingConfiguration, SampleFormat};
use crate::session::RecordingSessionId;

/// Technical lifecycle state of a Recording Artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactStatus {
    Created,
    Available,
    Stored,
}

/// Opaque reference to the domain recording context that produced an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingArtifactAssociation {
    production_id: String,
    recording_id: String,
}

impl RecordingArtifactAssociation {
    pub fn new(production_id: impl Into<String>, recording_id: impl Into<String>) -> Self {
        Self {
            production_id: production_id.into(),
            recording_id: recording_id.into(),
        }
    }

    pub fn production_id(&self) -> &str {
        &self.production_id
    }

    pub fn recording_id(&self) -> &str {
        &self.recording_id
    }
}

/// Storage-provider-independent logical identity of one payload segment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingPayloadReference(String);

impl RecordingPayloadReference {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

/// Technical payload belonging to one RecordingChunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingPayload {
    reference: RecordingPayloadReference,
    data: Vec<u8>,
    hash: PayloadHash,
}

impl RecordingPayload {
    pub fn new(reference: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        let data = data.into();
        let hash = PayloadHash::from_bytes(&data);

        Self {
            reference: RecordingPayloadReference::new(reference),
            data,
            hash,
        }
    }

    pub fn reference(&self) -> &RecordingPayloadReference {
        &self.reference
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size_bytes(&self) -> u64 {
        self.data.len() as u64
    }

    /// Returns the SHA-256 hash of the payload bytes.
    pub fn hash(&self) -> &PayloadHash {
        &self.hash
    }
}

/// A technical chunk of recording data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingChunk {
    pub sequence: u32,
    sample_offset: u64,
    payload: RecordingPayload,
}

impl RecordingChunk {
    /// Creates a recording chunk without payload data at sample offset zero.
    pub fn new(sequence: u32) -> Self {
        Self::with_sample_offset(sequence, 0, format!("chunk-{sequence:06}"), Vec::new())
    }

    /// Creates a recording chunk with its logical payload reference and data.
    pub fn with_payload(
        sequence: u32,
        reference: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) -> Self {
        Self::with_sample_offset(sequence, 0, reference, data)
    }

    /// Creates a recording chunk with an explicit sample-based position.
    pub fn with_sample_offset(
        sequence: u32,
        sample_offset: u64,
        reference: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            sequence,
            sample_offset,
            payload: RecordingPayload::new(reference, data),
        }
    }

    pub const fn sample_offset(&self) -> u64 {
        self.sample_offset
    }

    pub fn payload(&self) -> &RecordingPayload {
        &self.payload
    }
}

/// A technical recording track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingTrack {
    pub id: RecordingTrackId,
    configuration: Option<RecordingConfiguration>,
    chunks: Vec<RecordingChunk>,
}

impl RecordingTrack {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingTrackId::new(id),
            configuration: None,
            chunks: Vec::new(),
        }
    }

    pub fn with_configuration(
        id: impl Into<String>,
        configuration: RecordingConfiguration,
    ) -> Self {
        Self {
            id: RecordingTrackId::new(id),
            configuration: Some(configuration),
            chunks: Vec::new(),
        }
    }

    pub const fn configuration(&self) -> Option<RecordingConfiguration> {
        self.configuration
    }

    pub fn add_chunk(&mut self, chunk: RecordingChunk) {
        self.chunks.push(chunk);
    }

    pub fn chunks(&self) -> &[RecordingChunk] {
        &self.chunks
    }
}

/// Technical representation of the result of a local recording.
#[derive(Debug, Clone)]
pub struct RecordingArtifact {
    pub id: ArtifactId,
    pub recording_session_id: RecordingSessionId,
    status: ArtifactStatus,
    tracks: Vec<RecordingTrack>,
    association: Option<RecordingArtifactAssociation>,
}

impl RecordingArtifact {
    pub fn new(id: impl Into<String>, recording_session_id: RecordingSessionId) -> Self {
        Self {
            id: ArtifactId::new(id),
            recording_session_id,
            status: ArtifactStatus::Created,
            tracks: Vec::new(),
            association: None,
        }
    }

    pub fn status(&self) -> &ArtifactStatus {
        &self.status
    }

    pub fn make_available(&mut self) {
        self.status = ArtifactStatus::Available;
    }

    pub fn store(&mut self) {
        self.status = ArtifactStatus::Stored;
    }

    pub fn add_track(&mut self, track: RecordingTrack) {
        self.tracks.push(track);
    }

    pub fn tracks(&self) -> &[RecordingTrack] {
        &self.tracks
    }

    /// Returns the deterministic integrity hash of the technical artifact manifest.
    ///
    /// Lifecycle status is deliberately excluded. The manifest covers the
    /// artifact identity, recording session, domain association, track
    /// configuration, chunk positions, payload references and payload hashes.
    pub fn manifest_hash(&self) -> ManifestHash {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"NC-PORE:recording-artifact-manifest:v1\0");

        append_str(&mut bytes, self.id.value());
        append_str(&mut bytes, self.recording_session_id.value());

        match &self.association {
            Some(association) => {
                bytes.push(1);
                append_str(&mut bytes, association.production_id());
                append_str(&mut bytes, association.recording_id());
            }
            None => bytes.push(0),
        }

        let mut tracks: Vec<&RecordingTrack> = self.tracks.iter().collect();
        tracks.sort_by(|left, right| left.id.value().cmp(right.id.value()));
        append_u32(&mut bytes, tracks.len() as u32);

        for track in tracks {
            append_str(&mut bytes, track.id.value());

            match track.configuration() {
                Some(configuration) => {
                    bytes.push(1);
                    append_u32(&mut bytes, configuration.sample_rate_hz());
                    append_u16(&mut bytes, configuration.channels());
                    bytes.push(match configuration.sample_format() {
                        SampleFormat::Pcm24 => 1,
                        SampleFormat::F32 => 2,
                    });
                    append_chunk_duration(&mut bytes, configuration.chunk_duration());
                }
                None => bytes.push(0),
            }

            let mut chunks: Vec<&RecordingChunk> = track.chunks().iter().collect();
            chunks.sort_by_key(|chunk| chunk.sequence);
            append_u32(&mut bytes, chunks.len() as u32);

            for chunk in chunks {
                append_u32(&mut bytes, chunk.sequence);
                append_u64(&mut bytes, chunk.sample_offset());
                append_str(&mut bytes, chunk.payload().reference().value());
                bytes.extend_from_slice(chunk.payload().hash().as_bytes());
            }
        }

        ManifestHash::from_manifest_bytes(&bytes)
    }

    pub fn set_domain_association(
        &mut self,
        production_id: impl Into<String>,
        recording_id: impl Into<String>,
    ) {
        self.association = Some(RecordingArtifactAssociation::new(
            production_id,
            recording_id,
        ));
    }

    pub fn association(&self) -> Option<&RecordingArtifactAssociation> {
        self.association.as_ref()
    }

    pub fn production_id(&self) -> Option<&str> {
        self.association
            .as_ref()
            .map(RecordingArtifactAssociation::production_id)
    }

    pub fn recording_id(&self) -> Option<&str> {
        self.association
            .as_ref()
            .map(RecordingArtifactAssociation::recording_id)
    }
}

fn append_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn append_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn append_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_be_bytes());
}

fn append_str(bytes: &mut Vec<u8>, value: &str) {
    append_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

fn append_chunk_duration(bytes: &mut Vec<u8>, duration: RecordingChunkDuration) {
    append_u32(bytes, duration.seconds());
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-09
    #[test]
    fn new_artifact_starts_as_created() {
        let artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        assert_eq!(artifact.status(), &ArtifactStatus::Created);
    }

    // TEST-10
    #[test]
    fn artifact_can_become_available() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();

        assert_eq!(artifact.status(), &ArtifactStatus::Available);
    }

    // TEST-11
    #[test]
    fn artifact_can_be_stored() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();
        artifact.store();

        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
    }

    // TEST-28
    #[test]
    fn artifact_can_contain_tracks() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.add_track(RecordingTrack::new("track-001"));

        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].id.value(), "track-001");
    }

    // TEST-29
    #[test]
    fn track_can_contain_chunks() {
        let mut track = RecordingTrack::new("track-001");

        track.add_chunk(RecordingChunk::new(1));
        track.add_chunk(RecordingChunk::new(2));
        track.add_chunk(RecordingChunk::new(3));

        assert_eq!(track.chunks().len(), 3);
        assert_eq!(track.chunks()[0].sequence, 1);
        assert_eq!(track.chunks()[1].sequence, 2);
        assert_eq!(track.chunks()[2].sequence, 3);
    }

    // TEST-30
    #[test]
    fn artifact_can_contain_multiple_independent_tracks() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        let mut host_track = RecordingTrack::new("track-host");
        host_track.add_chunk(RecordingChunk::new(1));
        host_track.add_chunk(RecordingChunk::new(2));

        let mut guest_track = RecordingTrack::new("track-guest");
        guest_track.add_chunk(RecordingChunk::new(1));
        guest_track.add_chunk(RecordingChunk::new(2));
        guest_track.add_chunk(RecordingChunk::new(3));

        artifact.add_track(host_track);
        artifact.add_track(guest_track);

        assert_eq!(artifact.tracks().len(), 2);
        assert_eq!(artifact.tracks()[0].chunks().len(), 2);
        assert_eq!(artifact.tracks()[1].chunks().len(), 3);
    }

    // TEST-31
    #[test]
    fn artifact_can_preserve_domain_association() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.set_domain_association("production-001", "recording-017");

        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
    }

    // TEST-35
    #[test]
    fn recording_chunk_can_contain_payload() {
        let chunk = RecordingChunk::with_payload(1, "track-host/chunk-000001", vec![1, 2, 3]);

        assert_eq!(chunk.sequence, 1);
        assert_eq!(
            chunk.payload().reference().value(),
            "track-host/chunk-000001"
        );
        assert_eq!(chunk.payload().data(), &[1, 2, 3]);
        assert_eq!(chunk.payload().size_bytes(), 3);
        assert_eq!(chunk.payload().hash(), &PayloadHash::from_bytes(&[1, 2, 3]));
    }

    // TEST-37
    #[test]
    fn recording_track_preserves_configuration() {
        let configuration = RecordingConfiguration::new(48_000, 1, crate::audio::SampleFormat::F32);
        let track = RecordingTrack::with_configuration("track-host", configuration);

        assert_eq!(track.configuration(), Some(configuration));
    }

    // TEST-38
    #[test]
    fn recording_chunk_preserves_sample_offset() {
        let chunk = RecordingChunk::with_sample_offset(
            2,
            14_400_000,
            "track-host/chunk-000002",
            vec![1, 2, 3],
        );

        assert_eq!(chunk.sequence, 2);
        assert_eq!(chunk.sample_offset(), 14_400_000);
    }
}
