use nc_pore_application::client::{ClientProductionStatus, ClientSessionError};
use nc_pore_application::production_coordinator::{ensure_production, start_production};
use nc_pore_core::session::repository::ProductionSessionRepository;
use serde::{Deserialize, Serialize};

use crate::PROTOCOL_VERSION;

pub const OPERATION_PRODUCTION_COMMAND: &str = "production.command";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductionCommandRequest {
    pub protocol_version: u16,
    pub operation: String,
    pub request_id: String,
    pub session_id: String,
    pub actor_id: String,
    pub owner_id: String,
    pub participants: Vec<String>,
    pub command: ProductionCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProductionCommand {
    Ensure,
    Start,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductionCommandResponse {
    pub protocol_version: u16,
    pub request_id: String,
    pub status: String,
    pub production_status: Option<String>,
    pub participants: Vec<String>,
    pub error_code: Option<String>,
}

pub fn handle_production_command<R: ProductionSessionRepository>(
    request: &ProductionCommandRequest,
    repository: &mut R,
) -> ProductionCommandResponse {
    if request.protocol_version != PROTOCOL_VERSION {
        return production_error(request, "unsupported_protocol_version");
    }
    if request.operation != OPERATION_PRODUCTION_COMMAND {
        return production_error(request, "unsupported_operation");
    }

    let result = match &request.command {
        ProductionCommand::Ensure => ensure_production(
            repository,
            &request.session_id,
            &request.actor_id,
            &request.owner_id,
            &request.participants,
        ),
        ProductionCommand::Start => {
            start_production(repository, &request.session_id, &request.actor_id)
        }
    };

    match result {
        Ok(session) => ProductionCommandResponse {
            protocol_version: PROTOCOL_VERSION,
            request_id: request.request_id.clone(),
            status: "ok".to_owned(),
            production_status: Some(
                match session.status {
                    ClientProductionStatus::Created => "created",
                    ClientProductionStatus::Active => "active",
                    ClientProductionStatus::Completed => "completed",
                }
                .to_owned(),
            ),
            participants: session
                .participants
                .into_iter()
                .map(|participant| participant.id)
                .collect(),
            error_code: None,
        },
        Err(error) => production_error(request, client_error_code(error)),
    }
}

fn production_error(
    request: &ProductionCommandRequest,
    error_code: &str,
) -> ProductionCommandResponse {
    ProductionCommandResponse {
        protocol_version: PROTOCOL_VERSION,
        request_id: request.request_id.clone(),
        status: "rejected".to_owned(),
        production_status: None,
        participants: Vec::new(),
        error_code: Some(error_code.to_owned()),
    }
}

fn client_error_code<E>(error: ClientSessionError<E>) -> &'static str {
    match error {
        ClientSessionError::SessionNotFound => "session_not_found",
        ClientSessionError::Repository(_) => "repository_error",
        ClientSessionError::Unauthorized => "unauthorized",
        ClientSessionError::InvalidStateTransition => "invalid_state_transition",
        ClientSessionError::ParticipantAlreadyExists => "participant_already_exists",
        ClientSessionError::MissingOwner => "missing_owner",
        ClientSessionError::RecordingNotFound => "recording_not_found",
    }
}
