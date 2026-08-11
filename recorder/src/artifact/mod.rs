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

pub mod coordination;
pub mod id;
pub mod factory;
pub mod processing;
pub mod recovery;
pub mod registry;

pub use id::ArtifactId;

use crate::session::RecordingSessionId;

/// Technical lifecycle state of a Recording Artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactStatus {
    Created,
    Available,
    Stored,
}

/// A technical chunk of recording data.
///
/// The chunk deliberately contains no filesystem-specific information.
/// Its position belongs to the track and is represented by its sequence
/// number.
///
/// See ADR-003 and ADR-054.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingChunk {
    pub sequence: u32,
}

impl RecordingChunk {
    /// Creates a recording chunk at the given sequence position.
    pub fn new(sequence: u32) -> Self {
        Self { sequence }
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
    pub id: ArtifactId,
    chunks: Vec<RecordingChunk>,
}

impl RecordingTrack {
    /// Creates an empty recording track.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: ArtifactId::new(id),
            chunks: Vec::new(),
        }
    }

    /// Adds a chunk to this track.
    pub fn add_chunk(&mut self, chunk: RecordingChunk) {
        self.chunks.push(chunk);
    }

    /// Returns the chunks belonging to this track.
    pub fn chunks(&self) -> &[RecordingChunk] {
        &self.chunks
    }
}

/// Technical representation of the result of a local recording.
///
/// A RecordingArtifact is not itself a file. It represents the technical
/// recording result and associates its tracks and recording data.
///
/// See ADR-042 and ADR-054.
#[derive(Debug, Clone)]
pub struct RecordingArtifact {
    pub id: ArtifactId,
    pub recording_session_id: RecordingSessionId,
    status: ArtifactStatus,
    tracks: Vec<RecordingTrack>,
}

impl RecordingArtifact {
    /// Creates a new recording artifact.
    ///
    /// A new artifact represents a technical recording result
    /// that has been created but is not yet available or stored.
    pub fn new(
        id: impl Into<String>,
        recording_session_id: RecordingSessionId,
    ) -> Self {
        Self {
            id: ArtifactId::new(id),
            recording_session_id,
            status: ArtifactStatus::Created,
            tracks: Vec::new(),
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
        let artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        assert_eq!(artifact.status(), &ArtifactStatus::Created);
    }

    // TEST-10
    //
    // Verify: Artifact lifecycle can progress from Created to Available.
    #[test]
    fn artifact_can_become_available() {
        let mut artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.make_available();

        assert_eq!(artifact.status(), &ArtifactStatus::Available);
    }

    // TEST-11
    //
    // Verify: Available artifacts can be stored.
    #[test]
    fn artifact_can_be_stored() {
        let mut artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

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
        let mut artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

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
        let mut artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

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
}
