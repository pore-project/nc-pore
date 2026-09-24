use crate::client::{
    ClientProductionSession, ClientProductionStatus, ClientRole, ClientSessionError,
    ClientSessionService,
};
use nc_pore_core::session::repository::ProductionSessionRepository;

/// Materialize the provider-selected Production identity without activating it.
///
/// Creation is restricted to the provider-selected owner. Once materialized,
/// the owner may reconcile the provider participant list; other participants
/// only observe the existing Production.
pub fn ensure_production<R>(
    repository: &mut R,
    production_id: &str,
    actor_id: &str,
    owner_id: &str,
    participant_ids: &[String],
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    let mut client = ClientSessionService::new(repository);

    match client.get(production_id) {
        Ok(existing) => {
            if actor_id != owner_id {
                return Ok(existing);
            }

            let existing_ids = existing
                .participants
                .iter()
                .map(|participant| participant.id.as_str())
                .collect::<Vec<_>>();
            for participant_id in participant_ids {
                if participant_id != owner_id && !existing_ids.contains(&participant_id.as_str()) {
                    client.add_participant(
                        production_id,
                        actor_id,
                        participant_id,
                        [ClientRole::Participant],
                    )?;
                }
            }

            client.get(production_id)
        }
        Err(ClientSessionError::SessionNotFound) => {
            if actor_id != owner_id {
                return Err(ClientSessionError::Unauthorized);
            }

            client.create(production_id, owner_id)?;
            for participant_id in participant_ids {
                if participant_id != owner_id {
                    client.add_participant(
                        production_id,
                        owner_id,
                        participant_id,
                        [ClientRole::Participant],
                    )?;
                }
            }

            client.get(production_id)
        }
        Err(error) => Err(error),
    }
}

pub fn start_production<R>(
    repository: &mut R,
    production_id: &str,
    actor_id: &str,
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    ClientSessionService::new(repository).start(production_id, actor_id)
}

pub fn force_close_production<R>(
    repository: &mut R,
    production_id: &str,
    actor_id: &str,
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    ClientSessionService::new(repository).force_close(production_id, actor_id)
}

pub fn check_production_timeout<R>(
    repository: &mut R,
    production_id: &str,
    now: std::time::SystemTime,
) -> Result<ClientProductionSession, ClientSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    ClientSessionService::new(repository).check_timeout(production_id, now)
}


#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::identity::ProductionId;
    use nc_pore_core::session::ProductionSession;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            if self
                .sessions
                .iter()
                .any(|existing| existing.id == session.id)
            {
                return Err("session already exists");
            }
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

    // TEST-01: The Host materializes a Production with the recording participants.
    #[test]
    fn TEST_01_host_materializes_production_with_recording_participants() {
        let mut repository = InMemory { sessions: vec![] };

        let production = ensure_production(
            &mut repository,
            "talk-room-001",
            "host-1",
            "host-1",
            &["host-1".to_owned(), "guest-1".to_owned()],
        )
        .unwrap();

        assert_eq!(production.status, ClientProductionStatus::Created);
        assert_eq!(
            production
                .participants
                .iter()
                .map(|participant| (participant.id.as_str(), participant.roles.clone()))
                .collect::<Vec<_>>()
                .len(),
            2
        );
        assert_eq!(production.participants[0].id, "host-1");
        assert_eq!(production.participants[1].id, "guest-1");

        let started = start_production(&mut repository, "talk-room-001", "host-1").unwrap();
        assert_eq!(started.status, ClientProductionStatus::Active);
        assert_eq!(started.participants[1].id, "guest-1");
    }

    // TEST-02: A non-owner cannot materialize a Production before Host start.
    #[test]
    fn TEST_02_non_owner_does_not_create_production_before_host_start() {
        let mut repository = InMemory { sessions: vec![] };

        assert_eq!(
            ensure_production(
                &mut repository,
                "talk-room-002",
                "guest-1",
                "host-1",
                &["host-1".to_owned(), "guest-1".to_owned()],
            ),
            Err(ClientSessionError::Unauthorized)
        );
        assert!(repository.sessions.is_empty());
    }
}
