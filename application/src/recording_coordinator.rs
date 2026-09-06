use crate::recording_state::{recording_state, ClientRecordingState};
use crate::session::{add_participation_to_production_session, add_recording_to_production_session, create_production_session, start_production_session};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::{Recording, RecordingArtifactId, RecordingId};
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::ProductionSessionError;

#[derive(Debug)]
pub struct RecordingCoordinator<'a, R: ProductionSessionRepository> {
    repository: &'a mut R,
    session_id: ProductionId,
    actor_id: ParticipantId,
    recording_id: RecordingId,
}

impl<'a, R: ProductionSessionRepository> RecordingCoordinator<'a, R> {
    pub fn new(repository: &'a mut R, session_id: ProductionId, actor_id: ParticipantId, recording_id: RecordingId) -> Self { Self { repository, session_id, actor_id, recording_id } }

    pub fn ensure_session(&mut self, owner_id: ParticipantId, participants: impl IntoIterator<Item = ParticipantId>) -> Result<(), ProductionSessionError> {
        if self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.is_none() {
            create_production_session(self.repository, self.session_id.clone(), owner_id.clone()).map_err(|error| match error {
                crate::session::CreateProductionSessionError::Session(error) => error,
                crate::session::CreateProductionSessionError::Repository(_) => ProductionSessionError::InvalidStateTransition,
            })?;
        }
        for participant_id in participants {
            let exists = self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.map(|session| session.participations().iter().any(|p| p.participant_id == participant_id)).unwrap_or(false);
            if exists { continue; }
            add_participation_to_production_session(self.repository, &self.session_id, &owner_id, Participation::new(participant_id, ParticipantRole::Participant)).map_err(|error| match error {
                crate::session::AddParticipationToProductionSessionError::Session(error) => error,
                crate::session::AddParticipationToProductionSessionError::SessionNotFound | crate::session::AddParticipationToProductionSessionError::Repository(_) => ProductionSessionError::InvalidStateTransition,
            })?;
        }
        let session = self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.ok_or(ProductionSessionError::InvalidStateTransition)?;
        if session.status() == nc_pore_core::session::ProductionStatus::Created {
            start_production_session(self.repository, &self.session_id, &owner_id).map_err(|error| match error {
                crate::session::StartProductionSessionError::Session(error) => error,
                crate::session::StartProductionSessionError::SessionNotFound | crate::session::StartProductionSessionError::Repository(_) => ProductionSessionError::InvalidStateTransition,
            })?;
        }
        Ok(())
    }

    pub fn ensure_recording(&mut self) -> Result<(), ProductionSessionError> {
        let session = self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.ok_or(ProductionSessionError::InvalidStateTransition)?;
        if session.recordings().iter().any(|recording| recording.id() == &self.recording_id) { return Ok(()); }
        add_recording_to_production_session(self.repository, &self.session_id, &self.actor_id, Recording::new(self.recording_id.value())).map(|_| ()).map_err(|error| match error {
            crate::session::AddRecordingToProductionSessionError::Session(error) => error,
            crate::session::AddRecordingToProductionSessionError::SessionNotFound | crate::session::AddRecordingToProductionSessionError::Repository(_) => ProductionSessionError::InvalidStateTransition,
        })
    }

    pub fn begin(&mut self, participants: impl IntoIterator<Item = ParticipantId>) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); self.mutate(|s| s.begin_recording_by(&actor, &recording, participants))?; self.snapshot() }
    pub fn mark_ready(&mut self) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); self.mutate(|s| s.mark_recording_ready_by(&actor, &recording).map(|_| ()))?; self.snapshot() }
    pub fn start(&mut self) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); self.mutate(|s| s.start_recording_by(&actor, &recording))?; self.snapshot() }
    pub fn request_stop(&mut self) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); self.mutate(|s| s.stop_recording_by(&actor, &recording))?; self.snapshot() }
    pub fn acknowledge_stop(&mut self) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); self.mutate(|s| s.acknowledge_recording_stop_by(&actor, &recording))?; self.snapshot() }
    pub fn complete(&mut self, artifact_id: impl Into<String>) -> Result<ClientRecordingState, ProductionSessionError> { let actor = self.actor_id.clone(); let recording = self.recording_id.clone(); let artifact = RecordingArtifactId::new(artifact_id.into()); self.mutate(|s| s.complete_recording_by(&actor, &recording, artifact))?; self.snapshot() }

    pub fn snapshot(&self) -> Result<ClientRecordingState, ProductionSessionError> {
        let session = self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.ok_or(ProductionSessionError::InvalidStateTransition)?;
        recording_state(&session, self.actor_id.value(), self.recording_id.value()).map_err(|error| match error {
            crate::recording_state::RecordingStateError::RecordingNotFound => ProductionSessionError::RecordingNotFound,
            crate::recording_state::RecordingStateError::RecordingCoordinationNotFound => ProductionSessionError::RecordingCoordinationNotFound,
        })
    }

    fn mutate<F>(&mut self, mutate: F) -> Result<(), ProductionSessionError> where F: FnOnce(&mut nc_pore_core::session::ProductionSession) -> Result<(), ProductionSessionError> {
        let mut session = self.repository.get(&self.session_id).map_err(|_| ProductionSessionError::InvalidStateTransition)?.ok_or(ProductionSessionError::InvalidStateTransition)?;
        mutate(&mut session)?;
        self.repository.update(&session).map_err(|_| ProductionSessionError::InvalidStateTransition)
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

    struct InMemoryRepository { sessions: Vec<ProductionSession> }
    impl ProductionSessionRepository for InMemoryRepository {
        type Error = &'static str;
        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> { self.sessions.push(session.clone()); Ok(()) }
        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> { let existing = self.sessions.iter_mut().find(|s| s.id == session.id).ok_or("session not found")?; *existing = session.clone(); Ok(()) }
        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> { Ok(self.sessions.iter().find(|s| &s.id == id).cloned()) }
    }
    fn repository() -> InMemoryRepository {
        let owner = ParticipantId::new("alice");
        let mut session = ProductionSession::new_with_actor(ProductionId::new("session-001"), Some(owner.clone()));
        session.add_participation_by(&owner, Participation::with_roles(owner.clone(), [ParticipantRole::Owner, ParticipantRole::Producer])).unwrap();
        session.add_participation_by(&owner, Participation::new(ParticipantId::new("bob"), ParticipantRole::Participant)).unwrap();
        session.start_by(&owner).unwrap();
        InMemoryRepository { sessions: vec![session] }
    }
    #[test]
    fn coordinator_can_establish_session_and_participants() {
        let mut repository = InMemoryRepository { sessions: vec![] };
        let mut coordinator = RecordingCoordinator::new(&mut repository, ProductionId::new("session-001"), ParticipantId::new("alice"), RecordingId::new("recording-001"));
        coordinator.ensure_session(ParticipantId::new("alice"), [ParticipantId::new("alice"), ParticipantId::new("bob")]).unwrap();
        assert_eq!(repository.sessions[0].participant_count(), 2);
        assert_eq!(repository.sessions[0].status(), nc_pore_core::session::ProductionStatus::Active);
    }
    #[test]
    fn coordinator_delegates_the_complete_lifecycle_to_core_and_persisted_session_state() {
        let mut repository = repository();
        {
            let mut coordinator = RecordingCoordinator::new(&mut repository, ProductionId::new("session-001"), ParticipantId::new("alice"), RecordingId::new("recording-001"));
            coordinator.ensure_recording().unwrap();
            coordinator.begin([ParticipantId::new("alice"), ParticipantId::new("bob")]).unwrap();
            coordinator.mark_ready().unwrap();
        }
        {
            let mut bob = RecordingCoordinator::new(&mut repository, ProductionId::new("session-001"), ParticipantId::new("bob"), RecordingId::new("recording-001"));
            bob.mark_ready().unwrap();
        }
        let mut coordinator = RecordingCoordinator::new(&mut repository, ProductionId::new("session-001"), ParticipantId::new("alice"), RecordingId::new("recording-001"));
        assert_eq!(coordinator.snapshot().unwrap().phase, crate::recording_state::ClientRecordingPhase::Ready);
        coordinator.start().unwrap();
        coordinator.request_stop().unwrap();
        coordinator.complete("artifact-001").unwrap();
        assert_eq!(coordinator.snapshot().unwrap().phase, crate::recording_state::ClientRecordingPhase::Completed);
    }
    #[test]
    fn coordinator_has_no_local_recording_state() {
        let mut repository = repository();
        let mut coordinator = RecordingCoordinator::new(&mut repository, ProductionId::new("session-001"), ParticipantId::new("alice"), RecordingId::new("recording-001"));
        coordinator.ensure_recording().unwrap();
        coordinator.begin([ParticipantId::new("alice"), ParticipantId::new("bob")]).unwrap();
        assert_eq!(coordinator.snapshot().unwrap(), coordinator.snapshot().unwrap());
    }
}
