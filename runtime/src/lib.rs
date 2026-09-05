//! Minimal, host-neutral PoRE Runtime protocol boundary.
//!
//! The runtime is deliberately unaware of Nextcloud and Talk. It validates a
//! framed finalized-artifact request and returns a protocol-level result, but
//! it does not persist the artifact. Host adapters own the authoritative
//! storage lifecycle.

use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

pub const PROTOCOL_VERSION: u16 = 1;
pub const OPERATION_SUBMIT_FINALIZED_ARTIFACT: &str = "recording.submit_finalized_artifact";

/// Metadata supplied by a host adapter. No host-specific types are allowed.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitFinalizedArtifactRequest {
    pub protocol_version: u16,
    pub operation: String,
    pub request_id: String,
    pub capture_id: String,
    pub recording_session_id: String,
    pub production_id: String,
    pub recording_id: String,
    pub track_id: String,
    pub sample_rate_hz: u32,
    pub channels: u16,
    pub payload_length: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubmitFinalizedArtifactResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub status: String,
    pub artifact_id: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Debug)]
pub enum RuntimeProtocolError {
    Io(io::Error),
    InvalidHeader(String),
    InvalidPayloadLength,
    Json(serde_json::Error),
}

impl From<io::Error> for RuntimeProtocolError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for RuntimeProtocolError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

/// Reads one request frame from stdin.
///
/// Frame format:
///   4-byte big-endian JSON header length
///   UTF-8 JSON header
///   raw payload bytes, whose length is declared by `payload_length`
pub fn read_request<R: Read>(
    reader: &mut R,
) -> Result<(SubmitFinalizedArtifactRequest, Vec<u8>), RuntimeProtocolError> {
    let header_len = read_u32(reader)? as usize;
    if header_len == 0 || header_len > 1024 * 1024 {
        return Err(RuntimeProtocolError::InvalidHeader(
            "invalid header length".to_owned(),
        ));
    }

    let mut header = vec![0_u8; header_len];
    reader.read_exact(&mut header)?;
    let request: SubmitFinalizedArtifactRequest = serde_json::from_slice(&header)?;

    if request.protocol_version != PROTOCOL_VERSION {
        return Err(RuntimeProtocolError::InvalidHeader(
            "unsupported protocol version".to_owned(),
        ));
    }
    if request.operation != OPERATION_SUBMIT_FINALIZED_ARTIFACT {
        return Err(RuntimeProtocolError::InvalidHeader(
            "unsupported operation".to_owned(),
        ));
    }

    let payload_len =
        usize::try_from(request.payload_length)
            .map_err(|_| RuntimeProtocolError::InvalidPayloadLength)?;
    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload)?;
    Ok((request, payload))
}

pub fn write_response<W: Write>(
    writer: &mut W,
    response: &SubmitFinalizedArtifactResponse,
) -> Result<(), RuntimeProtocolError> {
    let bytes = serde_json::to_vec(response)?;
    let len = u32::try_from(bytes.len()).map_err(|_| {
        RuntimeProtocolError::InvalidHeader("response header too large".to_owned())
    })?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&bytes)?;
    writer.flush()?;
    Ok(())
}

/// Validates a finalized artifact at the host-neutral protocol boundary.
///
/// This function deliberately does not persist the payload. In V1, the
/// Nextcloud adapter owns authoritative storage and integrity confirmation.
pub fn handle_submit(
    request: &SubmitFinalizedArtifactRequest,
    payload: &[u8],
) -> SubmitFinalizedArtifactResponse {
    if request.payload_length != payload.len() as u64 {
        return SubmitFinalizedArtifactResponse {
            protocol_version: PROTOCOL_VERSION,
            request_id: request.request_id.clone(),
            status: "rejected".to_owned(),
            artifact_id: None,
            error_code: Some("payload_length_mismatch".to_owned()),
        };
    }

    SubmitFinalizedArtifactResponse {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id.clone(),
        status: "accepted".to_owned(),
        artifact_id: Some(request.capture_id.clone()),
        error_code: None,
    }
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, RuntimeProtocolError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request() -> SubmitFinalizedArtifactRequest {
        SubmitFinalizedArtifactRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: OPERATION_SUBMIT_FINALIZED_ARTIFACT.to_owned(),
            request_id: "request-001".to_owned(),
            capture_id: "capture-001".to_owned(),
            recording_session_id: "session-001".to_owned(),
            production_id: "production-001".to_owned(),
            recording_id: "recording-001".to_owned(),
            track_id: "track-001".to_owned(),
            sample_rate_hz: 48_000,
            channels: 1,
            payload_length: 4,
        }
    }

    #[test]
    fn request_frame_round_trips_without_base64() {
        let request = request();
        let header = serde_json::to_vec(&request).expect("header should serialize");
        let mut frame = Vec::new();
        frame.extend_from_slice(&(header.len() as u32).to_be_bytes());
        frame.extend_from_slice(&header);
        frame.extend_from_slice(&[1, 2, 3, 4]);

        let (decoded, payload) = read_request(&mut frame.as_slice()).expect("frame should parse");
        assert_eq!(decoded, request);
        assert_eq!(payload, vec![1, 2, 3, 4]);
    }

    #[test]
    fn accepted_request_does_not_claim_persistence() {
        let request = request();
        let response = handle_submit(&request, &[1, 2, 3, 4]);

        assert_eq!(response.status, "accepted");
        assert_eq!(response.artifact_id.as_deref(), Some("capture-001"));
        assert_eq!(response.error_code, None);
    }

    #[test]
    fn payload_length_mismatch_is_rejected() {
        let request = request();
        let response = handle_submit(&request, &[1, 2, 3]);

        assert_eq!(response.status, "rejected");
        assert_eq!(response.artifact_id, None);
        assert_eq!(
            response.error_code.as_deref(),
            Some("payload_length_mismatch")
        );
    }

    #[test]
    fn response_is_a_small_json_frame() {
        let response = SubmitFinalizedArtifactResponse {
            protocol_version: PROTOCOL_VERSION,
            request_id: "request-001".to_owned(),
            status: "accepted".to_owned(),
            artifact_id: Some("capture-001".to_owned()),
            error_code: None,
        };
        let mut output = Vec::new();
        write_response(&mut output, &response).expect("response should serialize");

        let len = u32::from_be_bytes(output[0..4].try_into().unwrap()) as usize;
        let decoded: SubmitFinalizedArtifactResponse =
            serde_json::from_slice(&output[4..4 + len]).expect("response should decode");
        assert_eq!(decoded, response);
    }
}
