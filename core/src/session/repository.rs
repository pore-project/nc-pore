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
use crate::recording::Recording;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

use super::{ProductionSession, ProductionStatus};

/// Domain-facing repository contract for Production Sessions.
///
/// The Core defines which persistence capabilities are required.
/// Concrete storage implementations remain behind this boundary.
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

/// Errors returned by the concrete filesystem-backed session repository.
#[derive(Debug)]
pub enum FileProductionSessionRepositoryError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    InvalidStatus(String),
    AlreadyExists,
}

impl std::fmt::Display for FileProductionSessionRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "persistence I/O error: {error}"),
            Self::Serialization(error) => write!(formatter, "persistence serialization error: {error}"),
            Self::InvalidStatus(status) => write!(formatter, "invalid persisted session status: {status}"),
            Self::AlreadyExists => write!(formatter, "production session already exists"),
        }
    }
}

impl std::error::Error for FileProductionSessionRepositoryError {}

impl From<std::io::Error> for FileProductionSessionRepositoryError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for FileProductionSessionRepositoryError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization(error)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedProductionSession {
    id: ProductionId,
    status: PersistedProductionStatus,
    participations: Vec<Participation>,
    recordings: Vec<Recording>,
    activities: Vec<ActivityEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedProductionStatus {
    Created,
    Active,
    Completed,
}

impl From<ProductionStatus> for PersistedProductionStatus {
    fn from(status: ProductionStatus) -> Self {
        match status {
            ProductionStatus::Created => Self::Created,
            ProductionStatus::Active => Self::Active,
            ProductionStatus::Completed => Self::Completed,
        }
    }
}

impl From<PersistedProductionStatus> for ProductionStatus {
    fn from(status: PersistedProductionStatus) -> Self {
        match status {
            PersistedProductionStatus::Created => Self::Created,
            PersistedProductionStatus::Active => Self::Active,
            PersistedProductionStatus::Completed => Self::Completed,
        }
    }
}

impl PersistedProductionSession {
    fn from_domain(session: &ProductionSession) -> Self {
        Self {
            id: session.id.clone(),
            status: session.status.into(),
            participations: session.participations.clone(),
            recordings: session.recordings.clone(),
            activities: session.activities.clone(),
        }
    }

    fn into_domain(self) -> ProductionSession {
        ProductionSession {
            id: self.id,
            status: self.status.into(),
            participations: self.participations,
            recordings: self.recordings,
            activities: self.activities,
        }
    }
}

/// Concrete local persistence implementation for `ProductionSession`.
///
/// The domain only depends on `ProductionSessionRepository`; this type is the
/// infrastructure-side implementation. Each session is stored as one JSON
/// document and updates use a temporary file followed by an atomic rename.
pub struct FileProductionSessionRepository {
    root: PathBuf,
}

impl FileProductionSessionRepository {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, FileProductionSessionRepositoryError> {
        let root = root.into();
        fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn path_for(&self, id: &ProductionId) -> PathBuf {
        let filename = id
            .value()
            .as_bytes()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        self.root.join(format!("{filename}.json"))
    }

    fn write(&self, session: &ProductionSession) -> Result<(), FileProductionSessionRepositoryError> {
        let path = self.path_for(&session.id);
        let temporary = self.root.join(format!(
            ".{}.{}.tmp",
            path.file_name().and_then(|name| name.to_str()).unwrap_or("session"),
            std::process::id()
        ));
        let persisted = PersistedProductionSession::from_domain(session);
        let bytes = serde_json::to_vec_pretty(&persisted)?;

        fs::write(&temporary, bytes)?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        Ok(())
    }
}

impl ProductionSessionRepository for FileProductionSessionRepository {
    type Error = FileProductionSessionRepositoryError;

    fn store(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        let path = self.path_for(&session.id);
        if path.exists() {
            return Err(FileProductionSessionRepositoryError::AlreadyExists);
        }
        self.write(session)
    }

    fn update(&mut self, session: &ProductionSession) -> Result<(), Self::Error> {
        let path = self.path_for(&session.id);
        if !path.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "production session does not exist",
            )
            .into());
        }
        self.write(session)
    }

    fn get(&self, id: &ProductionId) -> Result<Option<ProductionSession>, Self::Error> {
        let path = self.path_for(id);
        if !path.exists() {
            return Ok(None);
        }

        let bytes = fs::read(path)?;
        let persisted: PersistedProductionSession = serde_json::from_slice(&bytes)?;
        Ok(Some(persisted.into_domain()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::participant::ParticipantId;
    use crate::participation::Participation;
    use crate::recording::Recording;
    use crate::role::ParticipantRole;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("nc-pore-production-{nanos}"))
    }

    fn rich_session() -> ProductionSession {
        let id = ProductionId::new("production-001");
        let owner = ParticipantId::new("owner-1");
        let producer = ParticipantId::new("producer-1");
        let mut session = ProductionSession::new_with_actor(id, Some(owner.clone()));
        session
            .add_participation_by(
                &owner,
                Participation::with_roles(
                    owner.clone(),
                    [
                        ParticipantRole::Owner,
                        ParticipantRole::Producer,
                        ParticipantRole::Participant,
                    ],
                ),
            )
            .unwrap();
        session
            .add_participation_by(
                &owner,
                Participation::with_roles(
                    producer,
                    [ParticipantRole::Producer, ParticipantRole::Participant],
                ),
            )
            .unwrap();
        session.start_by(&owner).unwrap();
        session
            .add_recording_by(&owner, Recording::new("recording-001"))
            .unwrap();
        session.start_recording_by(&owner, &crate::recording::RecordingId::new("recording-001")).unwrap();
        session
            .complete_recording_by(
                &owner,
                &crate::recording::RecordingId::new("recording-001"),
                crate::recording::RecordingArtifactId::new("artifact-001"),
            )
            .unwrap();
        session
    }

    #[test]
    fn file_repository_round_trips_complete_session_state_and_history() {
        let root = temp_root();
        let mut repository = FileProductionSessionRepository::new(&root).unwrap();
        let session = rich_session();
        let id = session.id.clone();

        repository.store(&session).unwrap();
        let reloaded = repository.get(&id).unwrap().unwrap();

        assert_eq!(reloaded.id, session.id);
        assert_eq!(reloaded.status(), session.status());
        assert_eq!(reloaded.participations(), session.participations());
        assert_eq!(reloaded.recordings(), session.recordings());
        assert_eq!(reloaded.activities(), session.activities());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_repository_rejects_duplicate_store() {
        let root = temp_root();
        let mut repository = FileProductionSessionRepository::new(&root).unwrap();
        let session = ProductionSession::new(ProductionId::new("production-001"));

        repository.store(&session).unwrap();
        assert!(matches!(
            repository.store(&session),
            Err(FileProductionSessionRepositoryError::AlreadyExists)
        ));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_repository_returns_none_for_missing_session() {
        let root = temp_root();
        let repository = FileProductionSessionRepository::new(&root).unwrap();
        assert!(repository
            .get(&ProductionId::new("missing"))
            .unwrap()
            .is_none());
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn file_repository_update_rejects_missing_session() {
        let root = temp_root();
        let mut repository = FileProductionSessionRepository::new(&root).unwrap();
        let session = ProductionSession::new(ProductionId::new("missing"));
        assert!(matches!(
            repository.update(&session),
            Err(FileProductionSessionRepositoryError::Io(error))
                if error.kind() == std::io::ErrorKind::NotFound
        ));
        let _ = fs::remove_dir_all(root);
    }
}
