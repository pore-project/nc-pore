//! Repository boundary for ProductionSession.
//!
//! The repository defines the domain-facing capability to store and
//! retrieve Production Sessions without depending on a concrete
//! persistence technology.
//!
//! See ADR-036 Persistence Boundary and Storage Strategy.

use crate::activity::ActivityEvent;
use crate::identity::ProductionId;
use crate::participation::Participation;
use crate::recording::{Recording, RecordingCoordination};

use super::{ProductionSession, ProductionStatus};

/// Domain-facing repository contract for Production Sessions.
///
/// The Core defines which persistence capabilities are required.
/// Concrete storage implementations remain outside the domain model.
///
/// See ADR-036.
pub trait ProductionSessionRepository {
    type Error;

    /// Stores a new Production Session.
    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error>;

    /// Updates an existing Production Session.
    ///
    /// The session must already exist.
    fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error>;

    /// Retrieves a Production Session by its Production Identifier.
    ///
    /// `Ok(None)` means that no session with the given identifier exists.
    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error>;
}

/// Reconstitutes a domain session from already-decoded domain state.
///
/// Serialization and storage remain outside Core. This function only restores
/// the aggregate from domain values supplied by an outer persistence adapter.
pub fn reconstitute_production_session(
    id: ProductionId,
    status: ProductionStatus,
    participations: Vec<Participation>,
    recordings: Vec<Recording>,
    recording_coordination: Option<RecordingCoordination>,
    activities: Vec<ActivityEvent>,
) -> ProductionSession {
    ProductionSession {
        id,
        status,
        participations,
        recordings,
        recording_coordination,
        activities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::participant::ParticipantId;
    use crate::role::ParticipantRole;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            if self.sessions.iter().any(|s| s.id == session.id) {
                return Err("session already exists");
            }
            self.sessions.push(session.clone());
            Ok(())
        }

        fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            let existing = self.sessions.iter_mut().find(|s| s.id == session.id);
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
    fn repository_can_store_and_get_session() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();
        assert!(repo.get(&id).unwrap().is_some());
    }

    #[test]
    fn repository_rejects_duplicate_session_id() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();
        let result = repo.store(&ProductionSession::new(id));
        assert_eq!(result, Err("session already exists"));
    }

    #[test]
    fn repository_returns_none_for_unknown_session() {
        let repo = InMemory { sessions: vec![] };
        assert!(repo.get(&ProductionId::new("unknown")).unwrap().is_none());
    }

    #[test]
    fn repository_can_update_existing_session() {
        let mut repo = InMemory { sessions: vec![] };
        let id = ProductionId::new("session-001");
        repo.store(&ProductionSession::new(id.clone())).unwrap();

        let mut updated = ProductionSession::new(id.clone());
        let owner = ParticipantId::new("owner-1");
        updated
            .add_participation_by(
                &owner,
                Participation::new(owner.clone(), ParticipantRole::Owner),
            )
            .unwrap();
        updated.start_by(&owner).unwrap();
        repo.update(&updated).unwrap();
        assert_eq!(repo.get(&id).unwrap().unwrap().status(), updated.status());
    }
}
