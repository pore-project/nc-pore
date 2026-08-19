use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use crate::participation::Participation;
use crate::recording::Recording;
use crate::session::repository::ProductionSessionRepository;
use crate::session::{ProductionSession, ProductionSessionError};

#[derive(Debug, PartialEq, Eq)]
pub enum GetProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
}

pub fn get_production_session<R>(
    repository: &R,
    id: &ProductionId,
) -> Result<ProductionSession, GetProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    repository
        .get(id)
        .map_err(GetProductionSessionError::Repository)?
        .ok_or(GetProductionSessionError::SessionNotFound)
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateProductionSessionError<E> {
    Repository(E),
    Session(ProductionSessionError),
}

pub fn create_production_session<R>(
    repository: &mut R,
    id: ProductionId,
    owner: ParticipantId,
) -> Result<ProductionSession, CreateProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = ProductionSession::new(id);
    let participation = Participation::new(owner.clone(), crate::role::ParticipantRole::Owner);

    session
        .add_participation_by(&owner, participation)
        .map_err(CreateProductionSessionError::Session)?;

    repository
        .store(&session)
        .map_err(CreateProductionSessionError::Repository)?;

    Ok(session)
}

#[derive(Debug, PartialEq, Eq)]
pub enum StartProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
    Session(ProductionSessionError),
}

pub fn start_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    actor: &ParticipantId,
) -> Result<ProductionSession, StartProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(StartProductionSessionError::Repository)?
        .ok_or(StartProductionSessionError::SessionNotFound)?;

    session
        .start_by(actor)
        .map_err(StartProductionSessionError::Session)?;

    repository
        .update(&session)
        .map_err(StartProductionSessionError::Repository)?;

    Ok(session)
}

#[derive(Debug, PartialEq, Eq)]
pub enum CompleteProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
    Session(ProductionSessionError),
}

pub fn complete_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    actor: &ParticipantId,
) -> Result<ProductionSession, CompleteProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(CompleteProductionSessionError::Repository)?
        .ok_or(CompleteProductionSessionError::SessionNotFound)?;

    session
        .complete_by(actor)
        .map_err(CompleteProductionSessionError::Session)?;

    repository
        .update(&session)
        .map_err(CompleteProductionSessionError::Repository)?;

    Ok(session)
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddParticipationToProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
    Session(ProductionSessionError),
}

pub fn add_participation_to_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    actor: &ParticipantId,
    participation: Participation,
) -> Result<ProductionSession, AddParticipationToProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(AddParticipationToProductionSessionError::Repository)?
        .ok_or(AddParticipationToProductionSessionError::SessionNotFound)?;

    session
        .add_participation_by(actor, participation)
        .map_err(AddParticipationToProductionSessionError::Session)?;

    repository
        .update(&session)
        .map_err(AddParticipationToProductionSessionError::Repository)?;

    Ok(session)
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddRecordingToProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
    Session(ProductionSessionError),
}

pub fn add_recording_to_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    actor: &ParticipantId,
    recording: Recording,
) -> Result<ProductionSession, AddRecordingToProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(AddRecordingToProductionSessionError::Repository)?
        .ok_or(AddRecordingToProductionSessionError::SessionNotFound)?;

    session
        .add_recording_by(actor, recording)
        .map_err(AddRecordingToProductionSessionError::Session)?;

    repository
        .update(&session)
        .map_err(AddRecordingToProductionSessionError::Repository)?;

    Ok(session)
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListParticipantsError<E> {
    SessionNotFound,
    Repository(E),
}

pub fn list_participants<R>(
    repository: &R,
    id: &ProductionId,
) -> Result<Vec<Participation>, ListParticipantsError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let session = repository
        .get(id)
        .map_err(ListParticipantsError::Repository)?
        .ok_or(ListParticipantsError::SessionNotFound)?;

    Ok(session.participations().to_vec())
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListRecordingsError<E> {
    SessionNotFound,
    Repository(E),
}

pub fn list_recordings<R>(
    repository: &R,
    id: &ProductionId,
) -> Result<Vec<Recording>, ListRecordingsError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let session = repository
        .get(id)
        .map_err(ListRecordingsError::Repository)?
        .ok_or(ListRecordingsError::SessionNotFound)?;

    Ok(session.recordings().to_vec())
}

