use crate::identity::ProductionId;
use crate::session::repository::ProductionSessionRepository;
use crate::session::ProductionSession;

#[derive(Debug, PartialEq, Eq)]
pub enum GetProductionSessionError<E> {
    SessionNotFound,
    Repository(E),
}

pub fn get_production_session<R>(
    repository: &R,
    id: &ProductionId,
) -> Result<ProductionSession, GetProductionSessionError<R::Error>>
where
    R: ProductionSessionRepository,
{
    repository
        .get(id)
        .map_err(GetProductionSessionError::Repository)?
        .ok_or(GetProductionSessionError::SessionNotFound)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::ProductionSession;

    struct InMemory {
        sessions: Vec<ProductionSession>,
    }

    impl ProductionSessionRepository for InMemory {
        type Error = &'static str;

        fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
            self.sessions.push(session.clone());
            Ok(())
        }

        fn get(
            &self,
            id: &ProductionId,
        ) -> Result<Option<ProductionSession>, Self::Error> {
            Ok(self.sessions.iter().find(|session| &session.id == id).cloned())
        }
    }

    // TEST-01
    // Verify: An existing Production Session can be retrieved through the API boundary.
    #[test]
    fn get_production_session_returns_existing_session() {
        let id = ProductionId::new("session-001");
        let session = ProductionSession::new(id.clone());

        let repository = InMemory {
            sessions: vec![session],
        };

        let result = get_production_session(&repository, &id);

        assert!(result.is_ok());
        assert_eq!(result.unwrap().id, id);
    }

    // TEST-02
    // Verify: An unknown Production Session is reported as not found.
    #[test]
    fn get_production_session_reports_missing_session() {
        let repository = InMemory { sessions: vec![] };
        let id = ProductionId::new("unknown");

        let result = get_production_session(&repository, &id);

        assert!(matches!(result, Err(GetProductionSessionError::SessionNotFound)));
    }
}
