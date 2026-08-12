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

pub mod coordination;
pub mod factory;
pub mod id;
pub mod processing;
pub mod recovery;
pub mod registry;

pub use id::{ArtifactId, RecordingTrackId};

use crate::session::RecordingSessionId;

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
///
/// The reference identifies the payload within the artifact without encoding
/// an absolute filesystem path or another concrete storage location.
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
///
/// The payload bytes remain technical recording data. The logical reference
/// is intentionally independent of the physical persistence provider.
///
/// Integrity validation is deliberately deferred to the recovery work in #8.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingPayload {
    reference: RecordingPayloadReference,
    data: Vec<u8>,
}

impl RecordingPayload {
    pub fn new(reference: impl Into<String>, data: impl Into<Vec<u8>>) -> Self {
        Self {
            reference: RecordingPayloadReference::new(reference),
            data: data.into(),
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
}

/// A technical chunk of recording data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingChunk {
    pub sequence: u32,
    payload: RecordingPayload,
}

impl RecordingChunk {
    pub fn new(sequence: u32) -> Self {
        Self::with_payload(sequence, format!("chunk-{sequence:06}"), Vec::new())
    }

    pub fn with_payload(
        sequence: u32,
        reference: impl Into<String>,
        data: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            sequence,
            payload: RecordingPayload::new(reference, data),
        }
    }

    pub fn payload(&self) -> &RecordingPayload {
        &self.payload
    }
}

/// A technical recording track.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingTrack {
    pub id: RecordingTrackId,
    chunks: Vec<RecordingChunk>,
}

impl RecordingTrack {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingTrackId::new(id),
            chunks: Vec::new(),
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_artifact_starts_as_created() {
        let artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        assert_eq!(artifact.status(), &ArtifactStatus::Created);
    }

    #[test]
    fn artifact_can_become_available() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();

        assert_eq!(artifact.status(), &ArtifactStatus::Available);
    }

    #[test]
    fn artifact_can_be_stored() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();
        artifact.store();

        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
    }

    #[test]
    fn artifact_can_contain_tracks() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.add_track(RecordingTrack::new("track-001"));

        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].id.value(), "track-001");
    }

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

    #[test]
    fn artifact_can_preserve_domain_association() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.set_domain_association("production-001", "recording-017");

        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
    }

    #[test]
    fn recording_chunk_can_contain_payload() {
        let chunk = RecordingChunk::with_payload(1, "track-host/chunk-000001", vec![1, 2, 3]);

        assert_eq!(chunk.sequence, 1);
        assert_eq!(chunk.payload().reference().value(), "track-host/chunk-000001");
        assert_eq!(chunk.payload().data(), &[1, 2, 3]);
        assert_eq!(chunk.payload().size_bytes(), 3);
    }
}
