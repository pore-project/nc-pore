//! Minimal, host-neutral PoRE Runtime protocol boundary.
//!
//! The runtime is deliberately unaware of Nextcloud and Talk. Host adapters
//! exchange commands through this protocol; lifecycle orchestration is owned
//! by Application::RecordingCoordinator and lifecycle truth by Core.

use nc_pore_application::recording_coordinator::RecordingCoordinator;
use nc_pore_application::recording_state::{
    ClientRecordingPhase, ClientRecordingRole, ClientRecordingState,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::RecordingId;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;
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

/// Dispatches a host-neutral recording command to the Application coordinator.
///
/// The repository is supplied by the runtime boundary. No recording lifecycle
/// state is maintained in this protocol layer.
pub fn handle_recording_command<R>(
    request: &RecordingCommandRequest,
    repository: &mut R,
) -> RecordingCommandResponse
where
    R: ProductionSessionRepository,
{
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

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::ProductionSession;

    struct InMemoryRepository {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemoryRepository {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            let existing = self
                .sessions
                .iter_mut()
                .find(|existing| existing.id == session.id)
                .ok_or("session not found")?;
            *existing = session.clone();
            Ok(())
        }

        fn get(
            &self,
            id: &ProductionId,
        ) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

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

    fn session_repository() -> InMemoryRepository {
        let owner = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let mut session = ProductionSession::new_with_actor(
            ProductionId::new("session-001"),
            Some(owner.clone()),
        );
        session
            .add_participation_by(
                &owner,
                Participation::with_roles(
                    owner.clone(),
                    [ParticipantRole::Owner, ParticipantRole::Producer],
                ),
            )
            .unwrap();
        session
            .add_participation_by(
                &owner,
                Participation::new(bob, ParticipantRole::Participant),
            )
            .unwrap();
        session.start_by(&owner).unwrap();
        session
            .add_recording_by(&owner, Recording::new("recording-001"))
            .unwrap();
        InMemoryRepository {
            sessions: vec![session],
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

    #[test]
    fn recording_commands_delegate_to_application_coordinator() {
        let mut repository = session_repository();
        let snapshot = RecordingCommandRequest {
            protocol_version: PROTOCOL_VERSION,
            operation: OPERATION_RECORDING_COMMAND.to_owned(),
            request_id: "command-001".to_owned(),
            session_id: "session-001".to_owned(),
            actor_id: "alice".to_owned(),
            recording_id: "recording-001".to_owned(),
            command: RecordingCommand::Snapshot,
        };
        let response = handle_recording_command(&snapshot, &mut repository);
        assert_eq!(response.status, "ok");
        assert_eq!(response.state.unwrap().phase, "preparing");
    }
}
