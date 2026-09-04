use crate::client::{ClientSessionError, ClientSessionService};
use crate::recording_state::ClientRecordingState;
use nc_pore_core::session::repository::ProductionSessionRepository;

/// Provider-neutral read boundary for an external recording-state consumer.
///
/// The source exposes the authoritative Application read model. It does not
/// define a transport, serialize data, or maintain a second recording state
/// machine.
pub trait RecordingStateSource {
    type Error;

    fn read_recording_state(
        &self,
        session_id: &str,
        actor_id: &str,
        recording_id: &str,
    ) -> Result<ClientRecordingState, Self::Error>;
}

/// Adapter exposing the Application client facade through the external
/// recording-state source contract.
///
/// A concrete transport (for example an HTTP or WebSocket adapter) can depend
/// on this contract without coupling the transport to Core internals.
impl<'a, R> RecordingStateSource for ClientSessionService<'a, R>
where
    R: ProductionSessionRepository,
{
    type Error = ClientSessionError<R::Error>;

    fn read_recording_state(
        &self,
        session_id: &str,
        actor_id: &str,
        recording_id: &str,
    ) -> Result<ClientRecordingState, Self::Error> {
        self.recording_state(session_id, actor_id, recording_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participant::ParticipantId;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::recording::{Recording, RecordingId};
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

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

    fn repository_with_recording() -> InMemoryRepository {
        let owner = ParticipantId::new("alice");
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
        session.start_by(&owner).unwrap();
        session
            .add_recording_by(&owner, Recording::new("recording-001"))
            .unwrap();
        session
            .begin_recording_by(
                &owner,
                &RecordingId::new("recording-001"),
                [owner.clone()],
            )
            .unwrap();

        InMemoryRepository {
            sessions: vec![session],
        }
    }

    #[test]
    fn TEST_01_source_exposes_authoritative_application_state() {
        let mut repository = repository_with_recording();
        let client = ClientSessionService::new(&mut repository);

        let state = client
            .read_recording_state("session-001", "alice", "recording-001")
            .unwrap();

        assert_eq!(state.recording_id, "recording-001");
        assert_eq!(
            state.phase,
            crate::recording_state::ClientRecordingPhase::Preparing
        );
        assert_eq!(
            state.role,
            crate::recording_state::ClientRecordingRole::Host
        );
        assert_eq!(state.participants.len(), 1);
    }

    #[test]
    fn TEST_02_source_propagates_application_errors_without_reinterpreting_them() {
        let mut repository = repository_with_recording();
        let client = ClientSessionService::new(&mut repository);

        let result = client.read_recording_state(
            "session-001",
            "alice",
            "missing-recording",
        );

        assert_eq!(result, Err(ClientSessionError::RecordingNotFound));
    }
}
