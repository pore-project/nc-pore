use std::fs;
use std::path::{Path, PathBuf};

use crate::artifact::{RecordingArtifact, RecordingChunk, RecordingTrack};
use crate::persistence::{PersistenceLoadResult, PersistenceProvider};
use crate::session::RecordingSessionId;

/// Filesystem-backed implementation of the persistence boundary.
///
/// Each artifact is stored below its own directory. The provider is
/// responsible for translating filesystem state into persistence outcomes;
/// higher layers must not need to know about the storage layout.
pub struct FilesystemPersistenceProvider {
    root: PathBuf,
}

impl FilesystemPersistenceProvider {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        let _ = fs::create_dir_all(&root);
        Self { root }
    }

    fn artifact_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn write_artifact(&self, artifact: &RecordingArtifact) {
        let dir = self.artifact_dir(artifact.id.value());
        let _ = fs::create_dir_all(&dir);

        let metadata = serde_json::to_string_pretty(artifact).expect("artifact serialization");
        fs::write(dir.join("artifact.json"), metadata).expect("artifact metadata write");

        for track in artifact.tracks() {
            let chunks_dir = dir.join("tracks").join(track.id().value()).join("chunks");
            let _ = fs::create_dir_all(&chunks_dir);

            for chunk in track.chunks() {
                let payload_path = chunks_dir.join(format!("{}.payload", chunk.id()));
                fs::write(payload_path, chunk.payload().data()).expect("payload write");
            }
        }
    }

    fn validate_id(id: &str) -> bool {
        !id.is_empty()
            && id != "."
            && id != ".."
            && !id.contains('/')
            && !id.contains('\\')
    }

    fn read_artifact(dir: &Path) -> PersistenceLoadResult {
        let metadata_path = dir.join("artifact.json");
        if !metadata_path.is_file() {
            return PersistenceLoadResult::Incomplete;
        }

        let content = match fs::read_to_string(&metadata_path) {
            Ok(content) => content,
            Err(_) => return PersistenceLoadResult::Inconsistent,
        };

        let mut artifact: RecordingArtifact = match serde_json::from_str(&content) {
            Ok(artifact) => artifact,
            Err(_) => return PersistenceLoadResult::Inconsistent,
        };

        for track in artifact.tracks_mut() {
            for chunk in track.chunks_mut() {
                let payload_path = dir
                    .join("tracks")
                    .join(track.id().value())
                    .join("chunks")
                    .join(format!("{}.payload", chunk.id()));

                let payload = match fs::read(&payload_path) {
                    Ok(payload) => payload,
                    Err(_) => return PersistenceLoadResult::Incomplete,
                };

                if payload.len() as u64 != chunk.payload().size_bytes() {
                    return PersistenceLoadResult::Inconsistent;
                }

                chunk.replace_payload(payload);
            }
        }

        PersistenceLoadResult::Valid(artifact)
    }
}

impl PersistenceProvider for FilesystemPersistenceProvider {
    fn store(&mut self, artifact: RecordingArtifact) {
        self.write_artifact(&artifact);
    }

    fn load(&self, id: &str) -> PersistenceLoadResult {
        if !Self::validate_id(id) {
            return PersistenceLoadResult::Inconsistent;
        }

        Self::read_artifact(&self.artifact_dir(id))
    }

