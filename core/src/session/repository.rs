//! Repository boundary for ProductionSession.
//!
//! The repository defines the domain-facing capability to store and
//! retrieve Production Sessions without depending on a concrete
//! persistence technology.
//!
//! See:
//! - ADR-036 Persistence Boundary and Storage Strategy
//!

use crate::identity::ProductionId;

use super::ProductionSession;

/// Domain-facing repository contract for Production Sessions.
///
/// The Core defines which persistence capabilities are required.
/// Concrete storage implementations remain outside the domain model.
///
/// See ADR-036.
pub trait ProductionSessionRepository {
    type Error;

    /// Stores a Production Session.
    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error>;

    /// Retrieves a Production Session by its Production Identifier.
    ///
    /// `Ok(None)` means that no session with the given identifier exists.
    /// Technical retrieval failures are represented by the implementation's
    /// associated error type.
    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = ();

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
            Ok(())
        }

        fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self.sessions.iter().find(|s| &s.id == id).cloned())
        }
    }

    #[test]
    fn repository_can_store_and_get_session() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();
        assert!(repo.get(&id).unwrap().is_some());
    }

    #[test]
    fn repository_returns_none_for_unknown_session() {
        let repo = InMemory { sessions: vec![] };
        assert!(repo.get(&ProductionId::new("unknown")).unwrap().is_none());
    }
}
