//! Provider-neutral information exposed to remote artifact providers.
//!
//! The remote boundary deliberately carries recording information rather than
//! provider-specific paths or filenames. A provider may use or ignore any
//! field when constructing its own remote representation.

use std::time::SystemTime;

use crate::artifact::factory::RecordingArtifactFactory;
use crate::artifact::{ManifestHash, PayloadHash, RecordingArtifact};
use crate::completion::{
    CompletionJob, CompletionJobError, FilesystemCompletionJobStore, PreparedUpload,
};
use crate::preservation::{FilesystemPreservationStore, PreservationLoadResult};
use crate::session::RecordingSessionId;

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

/// Orchestrates the durable upload workflow up to an explicitly confirmed
/// remote upload. `Uploaded` is the terminal state of the upload; higher-level
/// completion and local cleanup may follow only after this point.
pub struct UploadCoordinator<'a, U> {
    jobs: &'a FilesystemCompletionJobStore,
    uploader: U,
}

impl<'a, U> UploadCoordinator<'a, U>
where
    U: RemoteArtifactUploader,
{
    pub fn new(jobs: &'a FilesystemCompletionJobStore, uploader: U) -> Self {
        Self { jobs, uploader }
    }

    /// Runs one upload attempt and finalizes it only after exact receipt validation.
    pub fn upload(
        &mut self,
        job: &mut CompletionJob,
        artifact: &RecordingArtifact,
    ) -> Result<(), UploadCoordinatorError<U::Error>> {
        job.begin_upload_preparation()
            .map_err(UploadCoordinatorError::Job)?;
        self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
        let prepared = match job.prepare_upload(artifact) {
            Ok(upload) => upload,
            Err(error) => {
                let _ = job.mark_retryable_failure();
                let _ = self.jobs.save(job);
                return Err(UploadCoordinatorError::Job(error));
            }
        };
        job.mark_ready_for_upload()
            .map_err(UploadCoordinatorError::Job)?;
        self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
        job.mark_uploading().map_err(UploadCoordinatorError::Job)?;
        self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
        let receipt = match self.uploader.upload(&prepared) {
            Ok(receipt) => receipt,
            Err(error) => {
                job.mark_retryable_failure()
                    .map_err(UploadCoordinatorError::Job)?;
                self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
                return Err(UploadCoordinatorError::Upload(error));
            }
        };
        if let Err(error) = validate_upload_receipt(&prepared, &receipt) {
            job.mark_retryable_failure()
                .map_err(UploadCoordinatorError::Job)?;
            self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
            return Err(UploadCoordinatorError::Confirmation(error));
        }
        // This is the upload boundary's finalization step. Only after exact
        // remote confirmation may the durable job become `Uploaded`.
        job.mark_uploaded().map_err(UploadCoordinatorError::Job)?;
        self.jobs.save(job).map_err(UploadCoordinatorError::Job)?;
        Ok(())
    }

    /// Restores the durable capture named by the upload job, rebuilds the
    /// artifact from that preserved representation, and then uploads it.
    pub fn upload_from_preservation(
        &mut self,
        job: &mut CompletionJob,
        preservation: &FilesystemPreservationStore,
        recording_session_id: RecordingSessionId,
    ) -> Result<(), UploadCoordinatorError<U::Error>> {
        let preserved = match preservation.load(job.capture_id()) {
            PreservationLoadResult::Valid(capture) => capture,
            PreservationLoadResult::Incomplete => {
                return Err(UploadCoordinatorError::Preservation(
                    PreservationCompletionError::Incomplete,
                ));
            }
            PreservationLoadResult::Inconsistent => {
                return Err(UploadCoordinatorError::Preservation(
                    PreservationCompletionError::Inconsistent,
                ));
            }
            PreservationLoadResult::NotFound => {
                return Err(UploadCoordinatorError::Preservation(
                    PreservationCompletionError::NotFound,
                ));
            }
        };
        let artifact = RecordingArtifactFactory::create(preserved, recording_session_id);
        self.upload(job, &artifact)
    }

    pub fn uploader(&self) -> &U {
        &self.uploader
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum UploadCoordinatorError<E> {
    Job(CompletionJobError),
    Preservation(PreservationCompletionError),
    Upload(E),
    Confirmation(RemoteUploadValidationError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreservationCompletionError {
    Incomplete,
    Inconsistent,
    NotFound,
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
    use crate::audio::{CaptureChunk, CaptureResult, RecordingConfiguration, SampleFormat};
    use crate::completion::CompletionJob;
    use crate::preservation::CapturePreserver;
    use crate::session::RecordingSessionId;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_upload() -> PreparedUpload {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut track = RecordingTrack::with_configuration("track-a", configuration);
        track.add_chunk(RecordingChunk::with_payload(
            1,
            "track-a/chunk-000001",
            vec![0, 0, 1, 0, 2, 0, 3, 0],
        ));
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));
        artifact.add_track(track);
        let mut job = CompletionJob::new("job-remote", "capture-remote");
        job.begin_upload_preparation().unwrap();
        job.prepare_upload(&artifact).unwrap()
    }

    fn test_artifact() -> RecordingArtifact {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut track = RecordingTrack::with_configuration("track-a", configuration);
        track.add_chunk(RecordingChunk::with_payload(
            1,
            "track-a/chunk-000001",
            vec![0, 0, 1, 0, 2, 0, 3, 0],
        ));
        let mut artifact = RecordingArtifact::new(
            "artifact-coordinator",
            RecordingSessionId::new("session-001"),
        );
        artifact.add_track(track);
        artifact
    }

    struct ConfirmingUploader;
    impl RemoteArtifactUploader for ConfirmingUploader {
        type Error = ();

        fn upload(&mut self, upload: &PreparedUpload) -> Result<RemoteUploadReceipt, Self::Error> {
            let tracks = upload
                .tracks()
                .iter()
                .map(|track| {
                    RemoteUploadTrackReceipt::new(
                        track.track_id(),
                        track.size_bytes(),
                        track.hash(),
                    )
                })
                .collect();
            Ok(RemoteUploadReceipt::new(
                upload.artifact_id(),
                upload.manifest_hash(),
                tracks,
            ))
        }
    }

    struct MismatchingUploader;
    impl RemoteArtifactUploader for MismatchingUploader {
        type Error = ();

        fn upload(&mut self, upload: &PreparedUpload) -> Result<RemoteUploadReceipt, Self::Error> {
            Ok(RemoteUploadReceipt::new(
                upload.artifact_id(),
                upload.manifest_hash(),
                vec![RemoteUploadTrackReceipt::new(
                    "track-a",
                    0,
                    PayloadHash::from_bytes(b"wrong"),
                )],
            ))
        }
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
        let receipt =
            RemoteUploadReceipt::new(upload.artifact_id(), upload.manifest_hash(), tracks);
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

    // TEST-54: Completion reaches Uploaded only after exact remote confirmation.
    #[test]
    fn upload_coordinator_requires_exact_confirmation_before_uploaded() {
        let root = std::env::temp_dir().join(format!("nc-pore-coordinator-{}", std::process::id()));
        let store = FilesystemCompletionJobStore::new(&root).unwrap();
        let artifact = test_artifact();
        let mut job = CompletionJob::new("job-054", "capture-054");
        let mut coordinator = UploadCoordinator::new(&store, ConfirmingUploader);
        coordinator.upload(&mut job, &artifact).unwrap();
        assert_eq!(job.state(), crate::completion::CompletionJobState::Uploaded);
        let _ = std::fs::remove_dir_all(root);
    }

    // TEST-55: A mismatching receipt cannot advance the durable job to Uploaded.
    #[test]
    fn upload_coordinator_rejects_mismatching_confirmation() {
        let root = std::env::temp_dir().join(format!(
            "nc-pore-coordinator-mismatch-{}",
            std::process::id()
        ));
        let store = FilesystemCompletionJobStore::new(&root).unwrap();
        let artifact = test_artifact();
        let mut job = CompletionJob::new("job-055", "capture-055");
        let mut coordinator = UploadCoordinator::new(&store, MismatchingUploader);
        let error = coordinator.upload(&mut job, &artifact).unwrap_err();
        assert_eq!(
            error,
            UploadCoordinatorError::Confirmation(RemoteUploadValidationError::TrackMismatch)
        );
        assert_eq!(
            job.state(),
            crate::completion::CompletionJobState::FailedRetryable
        );
        let _ = std::fs::remove_dir_all(root);
    }

    // TEST-56: Restart recovery reconstructs the artifact from durable preservation.
    #[test]
    fn upload_coordinator_can_upload_from_durable_preservation() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let preservation_root =
            std::env::temp_dir().join(format!("nc-pore-preservation-{}", suffix));
        let jobs_root = std::env::temp_dir().join(format!("nc-pore-jobs-{}", suffix));
        let preservation = FilesystemPreservationStore::new(&preservation_root);
        let jobs = FilesystemCompletionJobStore::new(&jobs_root).unwrap();
        let mut capture = CaptureResult::new("capture-056");
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut track = crate::audio::CaptureTrack::with_configuration("track-a", configuration);
        track.add_chunk(CaptureChunk::with_payload(1, vec![0, 0, 1, 0, 2, 0, 3, 0]));
        capture.add_track(track);
        let preserved = CapturePreserver::preserve(capture);
        preservation.store(&preserved).unwrap();
        let mut job = CompletionJob::new("job-056", "capture-056");
        let mut coordinator = UploadCoordinator::new(&jobs, ConfirmingUploader);
        coordinator
            .upload_from_preservation(
                &mut job,
                &preservation,
                RecordingSessionId::new("session-056"),
            )
            .unwrap();
        assert_eq!(job.state(), crate::completion::CompletionJobState::Uploaded);
        assert_eq!(job.artifact_id(), Some("capture-056"));
        let _ = std::fs::remove_dir_all(preservation_root);
        let _ = std::fs::remove_dir_all(jobs_root);
    }

    #[test]
    fn missing_durable_capture_is_not_treated_as_upload_failure() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let preservation_root =
            std::env::temp_dir().join(format!("nc-pore-preservation-missing-{}", suffix));
        let jobs_root = std::env::temp_dir().join(format!("nc-pore-jobs-missing-{}", suffix));
        let preservation = FilesystemPreservationStore::new(&preservation_root);
        let jobs = FilesystemCompletionJobStore::new(&jobs_root).unwrap();
        let mut job = CompletionJob::new("job-057", "capture-057");
        let mut coordinator = UploadCoordinator::new(&jobs, ConfirmingUploader);
        let error = coordinator
            .upload_from_preservation(
                &mut job,
                &preservation,
                RecordingSessionId::new("session-057"),
            )
            .unwrap_err();
        assert_eq!(
            error,
            UploadCoordinatorError::Preservation(PreservationCompletionError::NotFound)
        );
        assert_eq!(job.state(), crate::completion::CompletionJobState::Pending);
        let _ = std::fs::remove_dir_all(preservation_root);
        let _ = std::fs::remove_dir_all(jobs_root);
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
        assert_eq!(
            remote.metadata().recording_started_at(),
            SystemTime::UNIX_EPOCH
        );
    }
}
