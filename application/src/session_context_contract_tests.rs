use crate::session_context::{
    SessionCapability, SessionContext, SessionContextProvider, SessionState,
};
use nc_pore_core::identity::ProductionId;

struct ExternalContextProvider;

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
            production_id: ProductionId::new(session_id),
            state: SessionState::Available,
            actor_id: actor_id.to_owned(),
            participants: vec![
                super::session_context::SessionContextParticipant {
                    id: "alice".to_owned(),
                },
                super::session_context::SessionContextParticipant {
                    id: "bob".to_owned(),
                },
                super::session_context::SessionContextParticipant {
                    id: "guest".to_owned(),
                },
            ],
            capabilities: vec![SessionCapability::ParticipateInRecording],
        })
    }
}

struct DummyClient<P> {
    provider: P,
}

impl<P> DummyClient<P>
where
    P: SessionContextProvider,
{
    fn can_participate(&self, session_id: &str, actor_id: &str) -> Result<bool, P::Error> {
        let context = self.provider.resolve(session_id, actor_id)?;

        Ok(context.state != SessionState::Completed
            && context
                .capabilities
                .contains(&SessionCapability::ParticipateInRecording))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_provider_resolves_context_without_pore_roles() {
        let provider = ExternalContextProvider;

        let context = provider
            .resolve("external-session-001", "alice")
            .expect("external provider must resolve the context");

        assert_eq!(context.session_id, "external-session-001");
        assert_eq!(context.production_id.value(), "external-session-001");
        assert_eq!(context.state, SessionState::Available);
        assert_eq!(context.actor_id, "alice");
        assert_eq!(context.participants.len(), 3);
        assert_eq!(
            context.capabilities,
            vec![SessionCapability::ParticipateInRecording]
        );
    }

    #[test]
    fn external_provider_exposes_only_pore_relevant_capabilities() {
        let provider = ExternalContextProvider;

        let context = provider
            .resolve("external-session-001", "alice")
            .expect("external provider must resolve the context");

        assert!(context
            .capabilities
            .contains(&SessionCapability::ParticipateInRecording));
        assert!(!context
            .capabilities
            .contains(&SessionCapability::ManageParticipants));
    }

    #[test]
    fn dummy_client_uses_provider_independent_capability() {
        let client = DummyClient {
            provider: ExternalContextProvider,
        };

        assert_eq!(
            client.can_participate("external-session-001", "alice"),
            Ok(true)
        );
    }

    #[test]
    fn dummy_client_rejects_completed_session() {
        struct CompletedProvider;

        impl SessionContextProvider for CompletedProvider {
            type Error = &'static str;

            fn resolve(
                &self,
                session_id: &str,
                actor_id: &str,
            ) -> Result<SessionContext, Self::Error> {
                Ok(SessionContext {
                    session_id: session_id.to_owned(),
                    production_id: ProductionId::new(session_id),
                    state: SessionState::Completed,
                    actor_id: actor_id.to_owned(),
                    participants: vec![],
                    capabilities: vec![SessionCapability::ParticipateInRecording],
                })
            }
        }

        let client = DummyClient {
            provider: CompletedProvider,
        };

        assert_eq!(client.can_participate("session-001", "alice"), Ok(false));
    }

    #[test]
    fn dummy_client_propagates_provider_errors() {
        let client = DummyClient {
            provider: ExternalContextProvider,
        };

        assert_eq!(
            client.can_participate("unknown", "alice"),
            Err("session not found")
        );
        assert_eq!(
            client.can_participate("external-session-001", "bob"),
            Err("actor not found")
        );
    }
}
