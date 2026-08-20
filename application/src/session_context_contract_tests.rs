use crate::session_context::{
    SessionCapability, SessionContext, SessionContextProvider, SessionState,
};

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

    // TEST-01: A provider with no PoRE role model can satisfy the session context contract.
    #[test]
    fn external_provider_resolves_context_without_pore_roles() {
        let provider = ExternalContextProvider;

        let context = provider
            .resolve("external-session-001", "alice")
            .expect("external provider must resolve the context");

        assert_eq!(context.session_id, "external-session-001");
        assert_eq!(context.state, SessionState::Available);
        assert_eq!(context.actor_id, "alice");
        assert_eq!(context.participants.len(), 3);
        assert_eq!(
            context.capabilities,
            vec![SessionCapability::ParticipateInRecording]
        );
    }

    // TEST-02: The application contract exposes capabilities without exposing provider roles.
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

    // TEST-03: A consumer can use the contract without knowing provider-specific roles.
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

    // TEST-04: A consumer can enforce provider-independent session state semantics.
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

    // TEST-05: Provider errors remain part of the provider boundary and are propagated unchanged.
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
