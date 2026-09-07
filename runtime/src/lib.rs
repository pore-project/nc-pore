//! Host-neutral PoRE Runtime protocol boundary.
//!
//! The runtime knows neither Nextcloud nor Talk. Host adapters exchange commands
//! through this protocol; lifecycle orchestration belongs to Application and
//! lifecycle truth belongs to Core.

pub mod production;

use nc_pore_application::recording_coordinator::RecordingCoordinator;
use nc_pore_application::recording_state::{
    ClientRecordingPhase, ClientRecordingRole, ClientRecordingState,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::RecordingId;
use nc_pore_core::session::ProductionSessionError;
use nc_pore_core::session::repository::ProductionSessionRepository;
use serde::{Deserialize, Serialize};
use std::io::{self, Read, Write};

pub const PROTOCOL_VERSION: u16 = 1;
pub const OPERATION_SUBMIT_FINALIZED_ARTIFACT: &str = "recording.submit_finalized_artifact";
pub const OPERATION_RECORDING_COMMAND: &str = "recording.command";

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingCommandRequest {
    pub protocol_version: u16,
    pub operation: String,
    pub request_id: String,
    pub session_id: String,
    pub actor_id: String,
    pub recording_id: String,
    pub command: RecordingCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RecordingCommand {
    EnsureRecording,
    Begin { participants: Vec<String> },
    MarkReady,
    Start,
    RequestStop,
    AcknowledgeStop,
    Complete { artifact_id: String },
    Snapshot,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingCommandResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub status: String,
    pub state: Option<RecordingStateDto>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingStateDto {
    pub recording_id: String,
    pub phase: String,
    pub role: String,
    pub participants: Vec<RecordingParticipantDto>,
    pub confirmed: bool,
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecordingParticipantDto {
    pub id: String,
    pub ready: bool,
}

impl From<ClientRecordingState> for RecordingStateDto {
    fn from(state: ClientRecordingState) -> Self {
        Self {
            recording_id: state.recording_id,
            phase: match state.phase {
                ClientRecordingPhase::Preparing => "preparing",
                ClientRecordingPhase::Ready => "ready",
                ClientRecordingPhase::Recording => "recording",
                ClientRecordingPhase::Stopped => "stopped",
                ClientRecordingPhase::Completed => "completed",
            }
            .to_owned(),
            role: match state.role {
                ClientRecordingRole::Host => "host",
                ClientRecordingRole::Participant => "participant",
                ClientRecordingRole::Listener => "listener",
            }
            .to_owned(),
            participants: state
                .participants
                .into_iter()
                .map(|participant| RecordingParticipantDto {
                    id: participant.id,
                    ready: participant.ready,
                })
                .collect(),
            confirmed: state.confirmed,
            artifact_id: state.artifact_id,
        }
    }
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
    let payload_len = usize::try_from(request.payload_length)
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
    write_frame(writer, &bytes)
}

pub fn handle_recording_command<R: ProductionSessionRepository>(
    request: &RecordingCommandRequest,
    repository: &mut R,
) -> RecordingCommandResponse {
    if request.protocol_version != PROTOCOL_VERSION {
        return command_error(request, "unsupported_protocol_version");
    }
    if request.operation != OPERATION_RECORDING_COMMAND {
        return command_error(request, "unsupported_operation");
    }

    let mut coordinator = RecordingCoordinator::new(
        repository,
        ProductionId::new(&request.session_id),
        ParticipantId::new(&request.actor_id),
        RecordingId::new(&request.recording_id),
    );

    let result = match &request.command {
        RecordingCommand::EnsureRecording => coordinator.ensure_recording().map(|_| None),
        RecordingCommand::Begin { participants } => coordinator
            .begin(participants.iter().cloned().map(ParticipantId::new))
            .map(Some),
        RecordingCommand::MarkReady => coordinator.mark_ready().map(Some),
        RecordingCommand::Start => coordinator.start().map(Some),
        RecordingCommand::RequestStop => coordinator.request_stop().map(Some),
        RecordingCommand::AcknowledgeStop => coordinator.acknowledge_stop().map(Some),
        RecordingCommand::Complete { artifact_id } => coordinator.complete(artifact_id).map(Some),
        RecordingCommand::Snapshot => coordinator.snapshot().map(Some),
    };

    match result {
        Ok(state) => RecordingCommandResponse {
            protocol_version: PROTOCOL_VERSION,
            request_id: request.request_id.clone(),
            status: "ok".to_owned(),
            state: state.map(RecordingStateDto::from),
            error_code: None,
        },
        Err(error) => command_error(request, error_code(error)),
    }
}

fn command_error(request: &RecordingCommandRequest, error_code: &str) -> RecordingCommandResponse {
    RecordingCommandResponse {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id.clone(),
        status: "rejected".to_owned(),
        state: None,
        error_code: Some(error_code.to_owned()),
    }
}

fn error_code(error: ProductionSessionError) -> &'static str {
    match error {
        ProductionSessionError::Unauthorized => "unauthorized",
        ProductionSessionError::InvalidStateTransition => "invalid_state_transition",
        ProductionSessionError::ParticipantAlreadyExists => "participant_already_exists",
        ProductionSessionError::MissingOwner => "missing_owner",
        ProductionSessionError::RecordingAlreadyExists => "recording_already_exists",
        ProductionSessionError::RecordingNotFound => "recording_not_found",
        ProductionSessionError::RecordingLifecycle(_) => "recording_lifecycle_error",
        ProductionSessionError::RecordingCoordinationNotFound => "recording_coordination_not_found",
        ProductionSessionError::RecordingCoordinationAlreadyActive => {
            "recording_coordination_already_active"
        }
        ProductionSessionError::RecordingCoordination(_) => "recording_coordination_error",
    }
}

fn write_frame<W: Write>(writer: &mut W, bytes: &[u8]) -> Result<(), RuntimeProtocolError> {
    let len = u32::try_from(bytes.len())
        .map_err(|_| RuntimeProtocolError::InvalidHeader("response too large".to_owned()))?;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(bytes)?;
    writer.flush()?;
    Ok(())
}

fn read_u32<R: Read>(reader: &mut R) -> Result<u32, RuntimeProtocolError> {
    let mut bytes = [0_u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_be_bytes(bytes))
}

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