    fn list_ids(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;

                if name.starts_with('.') || !Self::validate_id(name) {
                    return None;
                }

                Some(name.to_owned())
            })
            .collect()
    }

    fn list(&self) -> Vec<RecordingArtifact> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };

        entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .filter_map(|entry| {
                let name = entry.file_name();
                let name = name.to_str()?;

                if name.starts_with('.') || !Self::validate_id(name) {
                    return None;
                }

                match Self::read_artifact(&entry.path()) {
                    PersistenceLoadResult::Valid(artifact) => Some(artifact),
                    PersistenceLoadResult::Incomplete
                    | PersistenceLoadResult::Inconsistent
                    | PersistenceLoadResult::NotFound => None,
                }
            })
            .collect()
    }

    fn remove(&mut self, id: &str) {
        if !Self::validate_id(id) {
            return;
        }

        let _ = fs::remove_dir_all(self.artifact_dir(id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_directory(name: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("nc-pore-{name}"));
        let _ = fs::remove_dir_all(&path);
        path
    }

    fn test_artifact() -> RecordingArtifact {
        let mut artifact =
            RecordingArtifact::new("artifact-001", RecordingSessionId::new("session-001"));

        artifact.set_domain_association("production-001", "recording-017");

        let mut host = RecordingTrack::new("track-host");
        host.add_chunk(RecordingChunk::with_payload(
            1,
            "track-host/chunk-000001",
            vec![1, 2, 3],
        ));
        host.add_chunk(RecordingChunk::with_payload(
            2,
            "track-host/chunk-000002",
            vec![4, 5],
        ));
        artifact.add_track(host);
        artifact.make_available();
        artifact.store();

        artifact
    }

    // TEST-16
    // Protects ADR-055: artifacts are stored below their own directory.
    #[test]
    fn test_16_filesystem_provider_can_store_artifact() {
        let path = test_directory("test-16");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());

        assert!(path.join("artifact-001/artifact.json").is_file());
        assert!(
            path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload")
                .is_file()
        );
        assert!(
            path.join("artifact-001/tracks/track-host/chunks/chunk-000002.payload")
                .is_file()
        );

        let _ = fs::remove_dir_all(path);
    }

    // TEST-17
    // Protects ADR-054/055 and ADR-059: persisted tracks, chunks,
    // payloads and the domain association can be restored.
    #[test]
    fn test_17_filesystem_provider_can_load_artifact_and_payload() {
        let path = test_directory("test-17");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        let result = provider.load("artifact-001");
        let PersistenceLoadResult::Valid(artifact) = result else {
            panic!("expected valid persisted artifact");
        };

        assert_eq!(artifact.recording_session_id.value(), "session-001");
        assert_eq!(artifact.production_id(), Some("production-001"));
        assert_eq!(artifact.recording_id(), Some("recording-017"));
        assert_eq!(artifact.tracks().len(), 1);
        assert_eq!(artifact.tracks()[0].chunks().len(), 2);
        assert_eq!(
            artifact.tracks()[0].chunks()[0].payload().data(),
            &[1, 2, 3]
        );
        assert_eq!(artifact.tracks()[0].chunks()[1].payload().data(), &[4, 5]);

        let _ = fs::remove_dir_all(path);
    }

    // TEST-18
    // Protects ADR-053/055: incomplete temporary directories are ignored.
    #[test]
    fn test_18_filesystem_provider_can_list_artifacts() {
        let path = test_directory("test-18");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::create_dir_all(path.join(".artifact-002.tmp")).unwrap();
        fs::write(path.join(".artifact-002.tmp/partial"), "incomplete").unwrap();

        assert_eq!(provider.list().len(), 1);

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn provider_can_list_artifact_ids() {
        let path = test_directory("list-ids");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::create_dir_all(path.join(".artifact-002.tmp")).unwrap();
        fs::create_dir_all(path.join("artifact-003")).unwrap();

        let ids = provider.list_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"artifact-001".to_owned()));
        assert!(ids.contains(&"artifact-003".to_owned()));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-19
    // Protects ADR-055: removal removes the complete artifact directory.
    #[test]
    fn test_19_filesystem_provider_can_remove_artifact() {
        let path = test_directory("test-19");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        provider.remove("artifact-001");

        assert!(!path.join("artifact-001").exists());
        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::NotFound
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-37
    // Protects ADR-059 and the persistence assessment boundary:
    // a missing payload is incomplete persisted data.
    #[test]
    fn missing_payload_is_incomplete() {
        let path = test_directory("missing-payload");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::remove_file(path.join("artifact-001/tracks/track-host/chunks/chunk-000001.payload"))
            .unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Incomplete
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-38
    // Protects the persistence assessment boundary:
    // payload size disagreement means persisted metadata and payload
    // disagree and therefore the artifact is inconsistent.
    #[test]
    fn payload_size_mismatch_is_inconsistent() {
        let path = test_directory("payload-size-mismatch");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());

        let metadata_path = path.join("artifact-001/artifact.json");
        let content = fs::read_to_string(&metadata_path).unwrap();
        let content = content.replace("\"payload_size_bytes\": 3", "\"payload_size_bytes\": 4");
        fs::write(metadata_path, content).unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    // TEST-39
    // Protects the persistence assessment boundary:
    // malformed metadata is inconsistent persisted data.
    #[test]
    fn malformed_metadata_is_inconsistent() {
        let path = test_directory("malformed-metadata");
        let mut provider = FilesystemPersistenceProvider::new(&path);

        provider.store(test_artifact());
        fs::write(path.join("artifact-001/artifact.json"), "not-json").unwrap();

        assert!(matches!(
            provider.load("artifact-001"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn invalid_ids_cannot_escape_persistence_root() {
        let path = test_directory("invalid-id");
        let provider = FilesystemPersistenceProvider::new(&path);

        assert!(matches!(
            provider.load("../outside"),
            PersistenceLoadResult::Inconsistent
        ));
        assert!(matches!(
            provider.load("nested/id"),
            PersistenceLoadResult::Inconsistent
        ));
        assert!(matches!(
            provider.load(r"nested\id"),
            PersistenceLoadResult::Inconsistent
        ));

        let _ = fs::remove_dir_all(path);
    }
}