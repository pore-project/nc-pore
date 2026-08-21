use crate::session::{
    add_participation_to_production_session, complete_production_session,
    create_production_session, get_production_session, start_production_session,
};
use crate::session_context::{SessionContext, SessionContextProvider};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::RecordingStatus;
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::repository::ProductionSessionRepository;
use nc_pore_core::session::{ProductionSession, ProductionSessionError, ProductionStatus};

/// Stable role vocabulary exposed to a client without leaking the domain role type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRole {
    Owner,
    Producer,
    Participant,
    Guest,
}

impl From<ParticipantRole> for ClientRole {
    fn from(role: ParticipantRole) -> Self {
        match role {
            ParticipantRole::Owner => Self::Owner,
            ParticipantRole::Producer => Self::Producer,
            ParticipantRole::Participant => Self::Participant,
            ParticipantRole::Guest => Self::Guest,
        }
    }
}

impl From<ClientRole> for ParticipantRole {
    fn from(role: ClientRole) -> Self {
        match role {
            ClientRole::Owner => Self::Owner,
            ClientRole::Producer => Self::Producer,
            ClientRole::Participant => Self::Participant,
            ClientRole::Guest => Self::Guest,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientProductionStatus {
    Created,
    Active,
    Completed,
}

impl From<ProductionStatus> for ClientProductionStatus {
    fn from(status: ProductionStatus) -> Self {
        match status {
            ProductionStatus::Created => Self::Created,
            ProductionStatus::Active => Self::Active,
            ProductionStatus::Completed => Self::Completed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientRecordingStatus {
    Prepared,
    Recording,
    Completed,
}

impl From<RecordingStatus> for ClientRecordingStatus {
    fn from(status: RecordingStatus) -> Self {
        match status {
            RecordingStatus::Prepared => Self::Prepared,
            RecordingStatus::Recording => Self::Recording,
            RecordingStatus::Completed => Self::Completed,
        }
    }
}

/// Client-facing participant representation. It deliberately contains no domain behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientParticipant {
    pub id: String,
    pub roles: Vec<ClientRole>,
}

/// Client-facing recording representation. The artifact remains an opaque reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientRecording {
    pub id: String,
    pub status: ClientRecordingStatus,
    pub artifact_id: Option<String>,
}

/// Read model returned across the application/client boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientProductionSession {
    pub id: String,
    pub status: ClientProductionStatus,
    pub participants: Vec<ClientParticipant>,
    pub recordings: Vec<ClientRecording>,
}

impl From<&ProductionSession> for ClientProductionSession {
    fn from(session: &ProductionSession) -> Self {
        Self {
            id: session.id.value().to_owned(),
            status: session.status().into(),
            participants: session
                .participations()
                .iter()
                .map(|participation| ClientParticipant {
                    id: participation.participant_id.value().to_owned(),
                    roles: participation
                        .roles
                        .iter()
                        .copied()
                        .map(ClientRole::from)
                        .collect(),
                })
                .collect(),
            recordings: session
                .recordings()
                .iter()
                .map(|recording| ClientRecording {
                    id: recording.id().value().to_owned(),
                    status: recording.status().into(),
                    artifact_id: recording
                        .artifact_id()
                        .map(|artifact_id| artifact_id.value().to_owned()),
                })
                .collect(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ClientSessionError<E> {
    SessionNotFound,
    Repository(E),
    Unauthorized,
    InvalidStateTransition,
    ParticipantAlreadyExists,
    MissingOwner,
    RecordingNotFound,
}

impl<E> From<ProductionSessionError> for ClientSessionError<E> {
    fn from(error: ProductionSessionError) -> Self {
        match error {
            ProductionSessionError::Unauthorized => Self::Unauthorized,
            ProductionSessionError::InvalidStateTransition => Self::InvalidStateTransition,
            ProductionSessionError::ParticipantAlreadyExists => Self::ParticipantAlreadyExists,
            ProductionSessionError::MissingOwner => Self::MissingOwner,
            ProductionSessionError::RecordingNotFound => Self::RecordingNotFound,
            ProductionSessionError::RecordingLifecycle(_) => Self::InvalidStateTransition,
            ProductionSessionError::RecordingCoordinationNotFound
            | ProductionSessionError::RecordingCoordinationAlreadyActive
            | ProductionSessionError::RecordingCoordination(_) => Self::InvalidStateTransition,
        }
    }
}

/// Minimal application-facing client facade.
///
/// It intentionally stops at the application boundary: transport, HTTP, WebSocket,
/// authentication and serialization are outside this type and can be added by a
/// concrete client without changing Core or the application use cases.
pub struct ClientSessionService<'a, R>
where
    R: ProductionSessionRepository,
{
    repository: &'a mut R,
}

impl<'a, R> ClientSessionService<'a, R>
where
    R: ProductionSessionRepository,
{
    pub fn new(repository: &'a mut R) -> Self {
        Self { repository }
    }

    /// Resolve provider-independent session context through an injected provider.
    ///
    /// The client service depends only on the application contract, so a native or
    /// external provider can be supplied without coupling the client to its role model.
    pub fn context<P>(
        &self,
        provider: &P,
        session_id: &str,
        actor_id: &str,
    ) -> Result<SessionContext, P::Error>
    where
        P: SessionContextProvider,
    {
        provider.resolve(session_id, actor_id)
    }

    /// Ask the injected provider whether the actor may participate in recording now.
    ///
    /// Participation is available only while the session is active; the client still
    /// evaluates the application capability rather than inspecting provider-specific roles.
    pub fn can_participate<P>(
        &self,
        provider: &P,
        session_id: &str,
        actor_id: &str,
    ) -> Result<bool, P::Error>
    where
        P: SessionContextProvider,
    {
        let context = self.context(provider, session_id, actor_id)?;

        Ok(
            context.state == crate::session_context::SessionState::Active
                && context
                    .capabilities
                    .contains(&crate::session_context::SessionCapability::ParticipateInRecording),
        )
    }

    pub fn get(&self, id: &str) -> Result<ClientProductionSession, ClientSessionError<R::Error>> {
        let id = ProductionId::new(id);
        get_production_session(self.repository, &id)
            .map(|session| ClientProductionSession::from(&session))
            .map_err(|error| match error {
                crate::session::GetProductionSessionError::SessionNotFound => {
                    ClientSessionError::SessionNotFound
                }
                crate::session::GetProductionSessionError::Repository(error) => {
                    ClientSessionError::Repository(error)
                }
            })
    }

    pub fn create(
        &mut self,
        id: &str,
        owner: &str,
    ) -> Result<ClientProductionSession, ClientSessionError<R::Error>> {
        create_production_session(
            self.repository,
            ProductionId::new(id),
            ParticipantId::new(owner),
        )
        .map(|session| ClientProductionSession::from(&session))
        .map_err(|error| match error {
            crate::session::CreateProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
            crate::session::CreateProductionSessionError::Session(error) => error.into(),
        })
    }

    pub fn add_participant(
        &mut self,
        session_id: &str,
        actor: &str,
        participant_id: &str,
        roles: impl IntoIterator<Item = ClientRole>,
    ) -> Result<ClientProductionSession, ClientSessionError<R::Error>> {
        let participation = Participation::with_roles(
            ParticipantId::new(participant_id),
            roles.into_iter().map(ParticipantRole::from),
        );

        add_participation_to_production_session(
            self.repository,
            &ProductionId::new(session_id),
            &ParticipantId::new(actor),
            participation,
        )
        .map(|session| ClientProductionSession::from(&session))
        .map_err(|error| match error {
            crate::session::AddParticipationToProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::AddParticipationToProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
            crate::session::AddParticipationToProductionSessionError::Session(error) => {
                error.into()
            }
        })
    }

    pub fn start(
        &mut self,
        session_id: &str,
        actor: &str,
    ) -> Result<ClientProductionSession, ClientSessionError<R::Error>> {
        start_production_session(
            self.repository,
            &ProductionId::new(session_id),
            &ParticipantId::new(actor),
        )
        .map(|session| ClientProductionSession::from(&session))
        .map_err(|error| match error {
            crate::session::StartProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::StartProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
            crate::session::StartProductionSessionError::Session(error) => error.into(),
        })
    }

    pub fn complete(
        &mut self,
        session_id: &str,
        actor: &str,
    ) -> Result<ClientProductionSession, ClientSessionError<R::Error>> {
        complete_production_session(
            self.repository,
            &ProductionId::new(session_id),
            &ParticipantId::new(actor),
        )
        .map(|session| ClientProductionSession::from(&session))
        .map_err(|error| match error {
            crate::session::CompleteProductionSessionError::SessionNotFound => {
                ClientSessionError::SessionNotFound
            }
            crate::session::CompleteProductionSessionError::Repository(error) => {
                ClientSessionError::Repository(error)
            }
            crate::session::CompleteProductionSessionError::Session(error) => error.into(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
                .find(|existing| existing.id == session.id);
            match existing {
                Some(existing) => {
                    *existing = session.clone();
                    Ok(())
                }
                None => Err("session not found"),
            }
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self
                .sessions
                .iter()
                .find(|session| &session.id == id)
                .cloned())
        }
    }

    #[test]
    fn TEST_01_client_can_create_and_read_session() {
        let mut repository = InMemory { sessions: vec![] };
        let mut client = ClientSessionService::new(&mut repository);

        let created = client.create("session-001", "owner-1").unwrap();
        assert_eq!(created.status, ClientProductionStatus::Created);
        assert_eq!(created.participants.len(), 1);
        assert_eq!(created.participants[0].id, "owner-1");
        assert_eq!(created.participants[0].roles, vec![ClientRole::Owner]);

        let read = client.get("session-001").unwrap();
        assert_eq!(read, created);
    }

    #[test]
    fn TEST_02_client_can_add_participant_and_start_session() {
        let mut repository = InMemory { sessions: vec![] };
        let mut client = ClientSessionService::new(&mut repository);

        client.create("session-001", "owner-1").unwrap();
        let updated = client
            .add_participant("session-001", "owner-1", "guest-1", [ClientRole::Guest])
            .unwrap();

        assert_eq!(updated.participants.len(), 2);
        assert_eq!(updated.participants[1].roles, vec![ClientRole::Guest]);

        let started = client.start("session-001", "owner-1").unwrap();
        assert_eq!(started.status, ClientProductionStatus::Active);
    }

    #[test]
    fn TEST_03_client_maps_domain_authorization_errors() {
        let mut repository = InMemory { sessions: vec![] };
        let mut client = ClientSessionService::new(&mut repository);

        client.create("session-001", "owner-1").unwrap();
        client
            .add_participant("session-001", "owner-1", "guest-1", [ClientRole::Guest])
            .unwrap();

        let result = client.start("session-001", "guest-1");
        assert_eq!(result, Err(ClientSessionError::Unauthorized));
    }
}
