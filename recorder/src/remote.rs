//! Provider-neutral information exposed to remote artifact providers.
//!
//! The remote boundary deliberately carries recording information rather than
//! provider-specific paths or filenames. A provider may use or ignore any
//! field when constructing its own remote representation.

use std::time::SystemTime;

use crate::artifact::{ManifestHash, PayloadHash, RecordingArtifact};
use crate::completion::PreparedUpload;

/// Optional provider-neutral recording information for a finished artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteArtifactMetadata {
    recording_started_at: SystemTime,
    display_name: Option<String>,
}

impl RemoteArtifactMetadata {
    pub fn new(recording_started_at: SystemTime, display_name: Option<String>) -> Self {
        Self {
            recording_started_at,
            display_name: normalize_display_name(display_name),
        }
    }

    pub fn recording_started_at(&self) -> SystemTime {
        self.recording_started_at
    }

    pub fn display_name(&self) -> Option<&str> {
        self.display_name.as_deref()
    }
}

#[derive(Debug, Clone)]
pub struct RemoteArtifact<'a> {
    artifact: &'a RecordingArtifact,
    metadata: RemoteArtifactMetadata,
}

impl<'a> RemoteArtifact<'a> {
    pub fn new(
        artifact: &'a RecordingArtifact,
        recording_started_at: SystemTime,
        display_name: Option<String>,
    ) -> Self {
        Self {
            artifact,
            metadata: RemoteArtifactMetadata::new(recording_started_at, display_name),
        }
    }

    pub fn artifact(&self) -> &'a RecordingArtifact {
        self.artifact
    }

    pub fn metadata(&self) -> &RemoteArtifactMetadata {
        &self.metadata
    }
}

/// Remote confirmation of the exact prepared artifact payloads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteUploadReceipt {
    artifact_id: String,
    manifest_hash: ManifestHash,
    tracks: Vec<RemoteUploadTrackReceipt>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteUploadTrackReceipt {
    track_id: String,
    size_bytes: u64,
    hash: PayloadHash,
}

impl RemoteUploadReceipt {
    pub fn new(
        artifact_id: impl Into<String>,
        manifest_hash: ManifestHash,
        tracks: Vec<RemoteUploadTrackReceipt>,
    ) -> Self {
        Self {
            artifact_id: artifact_id.into(),
            manifest_hash,
            tracks,
        }
    }

    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    pub fn manifest_hash(&self) -> ManifestHash {
        self.manifest_hash
    }

    pub fn tracks(&self) -> &[RemoteUploadTrackReceipt] {
        &self.tracks
    }
}

impl RemoteUploadTrackReceipt {
    pub fn new(track_id: impl Into<String>, size_bytes: u64, hash: PayloadHash) -> Self {
        Self {
            track_id: track_id.into(),
            size_bytes,
            hash,
        }
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn hash(&self) -> PayloadHash {
        self.hash
    }
}

/// Adapter boundary for transferring a prepared recording artifact.
pub trait RemoteArtifactUploader {
    type Error;

    fn upload(&mut self, upload: &PreparedUpload) -> Result<RemoteUploadReceipt, Self::Error>;
}

/// Validates that the remote receipt confirms exactly the prepared upload.
pub fn validate_upload_receipt(
    upload: &PreparedUpload,
    receipt: &RemoteUploadReceipt,
) -> Result<(), RemoteUploadValidationError> {
    if upload.artifact_id() != receipt.artifact_id() {
        return Err(RemoteUploadValidationError::ArtifactMismatch);
    }
    if upload.manifest_hash() != receipt.manifest_hash() {
        return Err(RemoteUploadValidationError::ManifestMismatch);
    }
    if upload.tracks().len() != receipt.tracks().len() {
        return Err(RemoteUploadValidationError::TrackMismatch);
    }

    for (prepared, confirmed) in upload.tracks().iter().zip(receipt.tracks()) {
        if prepared.track_id() != confirmed.track_id()
            || prepared.size_bytes() != confirmed.size_bytes()
            || prepared.hash() != confirmed.hash()
        {
            return Err(RemoteUploadValidationError::TrackMismatch);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoteUploadValidationError {
    ArtifactMismatch,
    ManifestMismatch,
    TrackMismatch,
}

fn normalize_display_name(display_name: Option<String>) -> Option<String> {
    display_name.and_then(|name| {
        let trimmed = name.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
    use crate::audio::{RecordingConfiguration, SampleFormat};
    use crate::completion::CompletionJob;
    use crate::session::RecordingSessionId;

    fn test_upload() -> PreparedUpload {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut track = RecordingTrack::with_configuration("track-a", configuration);
        track.add_chunk(RecordingChunk::with_payload(
            1,
            "track-a/chunk-000001",
            vec![0, 0, 1, 0, 2, 0, 3, 0],
        ));
        let mut artifact = RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));
        artifact.add_track(track);
        let mut job = CompletionJob::new("job-remote", "capture-remote");
        job.begin_upload_preparation().unwrap();
        job.prepare_upload(&artifact).unwrap()
    }

    #[test]
    fn receipt_must_confirm_exact_payloads() {
        let upload = test_upload();
        let tracks = upload
            .tracks()
            .iter()
            .map(|track| {
                RemoteUploadTrackReceipt::new(track.track_id(), track.size_bytes(), track.hash())
            })
            .collect();
        let receipt = RemoteUploadReceipt::new(upload.artifact_id(), upload.manifest_hash(), tracks);

        assert_eq!(validate_upload_receipt(&upload, &receipt), Ok(()));
    }

    #[test]
    fn receipt_rejects_wrong_payload_hash() {
        let upload = test_upload();
        let receipt = RemoteUploadReceipt::new(
            upload.artifact_id(),
            upload.manifest_hash(),
            vec![RemoteUploadTrackReceipt::new(
                upload.tracks()[0].track_id(),
                upload.tracks()[0].size_bytes(),
                PayloadHash::from_bytes(b"wrong"),
            )],
        );

        assert_eq!(
            validate_upload_receipt(&upload, &receipt),
            Err(RemoteUploadValidationError::TrackMismatch)
        );
    }

    #[test]
    fn empty_display_name_is_absent() {
        let metadata = RemoteArtifactMetadata::new(SystemTime::UNIX_EPOCH, Some("  ".into()));
        assert_eq!(metadata.display_name(), None);
    }

    #[test]
    fn display_name_is_preserved() {
        let metadata = RemoteArtifactMetadata::new(
            SystemTime::UNIX_EPOCH,
            Some(" Interview Frizz Feick ".into()),
        );
        assert_eq!(metadata.display_name(), Some("Interview Frizz Feick"));
    }

    #[test]
    fn remote_view_keeps_artifact_as_transfer_unit() {
        let session_id = RecordingSessionId::new("session-001");
        let artifact = RecordingArtifact::new("artifact-001", session_id);
        let remote = RemoteArtifact::new(&artifact, SystemTime::UNIX_EPOCH, None);

        assert_eq!(remote.artifact().id.value(), "artifact-001");
        assert_eq!(remote.metadata().display_name(), None);
        assert_eq!(remote.metadata().recording_started_at(), SystemTime::UNIX_EPOCH);
    }
}