use crate::recording_state::{recording_state, ClientRecordingState};
use crate::session::add_recording_to_production_session;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::recording::{Recording, RecordingArtifactId, RecordingId};
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;

/// Host-neutral application orchestration for a recording session.
///
/// Core owns the recording lifecycle and the supplied Application repository
/// owns its persisted session state. The coordinator is the single command
/// boundary between a host adapter and that state. It knows nothing about
/// Nextcloud, Talk, browser capture, or artifact transport.
#[derive(Debug)]
pub struct RecordingCoordinator<'a, R>
where
    R: ProductionSessionRepository,
{
    repository: &'a mut R,
    session_id: ProductionId,
    actor_id: ParticipantId,
    recording_id: RecordingId,
}

impl<'a, R> RecordingCoordinator<'a, R>
where
    R: ProductionSessionRepository,
{
    pub fn new(
        repository: &'a mut R,
        session_id: ProductionId,
        actor_id: ParticipantId,
        recording_id: RecordingId,
    ) -> Self {
        Self {
            repository,
            session_id,
            actor_id,
            recording_id,
        }
    }

    pub fn ensure_recording(&mut self) -> Result<(), ProductionSessionError> {
        add_recording_to_production_session(
            self.repository,
            &self.session_id,
            &self.actor_id,
            Recording::new(self.recording_id.value()),
        )
        .map(|_| ())
        .map_err(|error| match error {
            crate::session::AddRecordingToProductionSessionError::Session(error) => error,
            crate::session::AddRecordingToProductionSessionError::SessionNotFound
            | crate::session::AddRecordingToProductionSessionError::Repository(_) => {
                ProductionSessionError::InvalidStateTransition
            }
        })
    }

    pub fn begin(
        &mut self,
        participants: impl IntoIterator<Item = ParticipantId>,
    ) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        self.mutate(|session| session.begin_recording_by(&actor_id, &recording_id, participants))?;
        self.snapshot()
    }

    pub fn mark_ready(&mut self) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        self.mutate(|session| {
            session
                .mark_recording_ready_by(&actor_id, &recording_id)
                .map(|_| ())
        })?;
        self.snapshot()
    }

    pub fn start(&mut self) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        self.mutate(|session| session.start_recording_by(&actor_id, &recording_id))?;
        self.snapshot()
    }

    pub fn request_stop(&mut self) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        self.mutate(|session| session.stop_recording_by(&actor_id, &recording_id))?;
        self.snapshot()
    }

    pub fn acknowledge_stop(&mut self) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        self.mutate(|session| session.acknowledge_recording_stop_by(&actor_id, &recording_id))?;
        self.snapshot()
    }

    pub fn complete(
        &mut self,
        artifact_id: impl Into<String>,
    ) -> Result<ClientRecordingState, ProductionSessionError> {
        let actor_id = self.actor_id.clone();
        let recording_id = self.recording_id.clone();
        let artifact_id = RecordingArtifactId::new(artifact_id.into());
        self.mutate(|session| {
            session.complete_recording_by(&actor_id, &recording_id, artifact_id)
        })?;
        self.snapshot()
    }

    pub fn snapshot(&self) -> Result<ClientRecordingState, ProductionSessionError> {
        let session = self
            .repository
            .get(&self.session_id)
            .map_err(|_| ProductionSessionError::InvalidStateTransition)?
            .ok_or(ProductionSessionError::InvalidStateTransition)?;

        recording_state(&session, self.actor_id.value(), self.recording_id.value()).map_err(
            |error| match error {
                crate::recording_state::RecordingStateError::RecordingNotFound => {
                    ProductionSessionError::RecordingNotFound
                }
                crate::recording_state::RecordingStateError::RecordingCoordinationNotFound => {
                    ProductionSessionError::RecordingCoordinationNotFound
                }
            },
        )
    }

    fn load_session(
        &self,
    ) -> Result<nc_pore_core::session::ProductionSession, ProductionSessionError> {
        self.repository
            .get(&self.session_id)
            .map_err(|_| ProductionSessionError::InvalidStateTransition)?
            .ok_or(ProductionSessionError::InvalidStateTransition)
    }

    fn mutate<F>(&mut self, mutate: F) -> Result<(), ProductionSessionError>
    where
        F: FnOnce(
            &mut nc_pore_core::session::ProductionSession,
        ) -> Result<(), ProductionSessionError>,
    {
        let mut session = self.load_session()?;
        mutate(&mut session)?;
        self.repository
            .update(&session)
            .map_err(|_| ProductionSessionError::InvalidStateTransition)
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
        InMemoryRepository {
            sessions: vec![session],
        }
    }

    #[test]
    fn coordinator_delegates_the_complete_lifecycle_to_core_and_persisted_session_state() {
        let mut repository = repository();
        {
            let mut coordinator = RecordingCoordinator::new(
                &mut repository,
                ProductionId::new("session-001"),
                ParticipantId::new("alice"),
                RecordingId::new("recording-001"),
            );
            coordinator.ensure_recording().unwrap();
            coordinator
                .begin([ParticipantId::new("alice"), ParticipantId::new("bob")])
                .unwrap();
            coordinator.mark_ready().unwrap();
        }

        {
            let mut bob = RecordingCoordinator::new(
                &mut repository,
                ProductionId::new("session-001"),
                ParticipantId::new("bob"),
                RecordingId::new("recording-001"),
            );
            bob.mark_ready().unwrap();
        }

        let mut coordinator = RecordingCoordinator::new(
            &mut repository,
            ProductionId::new("session-001"),
            ParticipantId::new("alice"),
            RecordingId::new("recording-001"),
        );
        assert_eq!(
            coordinator.snapshot().unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Ready
        );
        coordinator.start().unwrap();
        assert_eq!(
            coordinator.snapshot().unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Recording
        );
        coordinator.request_stop().unwrap();
        assert_eq!(
            coordinator.snapshot().unwrap().phase,
            crate::recording_state::ClientRecordingPhase::Stopped
        );
        coordinator.complete("artifact-001").unwrap();
        let state = coordinator.snapshot().unwrap();
        assert_eq!(
            state.phase,
            crate::recording_state::ClientRecordingPhase::Completed
        );
        assert_eq!(state.artifact_id.as_deref(), Some("artifact-001"));
    }

    #[test]
    fn coordinator_has_no_local_recording_state() {
        let mut repository = repository();
        let mut coordinator = RecordingCoordinator::new(
            &mut repository,
            ProductionId::new("session-001"),
            ParticipantId::new("alice"),
            RecordingId::new("recording-001"),
        );
        coordinator.ensure_recording().unwrap();
        coordinator
            .begin([ParticipantId::new("alice"), ParticipantId::new("bob")])
            .unwrap();

        let first = coordinator.snapshot().unwrap();
        let second = coordinator.snapshot().unwrap();
        assert_eq!(first, second);
    }
}
