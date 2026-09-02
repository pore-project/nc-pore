//! Durable completion-job state for local recovery.
//!
//! A completion job is the durable hand-off between local preservation and
//! later artifact/transport/upload work. The job contains only workflow state
//! and stable identifiers; audio payloads remain in the preservation store.
//! This makes an interrupted completion resumable after a client restart.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::artifact::{ManifestHash, RecordingArtifact, RecordingTrack};
use crate::audio::CaptureChunk;
use crate::transport::{FlacEncodeError, encode_flac};

/// Stable lifecycle of one local completion job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionJobState {
    Pending,
    PreparingUpload,
    ReadyForUpload,
    Uploading,
    Uploaded,
    FailedRetryable,
}

impl CompletionJobState {
    /// Returns whether the job can be retried without creating a new job.
    pub fn is_resumable(self) -> bool {
        matches!(
            self,
            Self::Pending
                | Self::PreparingUpload
                | Self::ReadyForUpload
                | Self::Uploading
                | Self::FailedRetryable
        )
    }
}

/// Durable description of completion work for one preserved capture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionJob {
    id: String,
    capture_id: String,
    artifact_id: Option<String>,
    state: CompletionJobState,
    attempts: u32,
}

impl CompletionJob {
    /// Creates a new completion job for an already preserved capture.
    pub fn new(id: impl Into<String>, capture_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            capture_id: capture_id.into(),
            artifact_id: None,
            state: CompletionJobState::Pending,
            attempts: 0,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn capture_id(&self) -> &str {
        &self.capture_id
    }

    pub fn artifact_id(&self) -> Option<&str> {
        self.artifact_id.as_deref()
    }

    pub fn state(&self) -> CompletionJobState {
        self.state
    }

    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Associates the durable artifact identity once artifact creation succeeds.
    pub fn set_artifact_id(&mut self, artifact_id: impl Into<String>) {
        self.artifact_id = Some(artifact_id.into());
    }

    /// Starts or resumes upload preparation and counts one processing attempt.
    pub fn begin_upload_preparation(&mut self) -> Result<(), CompletionJobError> {
        if !self.state.is_resumable() {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.attempts = self.attempts.saturating_add(1);
        self.state = CompletionJobState::PreparingUpload;
        Ok(())
    }

    /// Encodes the artifact into the V1 lossless transport representation.
    ///
    /// Preparation is deterministic: it is derived from the local artifact,
    /// while the durable job keeps only stable identity and workflow state.
    /// This means a client restart can recreate the exact upload bytes from
    /// the preserved local recording instead of treating a temporary network
    /// representation as the source of truth.
    pub fn prepare_upload(
        &mut self,
        artifact: &RecordingArtifact,
    ) -> Result<PreparedUpload, CompletionJobError> {
        if self.state != CompletionJobState::PreparingUpload {
            return Err(CompletionJobError::InvalidTransition);
        }
        if let Some(expected) = self.artifact_id() {
            if expected != artifact.id.value() {
                return Err(CompletionJobError::ArtifactMismatch);
            }
        } else {
            self.set_artifact_id(artifact.id.value());
        }

        PreparedUpload::from_artifact(artifact).map_err(CompletionJobError::Transport)
    }

    pub fn mark_ready_for_upload(&mut self) -> Result<(), CompletionJobError> {
        if self.state != CompletionJobState::PreparingUpload {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.state = CompletionJobState::ReadyForUpload;
        Ok(())
    }

    pub fn mark_uploading(&mut self) -> Result<(), CompletionJobError> {
        if self.state != CompletionJobState::ReadyForUpload {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.state = CompletionJobState::Uploading;
        Ok(())
    }

    pub fn mark_uploaded(&mut self) -> Result<(), CompletionJobError> {
        if self.state != CompletionJobState::Uploading {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.state = CompletionJobState::Uploaded;
        Ok(())
    }

    pub fn mark_retryable_failure(&mut self) -> Result<(), CompletionJobError> {
        if !self.state.is_resumable() || self.state == CompletionJobState::Pending {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.state = CompletionJobState::FailedRetryable;
        Ok(())
    }
}

/// One transport payload prepared for upload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUploadTrack {
    track_id: String,
    data: Vec<u8>,
}

impl PreparedUploadTrack {
    fn new(track_id: impl Into<String>, data: Vec<u8>) -> Self {
        Self {
            track_id: track_id.into(),
            data,
        }
    }

    pub fn track_id(&self) -> &str {
        &self.track_id
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn size_bytes(&self) -> u64 {
        self.data.len() as u64
    }

    pub fn hash(&self) -> crate::artifact::PayloadHash {
        crate::artifact::PayloadHash::from_bytes(&self.data)
    }
}

/// Deterministic upload preparation result for one recording artifact.
///
/// The manifest hash binds the upload to the technical artifact identity and
/// its original chunk hashes. Each FLAC payload has its own SHA-256 digest so
/// an eventual remote uploader can verify the exact bytes it transfers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedUpload {
    artifact_id: String,
    manifest_hash: ManifestHash,
    tracks: Vec<PreparedUploadTrack>,
}

impl PreparedUpload {
    fn from_artifact(artifact: &RecordingArtifact) -> Result<Self, FlacEncodeError> {
        let mut tracks = Vec::with_capacity(artifact.tracks().len());
        for track in artifact.tracks() {
            tracks.push(prepare_track(track)?);
        }

        Ok(Self {
            artifact_id: artifact.id.value().to_owned(),
            manifest_hash: artifact.manifest_hash(),
            tracks,
        })
    }

    pub fn artifact_id(&self) -> &str {
        &self.artifact_id
    }

    pub fn manifest_hash(&self) -> ManifestHash {
        self.manifest_hash
    }

    pub fn tracks(&self) -> &[PreparedUploadTrack] {
        &self.tracks
    }
}

fn prepare_track(track: &RecordingTrack) -> Result<PreparedUploadTrack, FlacEncodeError> {
    let configuration = track.configuration().ok_or(FlacEncodeError::Configuration(
        "track has no recording configuration".into(),
    ))?;
    let chunks: Vec<CaptureChunk> = track
        .chunks()
        .iter()
        .map(|chunk| CaptureChunk::with_payload(chunk.sequence, chunk.payload().data().to_vec()))
        .collect();
    let data = encode_flac(&chunks, configuration)?;
    Ok(PreparedUploadTrack::new(track.id.value(), data))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionJobError {
    InvalidTransition,
    InvalidId,
    Io,
    Serialization,
    ArtifactMismatch,
    Transport(FlacEncodeError),
}

/// Durable filesystem store for completion-job state.
///
/// Jobs are written through a temporary file followed by rename, so a restart
/// cannot observe a partially written JSON document as a valid job.
pub struct FilesystemCompletionJobStore {
    root: PathBuf,
}

impl FilesystemCompletionJobStore {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CompletionJobError> {
        let root = path.into();
        fs::create_dir_all(&root).map_err(|_| CompletionJobError::Io)?;
        Ok(Self { root })
    }

    fn valid_id(id: &str) -> bool {
        !id.is_empty() && id != "." && id != ".." && !id.contains('/') && !id.contains('\\')
    }

    fn path_for(&self, id: &str) -> PathBuf {
        self.root.join(format!("{id}.json"))
    }

    pub fn save(&self, job: &CompletionJob) -> Result<(), CompletionJobError> {
        if !Self::valid_id(job.id()) || !Self::valid_id(job.capture_id()) {
            return Err(CompletionJobError::InvalidId);
        }
        if let Some(artifact_id) = job.artifact_id() {
            if !Self::valid_id(artifact_id) {
                return Err(CompletionJobError::InvalidId);
            }
        }

        let bytes =
            serde_json::to_vec_pretty(job).map_err(|_| CompletionJobError::Serialization)?;
        let path = self.path_for(job.id());
        let temp = temporary_path(&path);
        fs::write(&temp, bytes).map_err(|_| CompletionJobError::Io)?;
        if let Err(error) = fs::rename(&temp, &path) {
            let _ = fs::remove_file(&temp);
            return Err(if error.kind() == std::io::ErrorKind::AlreadyExists {
                CompletionJobError::Io
            } else {
                CompletionJobError::Io
            });
        }
        Ok(())
    }

    pub fn load(&self, id: &str) -> Result<Option<CompletionJob>, CompletionJobError> {
        if !Self::valid_id(id) {
            return Err(CompletionJobError::InvalidId);
        }
        let path = self.path_for(id);
        if !path.is_file() {
            return Ok(None);
        }
        let bytes = fs::read(path).map_err(|_| CompletionJobError::Io)?;
        serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|_| CompletionJobError::Serialization)
    }

    /// Returns all jobs which still require local completion work after restart.
    pub fn resumable_jobs(&self) -> Result<Vec<CompletionJob>, CompletionJobError> {
        let entries = fs::read_dir(&self.root).map_err(|_| CompletionJobError::Io)?;
        let mut jobs = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|_| CompletionJobError::Io)?;
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                continue;
            }
            let bytes = fs::read(path).map_err(|_| CompletionJobError::Io)?;
            let job: CompletionJob =
                serde_json::from_slice(&bytes).map_err(|_| CompletionJobError::Serialization)?;
            if job.state().is_resumable() {
                jobs.push(job);
            }
        }
        jobs.sort_by(|left, right| left.id().cmp(right.id()));
        Ok(jobs)
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let mut temp = path.to_path_buf();
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("job.json");
    temp.set_file_name(format!(".{file_name}.tmp"));
    temp
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
    use crate::audio::{RecordingConfiguration, SampleFormat};
    use crate::session::RecordingSessionId;

    fn temp_store(name: &str) -> FilesystemCompletionJobStore {
        let store_name = format!("nc-pore-completion-{name}-{}", std::process::id());
        let root = std::env::temp_dir().join(store_name);
        let _ = fs::remove_dir_all(&root);
        FilesystemCompletionJobStore::new(root).unwrap()
    }

    fn test_artifact(id: &str) -> RecordingArtifact {
        let configuration = RecordingConfiguration::new(48_000, 1, SampleFormat::Pcm16);
        let mut track = RecordingTrack::with_configuration("track-a", configuration);
        let mut payload = Vec::new();
        for sample in [0i16, 1000, -1000, 32767, -32768] {
            payload.extend_from_slice(&sample.to_ne_bytes());
        }
        track.add_chunk(RecordingChunk::with_payload(
            1,
            "track-a/chunk-000001",
            payload,
        ));
        let mut artifact = RecordingArtifact::new(id, RecordingSessionId::new("session-001"));
        artifact.add_track(track);
        artifact
    }

    // TEST-48: Completion state survives a process restart boundary.
    #[test]
    fn completion_job_round_trip_is_durable() {
        let store = temp_store("round-trip");
        let mut job = CompletionJob::new("job-048", "capture-048");
        job.begin_upload_preparation().unwrap();
        job.set_artifact_id("artifact-048");
        job.mark_ready_for_upload().unwrap();
        store.save(&job).unwrap();

        let restored = store.load("job-048").unwrap().unwrap();
        assert_eq!(restored, job);

        let _ = fs::remove_dir_all(store.root);
    }

    // TEST-49: Interrupted upload preparation is discoverable after restart.
    #[test]
    fn interrupted_jobs_are_resumable() {
        let store = temp_store("resume");
        let mut preparing = CompletionJob::new("job-049-a", "capture-049-a");
        preparing.begin_upload_preparation().unwrap();
        store.save(&preparing).unwrap();

        let mut uploading = CompletionJob::new("job-049-b", "capture-049-b");
        uploading.begin_upload_preparation().unwrap();
        uploading.mark_ready_for_upload().unwrap();
        uploading.mark_uploading().unwrap();
        store.save(&uploading).unwrap();

        let mut completed = CompletionJob::new("job-049-c", "capture-049-c");
        completed.begin_upload_preparation().unwrap();
        completed.mark_ready_for_upload().unwrap();
        completed.mark_uploading().unwrap();
        completed.mark_uploaded().unwrap();
        store.save(&completed).unwrap();

        let jobs = store.resumable_jobs().unwrap();
        assert_eq!(jobs.len(), 2);
        assert_eq!(jobs[0].id(), "job-049-a");
        assert_eq!(jobs[1].id(), "job-049-b");

        let _ = fs::remove_dir_all(store.root);
    }

    // TEST-50: An interrupted upload can be resumed without changing identity.
    #[test]
    fn interrupted_upload_can_resume() {
        let mut job = CompletionJob::new("job-050", "capture-050");
        job.begin_upload_preparation().unwrap();
        job.mark_ready_for_upload().unwrap();
        job.mark_uploading().unwrap();
        let original_id = job.id().to_owned();
        let original_capture_id = job.capture_id().to_owned();
        job.begin_upload_preparation().unwrap();

        assert_eq!(job.state(), CompletionJobState::PreparingUpload);
        assert_eq!(job.id(), original_id);
        assert_eq!(job.capture_id(), original_capture_id);
        assert_eq!(job.attempts(), 2);
    }

    // TEST-51: A retryable failure remains discoverable for a later retry.
    #[test]
    fn retryable_failure_returns_to_resumable_state() {
        let mut job = CompletionJob::new("job-051", "capture-051");
        job.begin_upload_preparation().unwrap();
        job.mark_ready_for_upload().unwrap();
        job.mark_uploading().unwrap();
        job.mark_retryable_failure().unwrap();

        assert_eq!(job.state(), CompletionJobState::FailedRetryable);
        assert!(job.state().is_resumable());
    }

    // TEST-52: Upload preparation contains deterministic FLAC bytes and integrity metadata.
    #[test]
    fn upload_preparation_contains_flac_and_integrity_metadata() {
        let artifact = test_artifact("artifact-052");
        let mut job = CompletionJob::new("job-052", "capture-052");
        job.begin_upload_preparation().unwrap();

        let prepared = job.prepare_upload(&artifact).unwrap();
        assert_eq!(prepared.artifact_id(), "artifact-052");
        assert_eq!(prepared.manifest_hash(), artifact.manifest_hash());
        assert_eq!(prepared.tracks().len(), 1);
        assert!(!prepared.tracks()[0].data().is_empty());
        assert_eq!(
            prepared.tracks()[0].size_bytes(),
            prepared.tracks()[0].data().len() as u64
        );
        assert_ne!(
            prepared.tracks()[0].hash(),
            crate::artifact::PayloadHash::from_bytes(b"")
        );
    }

    // TEST-53: A resumed job cannot silently switch to another artifact identity.
    #[test]
    fn upload_preparation_rejects_artifact_identity_mismatch() {
        let artifact = test_artifact("artifact-053");
        let other = test_artifact("artifact-053-other");
        let mut job = CompletionJob::new("job-053", "capture-053");
        job.set_artifact_id(artifact.id.value());
        job.begin_upload_preparation().unwrap();

        let error = job.prepare_upload(&other).unwrap_err();
        assert_eq!(error, CompletionJobError::ArtifactMismatch);
    }
}
