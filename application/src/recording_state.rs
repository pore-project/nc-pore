use nc_pore_core::recording::{RecordingCoordinationStatus, RecordingStatus};
use nc_pore_core::role::ProductionAction;
use nc_pore_core::session::ProductionSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRecordingPhase {
    Preparing,
    Ready,
    Recording,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRecordingRole {
    Host,
    Participant,
    Listener,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRecordingParticipant {
    pub id: String,
    pub ready: bool,
}

/// Authoritative application read model for the recording surface.
///
/// This is deliberately a projection of the Core-owned ProductionSession. It
/// contains no recording lifecycle logic and does not invent states that Core
/// cannot currently persist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRecordingState {
    pub recording_id: String,
    pub phase: ClientRecordingPhase,
    pub role: ClientRecordingRole,
    pub participants: Vec<ClientRecordingParticipant>,
    pub confirmed: bool,
    pub artifact_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordingStateError {
    RecordingNotFound,
    RecordingCoordinationNotFound,
}

pub fn recording_state(
    session: &ProductionSession,
    actor_id: &str,
    recording_id: &str,
) -> Result<ClientRecordingState, RecordingStateError> {
    let recording = session
        .recordings()
        .iter()
        .find(|recording| recording.id().value() == recording_id)
        .ok_or(RecordingStateError::RecordingNotFound)?;

    let participation = session
        .participations()
        .iter()
        .find(|participation| participation.participant_id.value() == actor_id);

    let role = match participation {
        Some(participation) if participation.allows(ProductionAction::ManageRecordings) => {
            ClientRecordingRole::Host
        }
        Some(participation) if participation.allows(ProductionAction::ParticipateInRecording) => {
            ClientRecordingRole::Participant
        }
        _ => ClientRecordingRole::Listener,
    };

    let coordination = session
        .recording_coordination()
        .filter(|coordination| coordination.recording_id().value() == recording_id);

    let (phase, participants) = match recording.status() {
        RecordingStatus::Completed => (
            ClientRecordingPhase::Completed,
            coordination
                .map(|coordination| {
                    coordination
                        .participants()
                        .iter()
                        .map(|participant| ClientRecordingParticipant {
                            id: participant.value().to_owned(),
                            ready: coordination.ready_participants().contains(participant),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        ),
        RecordingStatus::Recording => (
            ClientRecordingPhase::Recording,
            coordination
                .map(|coordination| {
                    coordination
                        .participants()
                        .iter()
                        .map(|participant| ClientRecordingParticipant {
                            id: participant.value().to_owned(),
                            ready: coordination.ready_participants().contains(participant),
                        })
                        .collect()
                })
                .unwrap_or_default(),
        ),
        RecordingStatus::Prepared => {
            let coordination = coordination.ok_or(RecordingStateError::RecordingCoordinationNotFound)?;
            let phase = match coordination.status() {
                RecordingCoordinationStatus::Ready => ClientRecordingPhase::Ready,
                RecordingCoordinationStatus::Preparing
                | RecordingCoordinationStatus::WaitingForReady => ClientRecordingPhase::Preparing,
            };
            (
                phase,
                coordination
                    .participants()
                    .iter()
                    .map(|participant| ClientRecordingParticipant {
                        id: participant.value().to_owned(),
                        ready: coordination.ready_participants().contains(participant),
                    })
                    .collect(),
            )
        }
    };

    Ok(ClientRecordingState {
        recording_id: recording.id().value().to_owned(),
        phase,
        role,
        participants,
        confirmed: recording.status() == RecordingStatus::Completed,
        artifact_id: recording.artifact_id().map(|id| id.value().to_owned()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participant::ParticipantId;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::Recording;
    use nc_pore_core::role::ParticipantRole;

    fn session_with_recording() -> ProductionSession {
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
                Participation::new(bob.clone(), ParticipantRole::Participant),
            )
            .unwrap();
        session.start_by(&owner).unwrap();
        session
            .add_recording_by(&owner, Recording::new("recording-001"))
            .unwrap();
        session
            .begin_recording_by(
                &owner,
                &nc_pore_core::recording::RecordingId::new("recording-001"),
                [owner.clone(), bob],
            )
            .unwrap();
        session
    }

    #[test]
    fn reports_core_preparation_and_roles() {
        let session = session_with_recording();
        let state = recording_state(&session, "alice", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Preparing);
        assert_eq!(state.role, ClientRecordingRole::Host);
        assert_eq!(state.participants.len(), 2);
        assert!(!state.confirmed);
    }

    #[test]
    fn reports_core_ready_aggregate_without_local_state() {
        let mut session = session_with_recording();
        let owner = ParticipantId::new("alice");
        session
            .mark_recording_ready_by(
                &owner,
                &nc_pore_core::recording::RecordingId::new("recording-001"),
            )
            .unwrap();
        let state = recording_state(&session, "bob", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Preparing);
        assert_eq!(state.participants.iter().filter(|p| p.ready).count(), 1);
        assert_eq!(state.role, ClientRecordingRole::Participant);
    }

    #[test]
    fn reports_recording_and_completion_from_core_recording() {
        let mut session = session_with_recording();
        let alice = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let recording_id = nc_pore_core::recording::RecordingId::new("recording-001");
        session.mark_recording_ready_by(&alice, &recording_id).unwrap();
        session.mark_recording_ready_by(&bob, &recording_id).unwrap();
        session.start_recording_by(&alice, &recording_id).unwrap();

        let state = recording_state(&session, "bob", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Recording);

        session
            .complete_recording_by(
                &alice,
                &recording_id,
                nc_pore_core::recording::RecordingArtifactId::new("artifact-001"),
            )
            .unwrap();
        let state = recording_state(&session, "alice", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Completed);
        assert!(state.confirmed);
        assert_eq!(state.artifact_id.as_deref(), Some("artifact-001"));
    }
}
