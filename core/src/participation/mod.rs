use crate::participant::ParticipantId;
use crate::role::ParticipantRole;

/// Represents a participant's involvement in a production session.
///
/// A participant identity is separated from the responsibility
/// they have within a specific production.
///
/// See ADR-031.
#[derive(Debug, Clone)]
pub struct Participation {
    pub participant_id: ParticipantId,
    pub role: ParticipantRole,
}

impl Participation {
    pub fn new(participant_id: ParticipantId, role: ParticipantRole) -> Self {
        Self {
            participant_id,
            role,
        }
    }

    /// Checks whether this participation belongs to an owner.
    pub fn is_owner(&self) -> bool {
        self.role == ParticipantRole::Owner
    }

    /// Checks whether this participation belongs to a guest.
    pub fn is_guest(&self) -> bool {
        self.role == ParticipantRole::Guest
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_participation_is_detected() {
        let participation =
            Participation::new(ParticipantId::new("owner-1"), ParticipantRole::Owner);

        assert!(participation.is_owner());
        assert!(!participation.is_guest());
    }

    #[test]
    fn guest_participation_is_detected() {
        let participation =
            Participation::new(ParticipantId::new("guest-1"), ParticipantRole::Guest);

        assert!(participation.is_guest());
        assert!(!participation.is_owner());
    }
}
