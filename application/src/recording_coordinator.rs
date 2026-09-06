use crate::client::{ClientSessionError, ClientSessionService};
use nc_pore_core::recording::{Recording, RecordingId};
use nc_pore_core::session::repository::ProductionSessionRepository;

/// Host-neutral application orchestration for a recording session.
///
/// The coordinator is deliberately stateless with regard to the recording
/// lifecycle. Core owns the lifecycle and the Application session repository
/// is the authoritative persisted state. A host adapter supplies the actor and
/// participant identities; this type knows nothing about Nextcloud, Talk,
/// browser capture, or artifact transport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordingCoordinator {
    session_id: String,
    actor_id: String,
    recording_id: String,
}

impl RecordingCoordinator {
    pub fn new(session_id: impl Into<String>, actor_id: impl Into<String>, recording_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            actor_id: actor_id.into(),
            recording_id: recording_id.into(),
        }
    }

    pub fn ensure_recording<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
    ) -> Result<(), ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.add_recording(
            &self.session_id,
            &self.actor_id,
            &self.recording_id,
        )
    }

    pub fn begin<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
        participants: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.begin_recording(
            &self.session_id,
            &self.actor_id,
            &self.recording_id,
            participants,
        )?;
        self.snapshot(client)
    }

    pub fn mark_ready<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.mark_recording_ready(&self.session_id, &self.actor_id, &self.recording_id)?;
        self.snapshot(client)
    }

    pub fn start<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.start_recording(&self.session_id, &self.actor_id, &self.recording_id)?;
        self.snapshot(client)
    }

    pub fn request_stop<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.stop_recording(&self.session_id, &self.actor_id, &self.recording_id)?;
        self.snapshot(client)
    }

    pub fn acknowledge_stop<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.acknowledge_recording_stop(
            &self.session_id,
            &self.actor_id,
            &self.recording_id,
        )?;
        self.snapshot(client)
    }

    pub fn complete<R>(
        &self,
        client: &mut ClientSessionService<'_, R>,
        artifact_id: impl Into<String>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.complete_recording(
            &self.session_id,
            &self.actor_id,
            &self.recording_id,
            artifact_id,
        )?;
        self.snapshot(client)
    }

    pub fn snapshot<R>(
        &self,
        client: &ClientSessionService<'_, R>,
    ) -> Result<crate::recording_state::ClientRecordingState, ClientSessionError<R::Error>>
    where
        R: ProductionSessionRepository,
    {
        client.recording_state(&self.session_id, &self.actor_id, &self.recording_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participant::ParticipantId;
    use nc_pore_core::participation::Participation;
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

    fn repository() -> InMemoryRepository {
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
        InMemoryRepository { sessions: vec![session] }
    }

    #[test]
    fn coordinator_delegates_the_complete_lifecycle_to_application_and_core() {
        let mut repository = repository();
        let mut client = ClientSessionService::new(&mut repository);
        let coordinator = RecordingCoordinator::new("session-001", "alice", "recording-001");

        coordinator.ensure_recording(&mut client).unwrap();
        coordinator
            .begin(&mut client, ["alice", "bob"])
            .unwrap();

        assert_eq!(
            coordinator.snapshot(&client).unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Preparing
        );

        coordinator.mark_ready(&mut client).unwrap();
        let bob = RecordingCoordinator::new("session-001", "bob", "recording-001");
        bob.mark_ready(&mut client).unwrap();

        assert_eq!(
            coordinator.snapshot(&client).unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Ready
        );

        coordinator.start(&mut client).unwrap();
        assert_eq!(
            coordinator.snapshot(&client).unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Recording
        );

        coordinator.request_stop(&mut client).unwrap();
        assert_eq!(
            coordinator.snapshot(&client).unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Stopped
        );

        coordinator
            .complete(&mut client, "artifact-001")
            .unwrap();
        let state = coordinator.snapshot(&client).unwrap();
        assert_eq!(state.phase, crate::recording_state::ClientRecordingPhase::Completed);
        assert_eq!(state.artifact_id.as_deref(), Some("artifact-001"));
    }

    #[test]
    fn coordinator_does_not_keep_a_second_recording_state_machine() {
        let mut repository = repository();
        let mut client = ClientSessionService::new(&mut repository);
        let coordinator = RecordingCoordinator::new("session-001", "alice", "recording-001");

        coordinator.ensure_recording(&mut client).unwrap();
        coordinator.begin(&mut client, ["alice", "bob"]).unwrap();

        let first = coordinator.snapshot(&client).unwrap();
        let second = coordinator.snapshot(&client).unwrap();
        assert_eq!(first, second);
    }
}
