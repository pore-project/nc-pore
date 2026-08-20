use crate::client::ClientSessionService;
use crate::session_context::{
    SessionCapability, SessionContext, SessionContextProvider, SessionState,
};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::session::repository::ProductionSessionRepository;
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

struct ExternalContextProvider {
    state: SessionState,
    capabilities: Vec<SessionCapability>,
}

impl SessionContextProvider for ExternalContextProvider {
    type Error = &'static str;

    fn resolve(&self, session_id: &str, actor_id: &str) -> Result<SessionContext, Self::Error> {
        if session_id != "external-session-001" {
            return Err("session not found");
        }
        if actor_id != "alice" {
            return Err("actor not found");
        }

        Ok(SessionContext {
            session_id: session_id.to_owned(),
            state: self.state,
            actor_id: actor_id.to_owned(),
            participants: vec![],
            capabilities: self.capabilities.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TEST-01: The existing application client consumes an external provider through the contract.
    #[test]
    fn client_service_resolves_provider_independent_context() {
        let mut repository = InMemory { sessions: vec![] };
        let client = ClientSessionService::new(&mut repository);

        let provider = ExternalContextProvider {
            state: SessionState::Available,
            capabilities: vec![SessionCapability::ParticipateInRecording],
        };

        let context = client
            .context(&provider, "external-session-001", "alice")
            .unwrap();

        assert_eq!(context.session_id, "external-session-001");
        assert_eq!(context.state, SessionState::Available);
        assert_eq!(context.actor_id, "alice");
        assert!(context
            .capabilities
            .contains(&SessionCapability::ParticipateInRecording));
    }

    // TEST-02: The client evaluates the provider-independent capability while the session is active.
    #[test]
    fn client_service_uses_participation_capability() {
        let mut repository = InMemory { sessions: vec![] };
        let client = ClientSessionService::new(&mut repository);

        let provider = ExternalContextProvider {
            state: SessionState::Active,
            capabilities: vec![SessionCapability::ParticipateInRecording],
        };

        assert_eq!(
            client.can_participate(&provider, "external-session-001", "alice"),
            Ok(true)
        );
    }

    // TEST-03: A non-active context cannot be used for recording even if its capability remains present.
    #[test]
    fn client_service_rejects_participation_outside_active_session() {
        let mut repository = InMemory { sessions: vec![] };
        let client = ClientSessionService::new(&mut repository);
        let provider = ExternalContextProvider {
            state: SessionState::Available,
            capabilities: vec![SessionCapability::ParticipateInRecording],
        };

        assert_eq!(
            client.can_participate(&provider, "external-session-001", "alice"),
            Ok(false)
        );

        let completed_provider = ExternalContextProvider {
            state: SessionState::Completed,
            capabilities: vec![SessionCapability::ParticipateInRecording],
        };

        assert_eq!(
            client.can_participate(&completed_provider, "external-session-001", "alice"),
            Ok(false)
        );
    }

    // TEST-04: Provider errors remain provider-owned and are propagated unchanged.
    #[test]
    fn client_service_propagates_provider_errors() {
        let mut repository = InMemory { sessions: vec![] };
        let client = ClientSessionService::new(&mut repository);
        let provider = ExternalContextProvider {
            state: SessionState::Available,
            capabilities: vec![SessionCapability::ParticipateInRecording],
        };

        assert_eq!(
            client.can_participate(&provider, "unknown", "alice"),
            Err("session not found")
        );
        assert_eq!(
            client.can_participate(&provider, "external-session-001", "bob"),
            Err("actor not found")
        );
    }
}
