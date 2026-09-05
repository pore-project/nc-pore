use std::io::{Read, Write};
use std::process::{Command, Stdio};

use pore_runtime::{
    OPERATION_SUBMIT_FINALIZED_ARTIFACT, PROTOCOL_VERSION, SubmitFinalizedArtifactRequest,
    SubmitFinalizedArtifactResponse,
};

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
fn runtime_process_validates_without_persisting_a_finalized_artifact() {
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
    assert_eq!(response.status, "accepted");
    assert_eq!(
        response.artifact_id.as_deref(),
        Some(request.capture_id.as_str())
    );
    assert_eq!(response.error_code, None);
}
