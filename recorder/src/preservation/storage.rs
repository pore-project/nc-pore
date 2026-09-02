//! Durable local storage for preserved captures.
//!
//! Payload bytes remain opaque and are verified by size and SHA-256 when a
//! preserved capture is restored after restart.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::audio::{
    CaptureChunk, CaptureResult, CaptureSourceProvenance, CaptureStatus, CaptureTrack,
    RecordingChunkDuration, RecordingConfiguration, SampleFormat,
};

use super::PreservedCapture;

#[derive(Debug, Serialize, Deserialize)]
struct PersistedCapture {
    id: String,
    status: PersistedCaptureStatus,
    tracks: Vec<PersistedCaptureTrack>,
}

#[derive(Debug, Serialize, Deserialize)]
enum PersistedCaptureStatus {
    Completed,
    Failed(String),
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedCaptureTrack {
    id: String,
    configuration: Option<PersistedConfiguration>,
    source_provenance: Option<PersistedSourceProvenance>,
    chunks: Vec<PersistedChunk>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedConfiguration {
    sample_rate_hz: u32,
    channels: u16,
    sample_format: PersistedSampleFormat,
    chunk_duration_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize)]
enum PersistedSampleFormat {
    Pcm16,
    Pcm24,
    F32,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedSourceProvenance {
    source_id: String,
    label: Option<String>,
    started_at_unix_ms: u64,
    ended_at_unix_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PersistedChunk {
    sequence: u32,
    payload_size_bytes: u64,
    payload_sha256: [u8; 32],
}

/// Result of restoring one preserved capture from durable local storage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreservationLoadResult {
    Valid(PreservedCapture),
    Incomplete,
    Inconsistent,
    NotFound,
}

/// Durable local store for preserved captures.
pub struct FilesystemPreservationStore {
    root: PathBuf,
}

impl FilesystemPreservationStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let root = path.into();
        fs::create_dir_all(&root).expect("failed to create preservation directory");
        Self { root }
    }

    fn validate_id(id: &str) -> bool {
        !id.is_empty() && id != "." && id != ".." && !id.contains('/') && !id.contains('\\')
    }

    fn capture_dir(&self, id: &str) -> PathBuf {
        self.root.join(id)
    }

    fn payload_path(capture_dir: &Path, track_id: &str, sequence: u32) -> PathBuf {
        capture_dir
            .join("tracks")
            .join(track_id)
            .join(format!("chunk-{sequence:06}.payload"))
    }

