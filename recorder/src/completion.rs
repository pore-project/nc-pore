//! Durable completion-job state for local recovery.
//!
//! A completion job is the durable hand-off between local preservation and
//! later artifact/transport/upload work. The job contains only workflow state
//! and stable identifiers; audio payloads remain in the preservation store.
//! This makes an interrupted completion resumable after a client restart.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

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

    /// Advances the job to upload preparation and counts one processing attempt.
    pub fn begin_upload_preparation(&mut self) -> Result<(), CompletionJobError> {
        if !matches!(self.state, CompletionJobState::Pending | CompletionJobState::FailedRetryable)
        {
            return Err(CompletionJobError::InvalidTransition);
        }
        self.attempts = self.attempts.saturating_add(1);
        self.state = CompletionJobState::PreparingUpload;
        Ok(())
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionJobError {
    InvalidTransition,
    InvalidId,
    Io,
    Serialization,
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

        let bytes = serde_json::to_vec_pretty(job).map_err(|_| CompletionJobError::Serialization)?;
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
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("job.json");
    temp.set_file_name(format!(".{file_name}.tmp"));
    temp
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_store(name: &str) -> FilesystemCompletionJobStore {
        let root = std::env::temp_dir().join(format!("nc-pore-completion-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        FilesystemCompletionJobStore::new(root).unwrap()
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
        let mut pending = CompletionJob::new("job-049-a", "capture-049-a");
        pending.begin_upload_preparation().unwrap();
        store.save(&pending).unwrap();

        let uploaded = CompletionJob::new("job-049-b", "capture-049-b");
        store.save(&uploaded).unwrap();
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

    // TEST-50: Uploading work can be retried without losing its identity.
    #[test]
    fn retryable_failure_returns_to_resumable_state() {
        let mut job = CompletionJob::new("job-050", "capture-050");
        job.begin_upload_preparation().unwrap();
        job.mark_ready_for_upload().unwrap();
        job.mark_uploading().unwrap();
        job.mark_retryable_failure().unwrap();

        assert_eq!(job.state(), CompletionJobState::FailedRetryable);
        assert!(job.state().is_resumable());
        assert_eq!(job.attempts(), 1);
    }
}
