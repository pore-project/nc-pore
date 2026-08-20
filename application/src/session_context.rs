use crate::session::get_production_session;
use nc_pore_core::role::{ParticipantRole, ProductionAction};
use nc_pore_core::session::repository::ProductionSessionRepository;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionCapability {
    StartSession,
    CompleteSession,
    ManageParticipants,
    ManageRecordings,
    ParticipateInRecording,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextParticipantRole {
    Owner,
    Producer,
    Participant,
    Guest,
}

impl From<ParticipantRole> for ContextParticipantRole {
    fn from(role: ParticipantRole) -> Self {
        match role {
            ParticipantRole::Owner => Self::Owner,
            ParticipantRole::Producer => Self::Producer,
            ParticipantRole::Participant => Self::Participant,
            ParticipantRole::Guest => Self::Guest,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContextParticipant {
    pub id: String,
    pub roles: Vec<ContextParticipantRole>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionContext {
    pub session_id: String,
    pub availability: SessionAvailability,
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
        let session = get_production_session(
            self.repository,
            &nc_pore_core::identity::ProductionId::new(session_id),
        )
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
            availability: if matches!(
                session.status(),
                nc_pore_core::session::ProductionStatus::Completed
            ) {
                SessionAvailability::Unavailable
            } else {
                SessionAvailability::Available
            },
            actor_id: actor.participant_id.value().to_owned(),
            participants: session
                .participations()
                .iter()
                .map(|participation| SessionContextParticipant {
                    id: participation.participant_id.value().to_owned(),
                    roles: participation
                        .roles
                        .iter()
                        .copied()
                        .map(ContextParticipantRole::from)
                        .collect(),
                })
                .collect(),
            capabilities,
        })
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

    // TEST-01: The native provider resolves the complete application context.
    #[test]
    fn native_provider_resolves_context() {
        let (repository, id) = session_with_owner();
        let provider = ProductionSessionContextProvider::new(&repository);

        let context = provider.resolve(id.value(), "owner-1").unwrap();

        assert_eq!(context.session_id, "session-001");
        assert_eq!(context.availability, SessionAvailability::Available);
        assert_eq!(context.actor_id, "owner-1");
        assert_eq!(context.participants.len(), 1);
        assert!(context
            .capabilities
            .contains(&SessionCapability::StartSession));
        assert!(context
            .capabilities
            .contains(&SessionCapability::ManageRecordings));
    }

    // TEST-02: Provider availability is part of the context, not inferred by the caller.
    #[test]
    fn native_provider_reports_completed_session_as_unavailable() {
        let (mut repository, id) = session_with_owner();
        let owner = ParticipantId::new("owner-1");
        let mut session = repository.get(&id).unwrap().unwrap();
        session.start_by(&owner).unwrap();
        session.complete_by(&owner).unwrap();
        repository.update(&session).unwrap();

        let provider = ProductionSessionContextProvider::new(&repository);
        let context = provider.resolve(id.value(), "owner-1").unwrap();

        assert_eq!(context.availability, SessionAvailability::Unavailable);
    }

    // TEST-03: Provider and actor failures remain explicit application errors.
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
