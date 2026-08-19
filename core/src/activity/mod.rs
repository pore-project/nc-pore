use std::sync::atomic::{AtomicU64, Ordering};
use std::time::SystemTime;

use crate::identity::ProductionId;
use crate::participant::ParticipantId;
use serde::{Deserialize, Serialize};

static NEXT_EVENT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityType {
    SessionCreated,
    SessionStarted,
    SessionCompleted,
    ParticipantAdded,
    RecordingAdded,
    RecordingStarted,
    RecordingCompleted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActivityResult {
    Success,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActivityEvent {
    pub event_id: String,
    pub timestamp: SystemTime,
    pub actor: Option<ParticipantId>,
    pub activity_type: ActivityType,
    pub target: Option<String>,
    pub session_id: ProductionId,
    pub result: ActivityResult,
}

impl ActivityEvent {
    pub fn new(
        activity_type: ActivityType,
        session_id: ProductionId,
        actor: Option<ParticipantId>,
        target: Option<String>,
        result: ActivityResult,
    ) -> Self {
        let sequence = NEXT_EVENT_ID.fetch_add(1, Ordering::Relaxed);
        let event_id = format!(
            "{}-{}",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or_default(),
            sequence
        );

        Self {
            event_id,
            timestamp: SystemTime::now(),
            actor,
            activity_type,
            target,
            session_id,
            result,
        }
    }
}
