//! Integrity primitives for recording artifacts.
//!
//! Integrity is part of the artifact model and deliberately independent of
//! any concrete persistence provider. The hashing primitive is kept small so
//! the capture path can later feed it incrementally without coupling capture
//! to storage latency.

use sha2::{Digest, Sha256};

/// SHA-256 digest of technical recording data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadHash([u8; 32]);

impl PayloadHash {
    /// Calculates the SHA-256 digest of a complete byte slice.
    ///
    /// This convenience constructor is suitable for already-buffered data.
    /// Streaming capture should use an incremental `Sha256` hasher and create
    /// the resulting `PayloadHash` once the chunk is complete.
    pub fn from_bytes(data: &[u8]) -> Self {
        let digest = Sha256::digest(data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest);
        Self(bytes)
    }

    /// Returns the raw 32-byte SHA-256 digest.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// SHA-256 digest of the deterministic technical recording-artifact manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ManifestHash([u8; 32]);

impl ManifestHash {
    pub(crate) fn from_manifest_bytes(data: &[u8]) -> Self {
        let digest = Sha256::digest(data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest);
        Self(bytes)
    }

    /// Returns the raw 32-byte SHA-256 digest.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
    use crate::audio::{RecordingChunkDuration, RecordingConfiguration, SampleFormat};
    use crate::session::RecordingSessionId;

    #[test]
    fn sha256_hash_is_deterministic() {
        let first = PayloadHash::from_bytes(b"nc-pore");
        let second = PayloadHash::from_bytes(b"nc-pore");

        assert_eq!(first, second);
    }

    #[test]
    fn different_payloads_have_different_hashes() {
        let first = PayloadHash::from_bytes(b"nc-pore");
        let second = PayloadHash::from_bytes(b"nc-pore!");

        assert_ne!(first, second);
    }

    #[test]
    fn manifest_hash_is_deterministic() {
        let first = ManifestHash::from_manifest_bytes(b"nc-pore-manifest");
        let second = ManifestHash::from_manifest_bytes(b"nc-pore-manifest");

        assert_eq!(first, second);
    }

    #[test]
    fn manifest_hash_changes_when_manifest_changes() {
        let first = ManifestHash::from_manifest_bytes(b"nc-pore-manifest");
        let second = ManifestHash::from_manifest_bytes(b"nc-pore-manifest!");

        assert_ne!(first, second);
    }

    #[test]
    fn manifest_hash_changes_for_relevant_configuration_changes() {
        let first = artifact_with_configuration(RecordingChunkDuration::OneMinute);
        let second = artifact_with_configuration(RecordingChunkDuration::FiveMinutes);

        assert_ne!(first.manifest_hash(), second.manifest_hash());
    }

    #[test]
    fn manifest_hash_ignores_lifecycle_status() {
        let first = artifact_with_configuration(RecordingChunkDuration::OneMinute);
        let mut second = first.clone();

        second.make_available();
        second.store();

        assert_eq!(first.manifest_hash(), second.manifest_hash());
    }

    #[test]
    fn manifest_hash_binds_payload_reference_and_hash() {
        let first = artifact_with_payload("track-a/chunk-000001", b"payload-a");
        let second = artifact_with_payload("track-a/chunk-000002", b"payload-a");
        let third = artifact_with_payload("track-a/chunk-000001", b"payload-b");

        assert_ne!(first.manifest_hash(), second.manifest_hash());
        assert_ne!(first.manifest_hash(), third.manifest_hash());
    }

    fn artifact_with_configuration(duration: RecordingChunkDuration) -> RecordingArtifact {
        let configuration =
            RecordingConfiguration::with_chunk_duration(48_000, 1, SampleFormat::Pcm24, duration);
        let mut track = RecordingTrack::with_configuration("track-a", configuration);
        track.add_chunk(RecordingChunk::with_payload(
            1,
            "track-a/chunk-000001",
            b"payload".to_vec(),
        ));

        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));
        artifact.add_track(track);
        artifact
    }

    fn artifact_with_payload(reference: &str, data: &[u8]) -> RecordingArtifact {
        let mut track = RecordingTrack::new("track-a");
        track.add_chunk(RecordingChunk::with_payload(1, reference, data.to_vec()));

        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));
        artifact.add_track(track);
        artifact
    }
}
