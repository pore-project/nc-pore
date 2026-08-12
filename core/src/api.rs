use crate::identity::ProductionId;
use crate::session::ProductionSession;
use crate::session::repository::ProductionSessionRepository;

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
}

#[derive(Debug, PartialEq, Eq)]
pub enum StartProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
    Session(crate::session::ProductionSessionError),
}

pub fn start_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
) -> Result<ProductionSession, StartProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(StartProductionSessionError::Repository)?
        .ok_or(StartProductionSessionError::SessionNotFound)?;

    session
        .start()
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
    Session(crate::session::ProductionSessionError),
}

pub fn complete_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
) -> Result<ProductionSession, CompleteProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(CompleteProductionSessionError::Repository)?
        .ok_or(CompleteProductionSessionError::SessionNotFound)?;

    session
        .complete()
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
    Session(crate::session::ProductionSessionError),
}

pub fn add_participation_to_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    participation: crate::participation::Participation,
) -> Result<ProductionSession, AddParticipationToProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(AddParticipationToProductionSessionError::Repository)?
        .ok_or(AddParticipationToProductionSessionError::SessionNotFound)?;

    session
        .add_participation(participation)
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
}

pub fn add_recording_to_production_session<R>(
    repository: &mut R,
    id: &ProductionId,
    recording: crate::recording::Recording,
) -> Result<ProductionSession, AddRecordingToProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut session = repository
        .get(id)
        .map_err(AddRecordingToProductionSessionError::Repository)?
        .ok_or(AddRecordingToProductionSessionError::SessionNotFound)?;

    session.add_recording(recording);

    repository
        .update(&session)
        .map_err(AddRecordingToProductionSessionError::Repository)?;

    Ok(session)
}