#[derive(Debug, PartialEq, Eq)]
pub enum ListActivityHistoryError<E> {
    SessionNotFound,
    Repository(E),
}

pub fn list_activity_history<R>(
    repository: &R,
    id: &ProductionId,
) -> Result<Vec<crate::activity::ActivityEvent>, ListActivityHistoryError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let session = repository
        .get(id)
        .map_err(ListActivityHistoryError::Repository)?
        .ok_or(ListActivityHistoryError::SessionNotFound)?;

    Ok(session.activities().to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role::ParticipantRole;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            if self.sessions.iter().any(|s| s.id == session.id) {
                return Err("session already exists");
            }
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            let existing = self.sessions.iter_mut().find(|s| s.id == session.id);
            match existing {
                Some(existing) => {
                    *existing = session.clone();
                    Ok(())
                }
                None => Err("session not found"),
            }
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self.sessions.iter().find(|session| &session.id == id).cloned())
        }
    }

    fn owner() -> ParticipantId {
        ParticipantId::new("owner-1")
    }

    #[test]
    fn create_production_session_establishes_owner() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");

        let result = create_production_session(&mut repository, id.clone(), owner());

        assert!(result.is_ok());
        let session = repository.get(&id).unwrap().unwrap();
        assert!(session.has_owner());
        assert_eq!(session.participant_count(), 1);
    }

    #[test]
    fn start_production_session_requires_authorized_actor() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        create_production_session(&mut repository, id.clone(), owner()).unwrap();

        let unauthorized = ParticipantId::new("participant-1");
        let result = start_production_session(&mut repository, &id, &unauthorized);

        assert!(matches!(
            result,
            Err(StartProductionSessionError::Session(
                ProductionSessionError::Unauthorized
            ))
        ));
    }

    #[test]
    fn authorized_actor_can_start_and_complete_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        let actor = owner();
        create_production_session(&mut repository, id.clone(), actor.clone()).unwrap();

        start_production_session(&mut repository, &id, &actor).unwrap();
        complete_production_session(&mut repository, &id, &actor).unwrap();

        assert_eq!(
            repository.get(&id).unwrap().unwrap().status(),
            crate::session::ProductionStatus::Completed
        );
    }

    #[test]
    fn producer_can_manage_participants() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        let actor = owner();
        create_production_session(&mut repository, id.clone(), actor.clone()).unwrap();

        let producer = ParticipantId::new("producer-1");
        add_participation_to_production_session(
            &mut repository,
            &id,
            &actor,
            Participation::with_roles(producer.clone(), [ParticipantRole::Producer]),
        )
        .unwrap();

        let participant = Participation::new(
            ParticipantId::new("participant-1"),
            ParticipantRole::Participant,
        );
        add_participation_to_production_session(&mut repository, &id, &producer, participant)
            .unwrap();
    }

    #[test]
    fn participant_cannot_manage_participants_or_recordings() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        let actor = owner();
        create_production_session(&mut repository, id.clone(), actor.clone()).unwrap();

        let participant = ParticipantId::new("participant-1");
        add_participation_to_production_session(
            &mut repository,
            &id,
            &actor,
            Participation::new(participant.clone(), ParticipantRole::Participant),
        )
        .unwrap();

        let result = add_recording_to_production_session(
            &mut repository,
            &id,
            &participant,
            Recording::new("recording-001"),
        );

        assert!(matches!(
            result,
            Err(AddRecordingToProductionSessionError::Session(
                ProductionSessionError::Unauthorized
            ))
        ));
    }

    #[test]
    fn activity_history_exposes_actor_and_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        let actor = owner();
        create_production_session(&mut repository, id.clone(), actor.clone()).unwrap();
        start_production_session(&mut repository, &id, &actor).unwrap();

        let history = list_activity_history(&repository, &id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].actor, Some(actor));
        assert_eq!(history[1].session_id, id);
    }

    #[test]
    fn list_operations_report_missing_sessions() {
        let repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        assert!(matches!(
            list_participants(&repository, &id),
            Err(ListParticipantsError::SessionNotFound)
        ));
        assert!(matches!(
            list_recordings(&repository, &id),
            Err(ListRecordingsError::SessionNotFound)
        ));
        assert!(matches!(
            list_activity_history(&repository, &id),
            Err(ListActivityHistoryError::SessionNotFound)
        ));
    }
}
