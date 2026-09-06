use crate::client::{ClientSessionError, ClientSessionService};
use crate::session::{
    add_recording_to_production_session, get_production_session,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{Recording, RecordingArtifactId, RecordingId};
use nc_pore_core::session::repository::ProductionSessionRepository;

impl<'a, R> ClientSessionService<'a, R>
where
    R: ProductionSessionRepository,
{
    pub fn add_recording(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
    ) -> Result<(), ClientSessionError<R::Error>> {
        add_recording_to_production_session(
            self.repository_mut(),
            &ProductionId::new(session_id),
            &ParticipantId::new(actor),
            Recording::new(recording_id),
        )
        .map(|_| ())
        .map_err(|error| match error {
            crate::session::AddRecordingToProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::AddRecordingToProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
            crate::session::AddRecordingToProductionSessionError::Session(error) => error.into(),
        })
    }

    pub fn begin_recording(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
        participants: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<(), ClientSessionError<R::Error>> {
        let mut session = get_production_session(
            self.repository_mut(),
            &ProductionId::new(session_id),
        )
        .map_err(|error| match error {
            crate::session::GetProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::GetProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
        })?;

        session
            .begin_recording_by(
                &ParticipantId::new(actor),
                &RecordingId::new(recording_id),
                participants.into_iter().map(|id| ParticipantId::new(id.into())),
            )
            .map_err(Into::into)?;

        self.repository_mut()
            .update(&session)
            .map_err(ClientSessionError::Repository)
    }

    pub fn mark_recording_ready(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
    ) -> Result<(), ClientSessionError<R::Error>> {
        self.mutate_recording(session_id, actor, recording_id, |session, actor, recording_id| {
            session.mark_recording_ready_by(actor, recording_id).map(|_| ())
        })
    }

    pub fn start_recording(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
    ) -> Result<(), ClientSessionError<R::Error>> {
        self.mutate_recording(session_id, actor, recording_id, |session, actor, recording_id| {
            session.start_recording_by(actor, recording_id)
        })
    }

    pub fn stop_recording(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
    ) -> Result<(), ClientSessionError<R::Error>> {
        self.mutate_recording(session_id, actor, recording_id, |session, actor, recording_id| {
            session.stop_recording_by(actor, recording_id)
        })
    }

    pub fn acknowledge_recording_stop(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
    ) -> Result<(), ClientSessionError<R::Error>> {
        self.mutate_recording(session_id, actor, recording_id, |session, actor, recording_id| {
            session.acknowledge_recording_stop_by(actor, recording_id)
        })
    }

    pub fn complete_recording(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
        artifact_id: impl Into<String>,
    ) -> Result<(), ClientSessionError<R::Error>> {
        self.mutate_recording(session_id, actor, recording_id, |session, actor, recording_id| {
            session.complete_recording_by(
                actor,
                recording_id,
                RecordingArtifactId::new(artifact_id.into()),
            )
        })
    }

    fn mutate_recording<F>(
        &mut self,
        session_id: &str,
        actor: &str,
        recording_id: &str,
        mutate: F,
    ) -> Result<(), ClientSessionError<R::Error>>
    where
        F: FnOnce(
            &mut nc_pore_core::session::ProductionSession,
            &ParticipantId,
            &RecordingId,
        ) -> Result<(), nc_pore_core::session::ProductionSessionError>,
    {
        let mut session = get_production_session(
            self.repository_mut(),
            &ProductionId::new(session_id),
        )
        .map_err(|error| match error {
            crate::session::GetProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::GetProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
        })?;

        mutate(
            &mut session,
            &ParticipantId::new(actor),
            &RecordingId::new(recording_id),
        )
        .map_err(Into::into)?;

        self.repository_mut()
            .update(&session)
            .map_err(ClientSessionError::Repository)
    }
}

trait ClientSessionRepositoryAccess<R>
where
    R: ProductionSessionRepository,
{
    fn repository_mut(&mut self) -> &mut R;
}

impl<'a, R> ClientSessionRepositoryAccess<R> for ClientSessionService<'a, R>
where
    R: ProductionSessionRepository,
{
    fn repository_mut(&mut self) -> &mut R {
        self.repository
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::ClientSessionService;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::participant::ParticipantId;
    use nc_pore_core::participation::Participation;
    use nc_pore_core::role::ParticipantRole;
    use nc_pore_core::session::ProductionSession;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            let existing = self.sessions.iter_mut().find(|s| s.id == session.id).ok_or("missing")?;
            *existing = session.clone();
            Ok(())
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self.sessions.iter().find(|s| &s.id == id).cloned())
        }
    }

    #[test]
    fn application_commands_persist_core_recording_lifecycle() {
        let owner = ParticipantId::new("alice");
        let bob = ParticipantId::new("bob");
        let mut session = ProductionSession::new_with_actor(ProductionId::new("session-001"), Some(owner.clone()));
        session.add_participation_by(&owner, Participation::with_roles(owner.clone(), [ParticipantRole::Owner, ParticipantRole::Producer])).unwrap();
        session.add_participation_by(&owner, Participation::new(bob, ParticipantRole::Participant)).unwrap();
        session.start_by(&owner).unwrap();

        let mut repository = InMemory { sessions: vec![session] };
        let mut client = ClientSessionService::new(&mut repository);
        client.add_recording("session-001", "alice", "recording-001").unwrap();
        client.begin_recording("session-001", "alice", "recording-001", ["alice", "bob"]).unwrap();
        client.mark_recording_ready("session-001", "alice", "recording-001").unwrap();
        client.mark_recording_ready("session-001", "bob", "recording-001").unwrap();
        client.start_recording("session-001", "alice", "recording-001").unwrap();
        client.stop_recording("session-001", "alice", "recording-001").unwrap();
        client.complete_recording("session-001", "alice", "recording-001", "artifact-001").unwrap();

        let state = client.recording_state("session-001", "alice", "recording-001").unwrap();
        assert_eq!(state.phase, crate::recording_state::ClientRecordingPhase::Completed);
        assert_eq!(state.artifact_id.as_deref(), Some("artifact-001"));
    }
}
