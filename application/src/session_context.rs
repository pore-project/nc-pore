use crate::session::get_production_session;
use nc_pore_core::identity::ProductionId;
use nc_pore_core::role::ProductionAction;
use nc_pore_core::session::repository::ProductionSessionRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Available,
    Active,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCapability {
    StartSession,
    CompleteSession,
    ManageParticipants,
    ManageRecordings,
    ParticipateInRecording,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContextParticipant {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    pub session_id: String,
    pub production_id: ProductionId,
    pub state: SessionState,
    pub actor_id: String,
    pub participants: Vec<SessionContextParticipant>,
    pub capabilities: Vec<SessionCapability>,
}

pub trait SessionContextProvider {
    type Error;

    fn resolve(&self, session_id: &str, actor_id: &str) -> Result<SessionContext, Self::Error>;
}

pub struct ProductionSessionContextProvider<'a, R>
where
    R: ProductionSessionRepository,
{
    repository: &'a R,
}

impl<'a, R> ProductionSessionContextProvider<'a, R>
where
    R: ProductionSessionRepository,
{
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProductionSessionContextError<E> {
    SessionNotFound,
    ActorNotFound,
    Repository(E),
}

impl<'a, R> SessionContextProvider for ProductionSessionContextProvider<'a, R>
where
    R: ProductionSessionRepository,
{
    type Error = ProductionSessionContextError<R::Error>;

    fn resolve(&self, session_id: &str, actor_id: &str) -> Result<SessionContext, Self::Error> {
        let session = get_production_session(self.repository, &ProductionId::new(session_id))
            .map_err(|error| match error {
                crate::session::GetProductionSessionError::SessionNotFound => {
                    ProductionSessionContextError::SessionNotFound
                }
                crate::session::GetProductionSessionError::Repository(error) => {
                    ProductionSessionContextError::Repository(error)
                }
            })?;

        let actor = session
            .participations()
            .iter()
            .find(|participation| participation.participant_id.value() == actor_id)
            .ok_or(ProductionSessionContextError::ActorNotFound)?;

        let capabilities = [
            (
                ProductionAction::StartSession,
                SessionCapability::StartSession,
            ),
            (
                ProductionAction::CompleteSession,
                SessionCapability::CompleteSession,
            ),
            (
                ProductionAction::ManageParticipants,
                SessionCapability::ManageParticipants,
            ),
            (
                ProductionAction::ManageRecordings,
                SessionCapability::ManageRecordings,
            ),
            (
                ProductionAction::ParticipateInRecording,
                SessionCapability::ParticipateInRecording,
            ),
        ]
        .into_iter()
        .filter(|(action, _)| actor.roles.iter().copied().any(|role| role.allows(*action)))
        .map(|(_, capability)| capability)
        .collect();

        Ok(SessionContext {
            session_id: session.id.value().to_owned(),
            production_id: session.id.clone(),
            state: match session.status() {
                nc_pore_core::session::ProductionStatus::Completed => SessionState::Completed,
                nc_pore_core::session::ProductionStatus::Active => SessionState::Active,
                _ => SessionState::Available,
            },
            actor_id: actor.participant_id.value().to_owned(),
            participants: session
                .participations()
                .iter()
                .map(|participation| SessionContextParticipant {
                    id: participation.participant_id.value().to_owned(),
                })
                .collect(),
            capabilities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn session_with_owner() -> (InMemory, ProductionId) {
        let mut repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        let owner = ParticipantId::new("owner-1");
        let mut session = ProductionSession::new_with_actor(id.clone(), Some(owner.clone()));
        session
            .add_participation_by(
                &owner,
                Participation::new(owner.clone(), ParticipantRole::Owner),
            )
            .unwrap();
        repository.store(&session).unwrap();
        (repository, id)
    }

    #[test]
    fn native_provider_resolves_context() {
        let (repository, id) = session_with_owner();
        let provider = ProductionSessionContextProvider::new(&repository);

        let context = provider.resolve(id.value(), "owner-1").unwrap();

        assert_eq!(context.session_id, "session-001");
        assert_eq!(context.production_id, id);
        assert_eq!(context.state, SessionState::Available);
        assert_eq!(context.actor_id, "owner-1");
        assert_eq!(context.participants.len(), 1);
        assert!(context
            .capabilities
            .contains(&SessionCapability::StartSession));
        assert!(context
            .capabilities
            .contains(&SessionCapability::ManageRecordings));
    }

    #[test]
    fn native_provider_reports_completed_session_state() {
        let (mut repository, id) = session_with_owner();
        let owner = ParticipantId::new("owner-1");
        let mut session = repository.get(&id).unwrap().unwrap();
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();
        repository.update(&session).unwrap();

        let provider = ProductionSessionContextProvider::new(&repository);
        let context = provider.resolve(id.value(), "owner-1").unwrap();

        assert_eq!(context.state, SessionState::Completed);
        assert_eq!(context.production_id, id);
    }

    #[test]
    fn native_provider_reports_missing_session_and_actor() {
        let (repository, id) = session_with_owner();
        let provider = ProductionSessionContextProvider::new(&repository);

        assert_eq!(
            provider.resolve("unknown", "owner-1"),
            Err(ProductionSessionContextError::SessionNotFound)
        );
        assert_eq!(
            provider.resolve(id.value(), "unknown"),
            Err(ProductionSessionContextError::ActorNotFound)
        );
    }
}
