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

pub mod coordination;
pub mod factory;
pub mod id;
pub mod processing;
pub mod recovery;
pub mod registry;

pub use id::{ArtifactId, RecordingTrackId};

use crate::audio::RecordingConfiguration;
use crate::session::RecordingSessionId;

/// Technical lifecycle state of a Recording Artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactStatus {
    Created,
    Available,
    Stored,
}

/// Opaque reference to the domain recording context that produced an artifact.
///
/// The recorder crate deliberately does not depend on the core crate. The
/// identifiers are therefore stored as opaque values at this boundary.
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
///
/// The chunk deliberately contains no filesystem-specific information.
/// Its position belongs to the track and is represented by its sequence
/// number. The physical payload location is decided by the persistence
/// provider.
///
/// See ADR-003, ADR-054 and ADR-058.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingChunk {
    pub sequence: u32,
    payload: RecordingPayload,
}

impl RecordingChunk {
    /// Creates a recording chunk without payload data.
    pub fn new(sequence: u32) -> Self {
        Self::with_payload(sequence, format!("chunk-{sequence:06}"), Vec::new())
    }

    /// Creates a recording chunk with its logical payload reference and data.
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

    /// Returns the technical payload belonging to this recording chunk.
    pub fn payload(&self) -> &RecordingPayload {
        &self.payload
    }
}

/// A technical recording track.
///
/// A track represents one technical audio stream within a
/// RecordingArtifact. It does not represent a domain participant or role.
///
/// See ADR-002 and ADR-054.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingTrack {
    pub id: RecordingTrackId,
    configuration: Option<RecordingConfiguration>,
    chunks: Vec<RecordingChunk>,
}

impl RecordingTrack {
    /// Creates an empty recording track without configuration metadata.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: RecordingTrackId::new(id),
            configuration: None,
            chunks: Vec::new(),
        }
    }

    /// Creates an empty recording track with the supplied recording configuration.
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

    /// Returns the recording configuration used for this track, if known.
    pub const fn configuration(&self) -> Option<RecordingConfiguration> {
        self.configuration
    }

    /// Adds a technical recording chunk.
    pub fn add_chunk(&mut self, chunk: RecordingChunk) {
        self.chunks.push(chunk);
    }

    /// Returns the chunks belonging to this recording track.
    pub fn chunks(&self) -> &[RecordingChunk] {
        &self.chunks
    }
}

/// Technical representation of the result of a local recording.
///
/// A RecordingArtifact is not itself a file. It represents the technical
/// recording result and associates its tracks and recording data.
///
/// See ADR-042, ADR-054 and ADR-058.
#[derive(Debug, Clone)]
pub struct RecordingArtifact {
    pub id: ArtifactId,
    pub recording_session_id: RecordingSessionId,
    status: ArtifactStatus,
    tracks: Vec<RecordingTrack>,
    association: Option<RecordingArtifactAssociation>,
}

impl RecordingArtifact {
    /// Creates a new recording artifact.
    ///
    /// A new artifact represents a technical recording result
    /// that has been created but is not yet available or stored.
    pub fn new(id: impl Into<String>, recording_session_id: RecordingSessionId) -> Self {
        Self {
            id: ArtifactId::new(id),
            recording_session_id,
            status: ArtifactStatus::Created,
            tracks: Vec::new(),
            association: None,
        }
    }

    /// Returns the current artifact status.
    pub fn status(&self) -> &ArtifactStatus {
        &self.status
    }

    /// Marks the artifact as available.
    pub fn make_available(&mut self) {
        self.status = ArtifactStatus::Available;
    }

    /// Marks the artifact as stored.
    pub fn store(&mut self) {
        self.status = ArtifactStatus::Stored;
    }

    /// Adds a technical recording track to the artifact.
    pub fn add_track(&mut self, track: RecordingTrack) {
        self.tracks.push(track);
    }

    /// Returns the tracks belonging to this artifact.
    pub fn tracks(&self) -> &[RecordingTrack] {
        &self.tracks
    }

    /// Associates the artifact with its originating domain recording context.
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

    /// Returns the originating domain association, if one was supplied.
    pub fn association(&self) -> Option<&RecordingArtifactAssociation> {
        self.association.as_ref()
    }

    /// Returns the originating production identifier, if one was supplied.
    pub fn production_id(&self) -> Option<&str> {
        self.association
            .as_ref()
            .map(RecordingArtifactAssociation::production_id)
    }

    /// Returns the originating domain recording identifier, if one was supplied.
    pub fn recording_id(&self) -> Option<&str> {
        self.association
            .as_ref()
            .map(RecordingArtifactAssociation::recording_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-09
    //
    // Verify: A new artifact starts in Created state.
    //
    // This protects ADR-042:
    // Recording artifacts have their own technical lifecycle
    // independent from domain recording states.
    #[test]
    fn new_artifact_starts_as_created() {
        let artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        assert_eq!(artifact.status(), &ArtifactStatus::Created);
    }

    // TEST-10
    //
    // Verify: Artifact lifecycle can progress from Created to Available.
    #[test]
    fn artifact_can_become_available() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();

        assert_eq!(artifact.status(), &ArtifactStatus::Available);
    }

    // TEST-11
    //
    // Verify: Available artifacts can be stored.
    #[test]
    fn artifact_can_be_stored() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();
        artifact.store();

        assert_eq!(artifact.status(), &ArtifactStatus::Stored);
    }

    // TEST-28
    //
    // Protects ADR-054:
    // A RecordingArtifact can contain technical recording tracks.
    #[test]
    fn artifact_can_contain_tracks() {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.add_track(RecordingTrack::new("track-001"));

        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].id.value(), "track-001");
    }

    // TEST-29
    //
    // Protects ADR-054:
    // A recording track can contain multiple ordered chunks.
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
    //
    // Protects ADR-054:
    // Tracks are independent technical structures.
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
    //
    // Protects ADR-058:
    // A RecordingChunk can carry actual technical payload bytes
    // while exposing only a storage-provider-independent reference.
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
    }

    // TEST-37
    //
    // Protects the capture-to-artifact boundary:
    // the recording configuration used for a technical track is preserved.
    #[test]
    fn recording_track_preserves_configuration() {
        let configuration = RecordingConfiguration::new(48_000, 1, crate::audio::SampleFormat::F32);
        let track = RecordingTrack::with_configuration("track-host", configuration);

        assert_eq!(track.configuration(), Some(configuration));
    }
}
