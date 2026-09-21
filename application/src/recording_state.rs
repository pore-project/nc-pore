use nc_pore_core::recording::{RecordingCoordinationStatus, RecordingStatus};
use nc_pore_core::role::ProductionAction;
use nc_pore_core::session::ProductionSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRecordingPhase {
    Preparing,
    Ready,
    Opening,
    Recording,
    Stopped,
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
    pub opening_confirmed: bool,
    pub artifact_id: Option<String>,
}

/// Authoritative application read model for the recording surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRecordingState {
    pub recording_id: String,
    pub phase: ClientRecordingPhase,
    pub role: ClientRecordingRole,
    pub participants: Vec<ClientRecordingParticipant>,
    pub confirmed: bool,
    pub opening_triggered: bool,
    /// The current actor's participant Artifact, if one has been confirmed.
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

    let participants = recording
        .expected_participant_ids()
        .map(|participant| ClientRecordingParticipant {
            id: participant.value().to_owned(),
            ready: coordination
                .map(|value| value.ready_participants().contains(participant))
                .unwrap_or(false),
            opening_confirmed: coordination
                .map(|value| value.opening_confirmed_participants().contains(participant))
                .unwrap_or(false),
            artifact_id: recording
                .artifact_for_participant(participant)
                .map(|artifact| artifact.value().to_owned()),
        })
        .collect::<Vec<_>>();

    let phase = match recording.status() {
        RecordingStatus::Completed => ClientRecordingPhase::Completed,
        RecordingStatus::Stopped => ClientRecordingPhase::Stopped,
        RecordingStatus::Recording => match coordination
            .map(|value| value.status())
            .unwrap_or(RecordingCoordinationStatus::Preparing)
        {
            RecordingCoordinationStatus::Opening => ClientRecordingPhase::Opening,
            RecordingCoordinationStatus::Preparing
            | RecordingCoordinationStatus::WaitingForReady
            | RecordingCoordinationStatus::Ready
            | RecordingCoordinationStatus::Recording => ClientRecordingPhase::Recording,
        },
        RecordingStatus::Prepared => {
            let coordination =
                coordination.ok_or(RecordingStateError::RecordingCoordinationNotFound)?;
            match coordination.status() {
                RecordingCoordinationStatus::Ready => ClientRecordingPhase::Ready,
                RecordingCoordinationStatus::Preparing
                | RecordingCoordinationStatus::WaitingForReady
                | RecordingCoordinationStatus::Opening
                | RecordingCoordinationStatus::Recording => ClientRecordingPhase::Preparing,
            }
        }
    };

    Ok(ClientRecordingState {
        recording_id: recording.id().value().to_owned(),
        phase,
        role,
        participants,
        confirmed: recording.status() == RecordingStatus::Completed,
        opening_triggered: coordination.map(|value| value.opening_triggered()).unwrap_or(false),
        artifact_id: recording
            .artifact_for_participant(&nc_pore_core::participant::ParticipantId::new(actor_id))
            .map(|artifact| artifact.value().to_owned()),
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
    use nc_pore_core::session::ProductionSession;

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
    fn reports_active_recording_before_ready_barrier() {
        let session = session_with_recording();
        let state = recording_state(&session, "alice", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Recording);
        assert_eq!(state.role, ClientRecordingRole::Host);
        assert_eq!(state.participants.len(), 2);
        assert!(!state.confirmed);
        assert!(state
            .participants
            .iter()
            .all(|p| p.artifact_id.is_none() && !p.opening_confirmed));
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
        assert_eq!(state.phase, ClientRecordingPhase::Recording);
        assert_eq!(state.participants.iter().filter(|p| p.ready).count(), 1);
        assert_eq!(state.role, ClientRecordingRole::Participant);
    }

    #[test]
    fn reports_stopped_before_completion_from_core_recording() {
        let mut session = session_with_recording();
        let alice = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let recording_id = nc_pore_core::recording::RecordingId::new("recording-001");
        session
            .mark_recording_ready_by(&alice, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&alice, &recording_id).unwrap();
        session.stop_recording_by(&alice, &recording_id).unwrap();

        let state = recording_state(&session, "bob", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Stopped);
        assert!(!state.confirmed);
        assert_eq!(state.artifact_id, None);
        assert!(state.participants.iter().all(|p| p.artifact_id.is_none()));
    }

    #[test]
    fn reports_partial_artifacts_without_completing_recording() {
        let mut session = session_with_recording();
        let alice = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let recording_id = nc_pore_core::recording::RecordingId::new("recording-001");
        session
            .mark_recording_ready_by(&alice, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&alice, &recording_id).unwrap();
        session.stop_recording_by(&alice, &recording_id).unwrap();
        session
            .complete_recording_by(
                &alice,
                &recording_id,
                nc_pore_core::recording::RecordingArtifactId::new("alice-artifact"),
            )
            .unwrap();

        let state = recording_state(&session, "alice", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Stopped);
        assert!(!state.confirmed);
        assert_eq!(state.artifact_id.as_deref(), Some("alice-artifact"));
        assert_eq!(
            state
                .participants
                .iter()
                .find(|p| p.id == "bob")
                .and_then(|p| p.artifact_id.as_deref()),
            None
        );
    }

    #[test]
    fn reports_recording_completed_only_after_last_expected_artifact() {
        let mut session = session_with_recording();
        let alice = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let recording_id = nc_pore_core::recording::RecordingId::new("recording-001");
        session
            .mark_recording_ready_by(&alice, &recording_id)
            .unwrap();
        session
            .mark_recording_ready_by(&bob, &recording_id)
            .unwrap();
        session
            .trigger_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&alice, &recording_id)
            .unwrap();
        session
            .confirm_recording_opening_by(&bob, &recording_id)
            .unwrap();
        session.start_recording_by(&alice, &recording_id).unwrap();
        session.stop_recording_by(&alice, &recording_id).unwrap();

        session
            .complete_recording_by(
                &alice,
                &recording_id,
                nc_pore_core::recording::RecordingArtifactId::new("alice-artifact"),
            )
            .unwrap();
        session
            .complete_recording_by(
                &bob,
                &recording_id,
                nc_pore_core::recording::RecordingArtifactId::new("bob-artifact"),
            )
            .unwrap();

        let state = recording_state(&session, "alice", "recording-001").unwrap();
        assert_eq!(state.phase, ClientRecordingPhase::Completed);
        assert!(state.confirmed);
        assert_eq!(state.artifact_id.as_deref(), Some("alice-artifact"));
        assert_eq!(
            state
                .participants
                .iter()
                .find(|p| p.id == "bob")
                .and_then(|p| p.artifact_id.as_deref()),
            Some("bob-artifact")
        );
    }
}
