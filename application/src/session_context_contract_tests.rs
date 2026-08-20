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
}