pub fn create_production_session<R>(
    repository: &mut R,
    id: ProductionId,
) -> Result<ProductionSession, CreateProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let session = ProductionSession::new(id);

    repository
        .store(&session)
        .map_err(CreateProductionSessionError::Repository)?;

    Ok(session)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::ProductionSession;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.retain(|s| s.id != session.id);
            self.sessions.push(session.clone());
            Ok(())
        }

        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
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

    // TEST-01
    // Verify: An existing Production Session can be retrieved through the API boundary.
    #[test]
    fn get_production_session_returns_existing_session() {
        let id = ProductionId::new("session-001");
        let session = ProductionSession::new(id.clone());

        let repository = InMemory {
            sessions: vec![session],
        };

        let result = get_production_session(&repository, &id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);
    }

    // TEST-02
    // Verify: An unknown Production Session is reported as not found.
    #[test]
    fn get_production_session_reports_missing_session() {
        let repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        let result = get_production_session(&repository, &id);

        assert!(matches!(
            result,
            Err(GetProductionSessionError::SessionNotFound)
        ));
    }
    // TEST-03
    // Verify: Creating a Production Session stores it in the repository.
    #[test]
    fn create_production_session_stores_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");

        let result = create_production_session(&mut repository, id.clone());

        assert!(result.is_ok());
        assert!(repository.get(&id).unwrap().is_some());
    }

    // TEST-05
    // Verify: Starting a Production Session updates the repository through the API boundary.
    #[test]
    fn start_production_session_updates_session() {
        let id = ProductionId::new("session-001");
        let session = ProductionSession::new(id.clone());

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let result = start_production_session(&mut repository, &id);

        assert!(result.is_ok());
        assert_eq!(
            repository.get(&id).unwrap().unwrap().status(),
            crate::session::ProductionStatus::Active
        );
    }

    // TEST-06
    // Verify: Completing a Production Session updates the repository through the API boundary.
    #[test]
    fn complete_production_session_updates_session() {
        let id = ProductionId::new("session-001");
        let mut session = ProductionSession::new(id.clone());

        session
            .add_participation(crate::participation::Participation {
                participant_id: crate::participant::ParticipantId::new("owner-1"),
                role: crate::role::ParticipantRole::Owner,
            })
            .unwrap();

        session.start().unwrap();

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let result = complete_production_session(&mut repository, &id);

        assert!(result.is_ok());
        assert_eq!(
            repository.get(&id).unwrap().unwrap().status(),
            crate::session::ProductionStatus::Completed
        );
    }

    // TEST-07
    // Verify: Completing an active Production Session without an owner
    // returns the domain error through the API boundary.
    #[test]
    fn complete_production_session_reports_missing_owner() {
        let id = ProductionId::new("session-001");
        let mut session = ProductionSession::new(id.clone());

        session.start().unwrap();

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let result = complete_production_session(&mut repository, &id);

        assert!(matches!(
            result,
            Err(CompleteProductionSessionError::Session(
                crate::session::ProductionSessionError::MissingOwner
            ))
        ));
    }

    // TEST-08
    // Verify: Completing an unknown Production Session is reported as not found.
    #[test]
    fn complete_production_session_reports_missing_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        let result = complete_production_session(&mut repository, &id);

        assert!(matches!(
            result,
            Err(CompleteProductionSessionError::SessionNotFound)
        ));
    }

    // TEST-09
    // Verify: Adding a participation through the API boundary updates the session.
    #[test]
    fn add_participation_to_production_session_updates_session() {
        let id = ProductionId::new("session-001");
        let session = ProductionSession::new(id.clone());

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let participation = crate::participation::Participation {
            participant_id: crate::participant::ParticipantId::new("participant-1"),
            role: crate::role::ParticipantRole::Participant,
        };

        let result = add_participation_to_production_session(&mut repository, &id, participation);

        assert!(result.is_ok());
        assert_eq!(repository.get(&id).unwrap().unwrap().participant_count(), 1);
    }

    // TEST-10
    // Verify: A duplicate participation is reported through the API boundary.
    #[test]
    fn add_participation_to_production_session_reports_duplicate_participant() {
        let id = ProductionId::new("session-001");
        let mut session = ProductionSession::new(id.clone());

        session
            .add_participation(crate::participation::Participation {
                participant_id: crate::participant::ParticipantId::new("participant-1"),
                role: crate::role::ParticipantRole::Participant,
            })
            .unwrap();

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let participation = crate::participation::Participation {
            participant_id: crate::participant::ParticipantId::new("participant-1"),
            role: crate::role::ParticipantRole::Guest,
        };

        let result = add_participation_to_production_session(&mut repository, &id, participation);

        assert!(matches!(
            result,
            Err(AddParticipationToProductionSessionError::Session(
                crate::session::ProductionSessionError::ParticipantAlreadyExists
            ))
        ));
    }

    // TEST-11
    // Verify: Adding a participation to an unknown session is reported as not found.
    #[test]
    fn add_participation_to_production_session_reports_missing_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        let participation = crate::participation::Participation {
            participant_id: crate::participant::ParticipantId::new("participant-1"),
            role: crate::role::ParticipantRole::Participant,
        };

        let result = add_participation_to_production_session(&mut repository, &id, participation);

        assert!(matches!(
            result,
            Err(AddParticipationToProductionSessionError::SessionNotFound)
        ));
    }

    // TEST-04
    // Verify: Repository errors are returned through the API boundary.
    #[test]
    fn create_production_session_reports_repository_error() {
        struct FailingRepository;

        impl ProductionSessionRepository for FailingRepository {
            fn update(&mut self, _session: &ProductionSession) -> Result<(), Self::Error> {
                Err("storage failed")
            }

            type Error = &'static str;

            fn store(&mut self, _session: &ProductionSession) -> Result<(), Self::Error> {
                Err("storage failed")
            }

            fn get(&self, _id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
                Ok(None)
            }
        }

        let mut repository = FailingRepository;
        let id = ProductionId::new("session-001");

        let result = create_production_session(&mut repository, id);

        assert!(matches!(
            result,
            Err(CreateProductionSessionError::Repository("storage failed"))
        ));
    }
    // TEST-12
    // Verify: Adding a recording through the API boundary updates the session.
    #[test]
    fn add_recording_to_production_session_updates_session() {
        let id = ProductionId::new("session-001");
        let session = ProductionSession::new(id.clone());

        let mut repository = InMemory {
            sessions: vec![session],
        };

        let recording = crate::recording::Recording::new("recording-001");

        let result = add_recording_to_production_session(&mut repository, &id, recording);

        assert!(result.is_ok());
        assert_eq!(repository.get(&id).unwrap().unwrap().recordings().len(), 1);
    }

    // TEST-13
    // Verify: Adding a recording to an unknown Production Session is reported as not found.
    #[test]
    fn add_recording_to_production_session_reports_missing_session() {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        let recording = crate::recording::Recording::new("recording-missing");

        let result = add_recording_to_production_session(&mut repository, &id, recording);

        assert!(matches!(
            result,
            Err(AddRecordingToProductionSessionError::SessionNotFound)
        ));
    }
}
