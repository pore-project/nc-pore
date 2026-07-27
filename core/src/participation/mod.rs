use crate::participant::ParticipantId;
use crate::role::ParticipantRole;

#[derive(Debug, Clone)]
pub struct Participation {
    pub participant_id: ParticipantId,
    pub role: ParticipantRole,
}
