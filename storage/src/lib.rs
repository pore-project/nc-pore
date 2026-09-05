//! Concrete local persistence for application synchronization work.
//!
//! The application layer owns the synchronization-work contract. This crate
//! supplies its filesystem implementation without depending on any remote
//! provider or host integration.

use std::fs;
use std::path::{Path, PathBuf};

use nc_pore_application::synchronization::{
    SynchronizationWork, SynchronizationWorkStore, SynchronizationWorkStoreError,
};
use nc_pore_application::synchronization_metadata::ArtifactTransferMetadata;
use nc_pore_core::recording::RecordingArtifactSynchronizationStatus;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct PersistedSynchronizationWork {
    artifact_id: String,
    manifest_hash: [u8; 32],
    display_name: Option<String>,
    recorded_at: Option<String>,
    status: PersistedSynchronizationStatus,
}

impl From<&SynchronizationWork> for PersistedSynchronizationWork {
    fn from(work: &SynchronizationWork) -> Self {
        Self {
            artifact_id: work.artifact_id().value().to_owned(),
            manifest_hash: *work.manifest_hash(),
            display_name: work.metadata().display_name().map(str::to_owned),
            recorded_at: work.metadata().recorded_at().map(str::to_owned),
            status: PersistedSynchronizationStatus::from_core(work.status()),
        }
    }
}

impl PersistedSynchronizationWork {
    fn into_work(self) -> SynchronizationWork {
        SynchronizationWork::reconstitute_with_metadata(
            nc_pore_core::recording::RecordingArtifactId::new(self.artifact_id),
            self.manifest_hash,
            ArtifactTransferMetadata::new(self.display_name, self.recorded_at),
            self.status.into_core(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
enum PersistedSynchronizationStatus {
    Local,
    Pending,
    Transferring,
    Synchronized,
    Failed,
}

impl PersistedSynchronizationStatus {
    fn into_core(self) -> RecordingArtifactSynchronizationStatus {
        match self {
            Self::Local => RecordingArtifactSynchronizationStatus::Local,
            Self::Pending => RecordingArtifactSynchronizationStatus::Pending,
            Self::Transferring => RecordingArtifactSynchronizationStatus::Transferring,
            Self::Synchronized => RecordingArtifactSynchronizationStatus::Synchronized,
            Self::Failed => RecordingArtifactSynchronizationStatus::Failed,
        }
    }

    fn from_core(status: RecordingArtifactSynchronizationStatus) -> Self {
        match status {
            RecordingArtifactSynchronizationStatus::Local => Self::Local,
            RecordingArtifactSynchronizationStatus::Pending => Self::Pending,
            RecordingArtifactSynchronizationStatus::Transferring => Self::Transferring,
            RecordingArtifactSynchronizationStatus::Synchronized => Self::Synchronized,
            RecordingArtifactSynchronizationStatus::Failed => Self::Failed,
        }
    }
}

/// Concrete local persistence provider for synchronization work.
pub struct FilesystemSynchronizationWorkStore {
    path: PathBuf,
}

impl FilesystemSynchronizationWorkStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            path: root.into().join("synchronization-work.json"),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn read(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError> {
        if !self.path.is_file() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&self.path)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        let persisted: Vec<PersistedSynchronizationWork> = serde_json::from_str(&content)
            .map_err(|error| SynchronizationWorkStoreError::Serialization(error.to_string()))?;

        Ok(persisted
            .into_iter()
            .map(PersistedSynchronizationWork::into_work)
            .collect())
    }

    fn write(&self, work: &[SynchronizationWork]) -> Result<(), SynchronizationWorkStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        }

        let persisted: Vec<PersistedSynchronizationWork> = work
            .iter()
            .map(PersistedSynchronizationWork::from)
            .collect();
        let content = serde_json::to_string_pretty(&persisted)
            .map_err(|error| SynchronizationWorkStoreError::Serialization(error.to_string()))?;
        let temp_path = self.path.with_extension("json.tmp");

        fs::write(&temp_path, content)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;
        fs::rename(&temp_path, &self.path)
            .map_err(|error| SynchronizationWorkStoreError::Io(error.to_string()))?;

        Ok(())
    }
}

impl SynchronizationWorkStore for FilesystemSynchronizationWorkStore {
    fn save(&mut self, work: SynchronizationWork) -> Result<(), SynchronizationWorkStoreError> {
        let mut all = self.read()?;

        if let Some(existing) = all
            .iter_mut()
            .find(|existing| existing.artifact_id().value() == work.artifact_id().value())
        {
            *existing = work;
        } else {
            all.push(work);
        }

        all.sort_by(|left, right| left.artifact_id().value().cmp(right.artifact_id().value()));
        self.write(&all)
    }

    fn list(&self) -> Result<Vec<SynchronizationWork>, SynchronizationWorkStoreError> {
        let mut work = self.read()?;
        work.sort_by(|left, right| left.artifact_id().value().cmp(right.artifact_id().value()));
        Ok(work)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_application::synchronization::PersistentSynchronizationQueue;
    use nc_pore_core::recording::RecordingArtifactId;

    fn manifest_hash(value: u8) -> [u8; 32] {
        [value; 32]
    }

    #[test]
    fn filesystem_store_survives_reconstruction() {
        let root = std::env::temp_dir().join("nc-pore-sync-work-test-01");
        let _ = fs::remove_dir_all(&root);

        let mut first =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        first
            .enqueue(RecordingArtifactId::new("artifact-001"), manifest_hash(1))
            .unwrap();
        drop(first);

        let second =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        let work = second.list().unwrap();

        assert_eq!(work.len(), 1);
        assert_eq!(work[0].artifact_id().value(), "artifact-001");
        assert_eq!(
            work[0].status(),
            RecordingArtifactSynchronizationStatus::Pending
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn filesystem_store_preserves_transfer_metadata() {
        let root = std::env::temp_dir().join("nc-pore-sync-work-test-02");
        let _ = fs::remove_dir_all(&root);
        let metadata = ArtifactTransferMetadata::new(
            Some("Interview mit Frizz".to_owned()),
            Some("2026-08-22T18:30:00+02:00".to_owned()),
        );

        let mut first =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        first
            .enqueue_with_metadata(
                RecordingArtifactId::new("artifact-002"),
                manifest_hash(2),
                metadata.clone(),
            )
            .unwrap();
        drop(first);

        let second =
            PersistentSynchronizationQueue::new(FilesystemSynchronizationWorkStore::new(&root));
        let work = second.list().unwrap();

        assert_eq!(work.len(), 1);
        assert_eq!(work[0].metadata(), &metadata);
        assert_eq!(work[0].transfer_request().metadata(), &metadata);

        let _ = fs::remove_dir_all(root);
    }
}