    fn metadata(capture: &PreservedCapture) -> PersistedCapture {
        PersistedCapture {
            id: capture.id().to_owned(),
            status: match capture.status() {
                CaptureStatus::Completed => PersistedCaptureStatus::Completed,
                CaptureStatus::Failed(message) => PersistedCaptureStatus::Failed(message.clone()),
            },
            tracks: capture
                .tracks()
                .iter()
                .map(|track| PersistedCaptureTrack {
                    id: track.id.value().to_owned(),
                    configuration: track.configuration().map(|configuration| {
                        PersistedConfiguration {
                            sample_rate_hz: configuration.sample_rate_hz(),
                            channels: configuration.channels(),
                            sample_format: match configuration.sample_format() {
                                SampleFormat::Pcm16 => PersistedSampleFormat::Pcm16,
                                SampleFormat::Pcm24 => PersistedSampleFormat::Pcm24,
                                SampleFormat::F32 => PersistedSampleFormat::F32,
                            },
                            chunk_duration_seconds: configuration.chunk_duration().seconds(),
                        }
                    }),
                    source_provenance: track.source_provenance().map(|provenance| {
                        PersistedSourceProvenance {
                            source_id: provenance.source_id().to_owned(),
                            label: provenance.label().map(str::to_owned),
                            started_at_unix_ms: provenance.started_at_unix_ms(),
                            ended_at_unix_ms: provenance.ended_at_unix_ms(),
                        }
                    }),
                    chunks: track
                        .chunks()
                        .iter()
                        .map(|chunk| PersistedChunk {
                            sequence: chunk.sequence,
                            payload_size_bytes: chunk.payload().len() as u64,
                            payload_sha256: sha256(chunk.payload()),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    fn restore(metadata: PersistedCapture, capture_dir: &Path) -> PreservationLoadResult {
        if !Self::validate_id(&metadata.id) {
            return PreservationLoadResult::Inconsistent;
        }

        let mut capture = match metadata.status {
            PersistedCaptureStatus::Completed => CaptureResult::new(metadata.id),
            PersistedCaptureStatus::Failed(message) => CaptureResult::failed(metadata.id, message),
        };

        for persisted_track in metadata.tracks {
            if !Self::validate_id(&persisted_track.id) {
                return PreservationLoadResult::Inconsistent;
            }

            let configuration = persisted_track.configuration.map_or_else(
                || Ok(None),
                |configuration| {
                    let duration = match configuration.chunk_duration_seconds {
                        10 => RecordingChunkDuration::TenSeconds,
                        30 => RecordingChunkDuration::ThirtySeconds,
                        60 => RecordingChunkDuration::OneMinute,
                        120 => RecordingChunkDuration::TwoMinutes,
                        300 => RecordingChunkDuration::FiveMinutes,
                        600 => RecordingChunkDuration::TenMinutes,
                        _ => return Err(()),
                    };
                    let format = match configuration.sample_format {
                        PersistedSampleFormat::Pcm16 => SampleFormat::Pcm16,
                        PersistedSampleFormat::Pcm24 => SampleFormat::Pcm24,
                        PersistedSampleFormat::F32 => SampleFormat::F32,
                    };
                    Ok(Some(RecordingConfiguration::with_chunk_duration(
                        configuration.sample_rate_hz,
                        configuration.channels,
                        format,
                        duration,
                    )))
                },
            );
            let Ok(configuration) = configuration else {
                return PreservationLoadResult::Inconsistent;
            };

            let mut track = match configuration {
                Some(configuration) => {
                    CaptureTrack::with_configuration(persisted_track.id.clone(), configuration)
                }
                None => CaptureTrack::new(persisted_track.id.clone()),
            };

            if let Some(provenance) = persisted_track.source_provenance {
                let mut restored = CaptureSourceProvenance::new(
                    provenance.source_id,
                    provenance.started_at_unix_ms,
                );
                if let Some(label) = provenance.label {
                    restored = restored.with_label(label);
                }
                if let Some(ended_at) = provenance.ended_at_unix_ms {
                    restored = restored.ended_at(ended_at);
                }
                track.set_source_provenance(restored);
            }

            for chunk in persisted_track.chunks {
                let path = Self::payload_path(capture_dir, &persisted_track.id, chunk.sequence);
                if !path.is_file() {
                    return PreservationLoadResult::Incomplete;
                }
                let payload = match fs::read(path) {
                    Ok(payload) => payload,
                    Err(_) => return PreservationLoadResult::Inconsistent,
                };
                if payload.len() as u64 != chunk.payload_size_bytes
                    || sha256(&payload) != chunk.payload_sha256
                {
                    return PreservationLoadResult::Inconsistent;
                }
                track.add_chunk(CaptureChunk::with_payload(chunk.sequence, payload));
            }
            capture.add_track(track);
        }

        PreservationLoadResult::Valid(PreservedCapture::from_capture_result(capture))
    }

    /// Durably stores a preserved capture without changing its representation.
    pub fn store(&self, capture: &PreservedCapture) -> Result<(), std::io::Error> {
        if !Self::validate_id(capture.id())
            || capture
                .tracks()
                .iter()
                .any(|track| !Self::validate_id(track.id.value()))
        {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "invalid capture or track id",
            ));
        }

        let metadata = Self::metadata(capture);
        let capture_dir = self.capture_dir(capture.id());
        let temp_dir = self.root.join(format!(".{}.tmp", capture.id()));
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir)?;
        let metadata_json = serde_json::to_string_pretty(&metadata)
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
        fs::write(temp_dir.join("capture.json"), metadata_json)?;

        for track in capture.tracks() {
            for chunk in track.chunks() {
                let path = Self::payload_path(&temp_dir, track.id.value(), chunk.sequence);
                fs::create_dir_all(path.parent().expect("payload path has parent"))?;
                fs::write(path, chunk.payload())?;
            }
        }

        let _ = fs::remove_dir_all(&capture_dir);
        fs::rename(temp_dir, capture_dir)
    }

    /// Restores and verifies a preserved capture after a restart.
    pub fn load(&self, id: &str) -> PreservationLoadResult {
        if !Self::validate_id(id) {
            return PreservationLoadResult::Inconsistent;
        }
        let capture_dir = self.capture_dir(id);
        if !capture_dir.is_dir() {
            return PreservationLoadResult::NotFound;
        }
        let metadata_path = capture_dir.join("capture.json");
        if !metadata_path.is_file() {
            return PreservationLoadResult::Incomplete;
        }
        let content = match fs::read_to_string(metadata_path) {
            Ok(content) => content,
            Err(_) => return PreservationLoadResult::Inconsistent,
        };
        let metadata = match serde_json::from_str::<PersistedCapture>(&content) {
            Ok(metadata) => metadata,
            Err(_) => return PreservationLoadResult::Inconsistent,
        };
        Self::restore(metadata, &capture_dir)
    }
}

fn sha256(payload: &[u8]) -> [u8; 32] {
    Sha256::digest(payload).into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_directory(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("nc-pore-preservation-{name}-{suffix}"))
    }

    fn test_capture() -> PreservedCapture {
        let mut result = CaptureResult::new("capture-persisted");
        let configuration = RecordingConfiguration::with_chunk_duration(
            48_000,
            2,
            SampleFormat::Pcm24,
            RecordingChunkDuration::ThirtySeconds,
        );
        let mut track = CaptureTrack::with_configuration("track-mic", configuration);
        track.set_source_provenance(
            CaptureSourceProvenance::new("device-1", 1_762_000_000_000)
                .with_label("Microphone")
                .ended_at(1_762_000_005_000),
        );
        track.add_chunk(CaptureChunk::with_payload(1, vec![1, 2, 3, 4, 5]));
        result.add_track(track);
        CapturePreserver::preserve(result)
    }

    // TEST-45
    #[test]
    fn store_and_load_preserved_capture_round_trip() {
        let root = test_directory("round-trip");
        let store = FilesystemPreservationStore::new(&root);
        let capture = test_capture();
        store.store(&capture).expect("capture should be stored");

        let restored = match store.load("capture-persisted") {
            PreservationLoadResult::Valid(capture) => capture,
            other => panic!("capture should restore, got {other:?}"),
        };
        assert_eq!(restored, capture);
        let _ = fs::remove_dir_all(root);
    }

    // TEST-46
    #[test]
    fn missing_payload_is_reported_as_incomplete() {
        let root = test_directory("missing-payload");
        let store = FilesystemPreservationStore::new(&root);
        store.store(&test_capture()).expect("capture should be stored");
        let payload = root
            .join("capture-persisted")
            .join("tracks")
            .join("track-mic")
            .join("chunk-000001.payload");
        fs::remove_file(payload).expect("payload should exist");
        assert_eq!(store.load("capture-persisted"), PreservationLoadResult::Incomplete);
        let _ = fs::remove_dir_all(root);
    }

    // TEST-47
    #[test]
    fn modified_payload_is_reported_as_inconsistent() {
        let root = test_directory("modified-payload");
        let store = FilesystemPreservationStore::new(&root);
        store.store(&test_capture()).expect("capture should be stored");
        let payload = root
            .join("capture-persisted")
            .join("tracks")
            .join("track-mic")
            .join("chunk-000001.payload");
        fs::write(payload, [9, 8, 7]).expect("payload should be mutable in the test");
        assert_eq!(store.load("capture-persisted"), PreservationLoadResult::Inconsistent);
        let _ = fs::remove_dir_all(root);
    }
}
