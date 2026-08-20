use crate::session_context::{
    SessionCapability, SessionContext, SessionContextParticipant, SessionContextProvider,
    SessionState,
};

/// Provider-neutral data returned by an external session integration.
///
/// An actual provider adapter (for example a future Nextcloud Talk integration)
/// maps its own session model to this small boundary type before PoRE sees it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSessionSnapshot {
    pub session_id: String,
    pub state: ExternalSessionState,
    pub actor_id: String,
    pub participants: Vec<String>,
    pub can_participate_in_recording: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalSessionState {
    Available,
    Active,
    Completed,
}

/// Source boundary for an external session provider.
///
/// This trait deliberately knows nothing about PoRE roles, Core aggregates,
/// repositories, or transport details. A concrete integration implements it.
pub trait ExternalSessionContextSource {
    type Error;

    fn resolve(
        &self,
        session_id: &str,
        actor_id: &str,
    ) -> Result<ExternalSessionSnapshot, Self::Error>;
}

/// Adapts an external session source to PoRE's SessionContextProvider contract.
pub struct ExternalSessionContextAdapter<S> {
    source: S,
}

impl<S> ExternalSessionContextAdapter<S> {
    pub fn new(source: S) -> Self {
        Self { source }
    }
}

impl<S> SessionContextProvider for ExternalSessionContextAdapter<S>
where
    S: ExternalSessionContextSource,
{
    type Error = S::Error;

    fn resolve(&self, session_id: &str, actor_id: &str) -> Result<SessionContext, Self::Error> {
        let snapshot = self.source.resolve(session_id, actor_id)?;

        let capabilities = snapshot
            .can_participate_in_recording
            .then_some(SessionCapability::ParticipateInRecording)
            .into_iter()
            .collect();

        Ok(SessionContext {
            session_id: snapshot.session_id,
            state: match snapshot.state {
                ExternalSessionState::Available => SessionState::Available,
                ExternalSessionState::Active => SessionState::Active,
                ExternalSessionState::Completed => SessionState::Completed,
            },
            actor_id: snapshot.actor_id,
            participants: snapshot
                .participants
                .into_iter()
                .map(|id| SessionContextParticipant { id })
                .collect(),
            capabilities,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fixture {
        snapshot: ExternalSessionSnapshot,
    }

    impl ExternalSessionContextSource for Fixture {
        type Error = &'static str;

        fn resolve(
            &self,
            session_id: &str,
            actor_id: &str,
        ) -> Result<ExternalSessionSnapshot, Self::Error> {
            if session_id != self.snapshot.session_id {
                return Err("session not found");
            }
            if actor_id != self.snapshot.actor_id {
                return Err("actor not found");
            }
            Ok(self.snapshot.clone())
        }
    }

    fn adapter(snapshot: ExternalSessionSnapshot) -> ExternalSessionContextAdapter<Fixture> {
        ExternalSessionContextAdapter::new(Fixture { snapshot })
    }

    #[test]
    fn external_adapter_maps_provider_state_and_participants() {
        let provider = adapter(ExternalSessionSnapshot {
            session_id: "external-001".to_owned(),
            state: ExternalSessionState::Active,
            actor_id: "alice".to_owned(),
            participants: vec!["alice".to_owned(), "bob".to_owned()],
            can_participate_in_recording: true,
        });

        let context = provider.resolve("external-001", "alice").unwrap();

        assert_eq!(context.session_id, "external-001");
        assert_eq!(context.state, SessionState::Active);
        assert_eq!(context.actor_id, "alice");
        assert_eq!(
            context.participants,
            vec![
                SessionContextParticipant {
                    id: "alice".to_owned()
                },
                SessionContextParticipant {
                    id: "bob".to_owned()
                }
            ]
        );
    }

    #[test]
    fn external_adapter_exposes_only_supported_pore_capability() {
        let provider = adapter(ExternalSessionSnapshot {
            session_id: "external-001".to_owned(),
            state: ExternalSessionState::Active,
            actor_id: "alice".to_owned(),
            participants: vec!["alice".to_owned()],
            can_participate_in_recording: true,
        });

        let context = provider.resolve("external-001", "alice").unwrap();

        assert_eq!(
            context.capabilities,
            vec![SessionCapability::ParticipateInRecording]
        );
    }

    #[test]
    fn external_adapter_does_not_invent_capabilities() {
        let provider = adapter(ExternalSessionSnapshot {
            session_id: "external-001".to_owned(),
            state: ExternalSessionState::Active,
            actor_id: "alice".to_owned(),
            participants: vec!["alice".to_owned()],
            can_participate_in_recording: false,
        });

        let context = provider.resolve("external-001", "alice").unwrap();

        assert!(context.capabilities.is_empty());
    }

    #[test]
    fn external_adapter_propagates_source_errors() {
        let provider = adapter(ExternalSessionSnapshot {
            session_id: "external-001".to_owned(),
            state: ExternalSessionState::Available,
            actor_id: "alice".to_owned(),
            participants: vec![],
            can_participate_in_recording: false,
        });

        assert_eq!(
            provider.resolve("unknown", "alice"),
            Err("session not found")
        );
        assert_eq!(
            provider.resolve("external-001", "bob"),
            Err("actor not found")
        );
    }
}
