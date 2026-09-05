use recorder::persistence::{FilesystemPersistenceProvider, PersistenceLoadResult, PersistenceProvider};
use std::fs;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use pore_runtime::{
    SubmitFinalizedArtifactRequest, SubmitFinalizedArtifactResponse,
    OPERATION_SUBMIT_FINALIZED_ARTIFACT, PROTOCOL_VERSION,
};

fn temporary_persistence_root() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nc-pore-runtime-test-{nonce}"));
    fs::create_dir_all(&path).expect("temporary persistence root should be creatable");
    path
}

fn encode_request(request: &SubmitFinalizedArtifactRequest, payload: &[u8]) -> Vec<u8> {
    let header = serde_json::to_vec(request).expect("request should serialize");
    let mut frame = Vec::with_capacity(4 + header.len() + payload.len());
    frame.extend_from_slice(&(header.len() as u32).to_be_bytes());
    frame.extend_from_slice(&header);
    frame.extend_from_slice(payload);
    frame
}

fn decode_response(mut output: &[u8]) -> SubmitFinalizedArtifactResponse {
    let mut length = [0_u8; 4];
    output
        .read_exact(&mut length)
        .expect("runtime should return a response frame");
    let length = u32::from_be_bytes(length) as usize;
    let mut json = vec![0_u8; length];
    output
        .read_exact(&mut json)
        .expect("runtime response JSON should be complete");
    serde_json::from_slice(&json).expect("runtime response should be valid JSON")
}

#[test]
fn runtime_process_persists_a_finalized_browser_artifact_end_to_end() {
    let persistence_root = temporary_persistence_root();
    let payload = b"finalized-wav-payload";
    let request = SubmitFinalizedArtifactRequest {
        protocol_version: PROTOCOL_VERSION,
        operation: OPERATION_SUBMIT_FINALIZED_ARTIFACT.to_owned(),
        request_id: "request-runtime-e2e-001".to_owned(),
        capture_id: "capture-runtime-e2e-001".to_owned(),
        recording_session_id: "session-runtime-e2e-001".to_owned(),
        production_id: "production-runtime-e2e-001".to_owned(),
        recording_id: "recording-runtime-e2e-001".to_owned(),
        track_id: "track-runtime-e2e-001".to_owned(),
        sample_rate_hz: 48_000,
        channels: 1,
        payload_length: payload.len() as u64,
    };

    let mut child = Command::new(env!("CARGO_BIN_EXE_pore-runtime"))
        .env("PORE_PERSISTENCE_ROOT", &persistence_root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("runtime binary should start");

    child
        .stdin
        .take()
        .expect("runtime stdin should be available")
        .write_all(&encode_request(&request, payload))
        .expect("request frame should be writable");

    let output = child
        .wait_with_output()
        .expect("runtime process should finish");

    assert!(
        output.status.success(),
        "runtime failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let response = decode_response(&output.stdout);
    assert_eq!(response.protocol_version, PROTOCOL_VERSION);
    assert_eq!(response.request_id, request.request_id);
    assert_eq!(response.status, "stored");
    assert_eq!(response.artifact_id.as_deref(), Some(request.capture_id.as_str()));
    assert_eq!(response.error_code, None);

    let provider = FilesystemPersistenceProvider::new(&persistence_root);
    let stored = provider.load(&request.capture_id);
    match stored {
        PersistenceLoadResult::Valid(artifact) => {
            assert_eq!(artifact.id.value(), request.capture_id);
            assert_eq!(artifact.recording_session_id.value(), request.recording_session_id);
            assert_eq!(artifact.production_id(), Some(request.production_id.as_str()));
            assert_eq!(artifact.recording_id(), Some(request.recording_id.as_str()));
            assert_eq!(artifact.tracks().len(), 1);
            assert_eq!(artifact.tracks()[0].id.value(), request.track_id);
            assert_eq!(artifact.tracks()[0].chunks().len(), 1);
            assert_eq!(artifact.tracks()[0].chunks()[0].payload().data(), payload);
        }
        other => panic!("expected valid persisted artifact, got {other:?}"),
    }

    fs::remove_dir_all(&persistence_root).expect("temporary persistence root should be removable");
}
