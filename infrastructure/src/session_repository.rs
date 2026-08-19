use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};

use nc_pore_core::activity::{ActivityEvent, ActivityResult, ActivityType};
use nc_pore_core::identity::ProductionId;
use nc_pore_core::participant::ParticipantId;
use nc_pore_core::participation::Participation;
use nc_pore_core::recording::{Recording, RecordingArtifactId, RecordingId, RecordingStatus};
use nc_pore_core::role::ParticipantRole;
use nc_pore_core::session::{
    repository::ProductionSessionRepository, ProductionSession, ProductionStatus,
};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum FileProductionSessionRepositoryError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    InvalidTimestamp(u128),
    AlreadyExists,
}

impl std::fmt::Display for FileProductionSessionRepositoryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "persistence I/O error: {error}"),
            Self::Serialization(error) => {
                write!(formatter, "persistence serialization error: {error}")
            }
            Self::InvalidTimestamp(value) => {
                write!(formatter, "invalid persisted activity timestamp: {value}")
            }
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
    id: String,
    status: PersistedProductionStatus,
    participations: Vec<PersistedParticipation>,
    recordings: Vec<PersistedRecording>,
    activities: Vec<PersistedActivityEvent>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedProductionStatus {
    Created,
    Active,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedParticipation {
    participant_id: String,
    roles: Vec<PersistedParticipantRole>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedParticipantRole {
    Owner,
    Producer,
    Participant,
    Guest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedRecording {
    id: String,
    status: PersistedRecordingStatus,
    artifact_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedRecordingStatus {
    Prepared,
    Recording,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PersistedActivityEvent {
    event_id: String,
    timestamp_nanos: u128,
    actor: Option<String>,
    activity_type: PersistedActivityType,
    target: Option<String>,
    session_id: String,
    result: PersistedActivityResult,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedActivityType {
    SessionCreated,
    SessionStarted,
    SessionCompleted,
    ParticipantAdded,
    RecordingAdded,
    RecordingStarted,
    RecordingCompleted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum PersistedActivityResult {
    Success,
    Rejected,
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

impl From<ParticipantRole> for PersistedParticipantRole {
    fn from(role: ParticipantRole) -> Self {
        match role {
            ParticipantRole::Owner => Self::Owner,
            ParticipantRole::Producer => Self::Producer,
            ParticipantRole::Participant => Self::Participant,
            ParticipantRole::Guest => Self::Guest,
        }
    }
}

impl From<PersistedParticipantRole> for ParticipantRole {
    fn from(role: PersistedParticipantRole) -> Self {
        match role {
            PersistedParticipantRole::Owner => Self::Owner,
            PersistedParticipantRole::Producer => Self::Producer,
            PersistedParticipantRole::Participant => Self::Participant,
            PersistedParticipantRole::Guest => Self::Guest,
        }
    }
}

impl From<RecordingStatus> for PersistedRecordingStatus {
    fn from(status: RecordingStatus) -> Self {
        match status {
            RecordingStatus::Prepared => Self::Prepared,
            RecordingStatus::Recording => Self::Recording,
            RecordingStatus::Completed => Self::Completed,
        }
    }
}

impl From<PersistedRecordingStatus> for RecordingStatus {
    fn from(status: PersistedRecordingStatus) -> Self {
        match status {
            PersistedRecordingStatus::Prepared => Self::Prepared,
            PersistedRecordingStatus::Recording => Self::Recording,
            PersistedRecordingStatus::Completed => Self::Completed,
        }
    }
}

impl From<ActivityType> for PersistedActivityType {
    fn from(activity_type: ActivityType) -> Self {
        match activity_type {
            ActivityType::SessionCreated => Self::SessionCreated,
            ActivityType::SessionStarted => Self::SessionStarted,
            ActivityType::SessionCompleted => Self::SessionCompleted,
            ActivityType::ParticipantAdded => Self::ParticipantAdded,
            ActivityType::RecordingAdded => Self::RecordingAdded,
            ActivityType::RecordingStarted => Self::RecordingStarted,
            ActivityType::RecordingCompleted => Self::RecordingCompleted,
        }
    }
}

impl From<PersistedActivityType> for ActivityType {
    fn from(activity_type: PersistedActivityType) -> Self {
        match activity_type {
            PersistedActivityType::SessionCreated => Self::SessionCreated,
            PersistedActivityType::SessionStarted => Self::SessionStarted,
            PersistedActivityType::SessionCompleted => Self::SessionCompleted,
            PersistedActivityType::ParticipantAdded => Self::ParticipantAdded,
            PersistedActivityType::RecordingAdded => Self::RecordingAdded,
            PersistedActivityType::RecordingStarted => Self::RecordingStarted,
            PersistedActivityType::RecordingCompleted => Self::RecordingCompleted,
        }
    }
}

impl From<ActivityResult> for PersistedActivityResult {
    fn from(result: ActivityResult) -> Self {
        match result {
            ActivityResult::Success => Self::Success,
            ActivityResult::Rejected => Self::Rejected,
        }
    }
}

impl From<PersistedActivityResult> for ActivityResult {
    fn from(result: PersistedActivityResult) -> Self {
        match result {
            PersistedActivityResult::Success => Self::Success,
            PersistedActivityResult::Rejected => Self::Rejected,
        }
    }
}

impl PersistedProductionSession {
    fn from_domain(session: &ProductionSession) -> Self {
        Self {
            id: session.id.value().to_owned(),
            status: session.status().into(),
            participations: session
                .participations()
                .iter()
                .map(|participation| PersistedParticipation {
                    participant_id: participation.participant_id.value().to_owned(),
                    roles: participation
                        .roles
                        .iter()
                        .copied()
                        .map(Into::into)
                        .collect(),
                })
                .collect(),
            recordings: session
                .recordings()
                .iter()
                .map(|recording| PersistedRecording {
                    id: recording.id().value().to_owned(),
                    status: recording.status().into(),
                    artifact_id: recording.artifact_id().map(|id| id.value().to_owned()),
                })
                .collect(),
            activities: session
                .activities()
                .iter()
                .map(|activity| PersistedActivityEvent {
                    event_id: activity.event_id.clone(),
                    timestamp_nanos: activity
                        .timestamp
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_nanos(),
                    actor: activity
                        .actor
                        .as_ref()
                        .map(|participant| participant.value().to_owned()),
                    activity_type: activity.activity_type.into(),
                    target: activity.target.clone(),
                    session_id: activity.session_id.value().to_owned(),
                    result: activity.result.into(),
                })
                .collect(),
        }
    }

    fn into_domain(self) -> Result<ProductionSession, FileProductionSessionRepositoryError> {
        let participations = self
            .participations
            .into_iter()
            .map(|participation| {
                Participation::with_roles(
                    ParticipantId::new(participation.participant_id),
                    participation.roles.into_iter().map(Into::into),
                )
            })
            .collect();

        let recordings = self
            .recordings
            .into_iter()
            .map(|recording| {
                Recording::reconstitute(
                    RecordingId::new(recording.id),
                    recording.status.into(),
                    recording.artifact_id.map(RecordingArtifactId::new),
                )
            })
            .collect();

        let mut activities = Vec::with_capacity(self.activities.len());
        for activity in self.activities {
            let seconds = activity.timestamp_nanos / 1_000_000_000;
            let nanos = activity.timestamp_nanos % 1_000_000_000;
            if seconds > u64::MAX as u128 {
                return Err(FileProductionSessionRepositoryError::InvalidTimestamp(
                    activity.timestamp_nanos,
                ));
            }
            let timestamp = UNIX_EPOCH
                .checked_add(Duration::new(seconds as u64, nanos as u32))
                .ok_or(FileProductionSessionRepositoryError::InvalidTimestamp(
                    activity.timestamp_nanos,
                ))?;
            activities.push(ActivityEvent::reconstitute(
                activity.event_id,
                timestamp,
                activity.actor.map(ParticipantId::new),
                activity.activity_type.into(),
                activity.target,
                ProductionId::new(activity.session_id),
                activity.result.into(),
            ));
        }

        Ok(
            nc_pore_core::session::repository::reconstitute_production_session(
                ProductionId::new(self.id),
                self.status.into(),
                participations,
                recordings,
                activities,
            ),
        )
    }
}

/// Concrete local filesystem persistence for `ProductionSession`.
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

    fn write(
        &self,
        session: &ProductionSession,
    ) -> Result<(), FileProductionSessionRepositoryError> {
        let path = self.path_for(&session.id);
        let temporary = self.root.join(format!(
            ".{}.{}.tmp",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("session"),
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
        Ok(Some(persisted.into_domain()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nc_pore_core::recording::RecordingArtifactId;
    use nc_pore_core::role::ParticipantRole;
    use std::time::{SystemTime, UNIX_EPOCH};

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
        session
            .start_recording_by(&owner, &RecordingId::new("recording-001"))
            .unwrap();
        session
            .complete_recording_by(
                &owner,
                &RecordingId::new("recording-001"),
                RecordingArtifactId::new("artifact-001"),
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
