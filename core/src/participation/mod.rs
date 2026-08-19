use crate::participant::ParticipantId;
use crate::role::{ParticipantRole, ProductionAction};
use serde::{Deserialize, Serialize};

/// Represents a participant's involvement in a production session.
///
/// A participant identity is separated from the responsibilities they have
/// within a specific production. Multiple roles may be held simultaneously.
///
/// See ADR-031 and the refined role semantics in issue #95.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Participation {
    pub participant_id: ParticipantId,
    pub roles: Vec<ParticipantRole>,
}

impl Participation {
    pub fn new(participant_id: ParticipantId, role: ParticipantRole) -> Self {
        Self {
            participant_id,
            roles: vec![role],
        }
    }

    pub fn with_roles(
        participant_id: ParticipantId,
        roles: impl IntoIterator<Item = ParticipantRole>,
    ) -> Self {
        let mut roles: Vec<_> = roles.into_iter().collect();
        roles.sort();
        roles.dedup();
        Self {
            participant_id,
            roles,
        }
    }

    pub fn has_role(&self, role: ParticipantRole) -> bool {
        self.roles.contains(&role)
    }

    pub fn allows(&self, action: ProductionAction) -> bool {
        self.roles.iter().copied().any(|role| role.allows(action))
    }

    pub fn is_owner(&self) -> bool {
        self.has_role(ParticipantRole::Owner)
    }

    pub fn is_producer(&self) -> bool {
        self.has_role(ParticipantRole::Producer)
    }

    pub fn is_participant(&self) -> bool {
        self.has_role(ParticipantRole::Participant)
    }

    pub fn is_guest(&self) -> bool {
        self.has_role(ParticipantRole::Guest)
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
    fn combined_roles_are_supported() {
        let participation = Participation::with_roles(
            ParticipantId::new("owner-1"),
            [
                ParticipantRole::Owner,
                ParticipantRole::Producer,
                ParticipantRole::Participant,
            ],
        );

        assert!(participation.is_owner());
        assert!(participation.is_producer());
        assert!(participation.is_participant());
        assert!(!participation.is_guest());
        assert!(participation.allows(ProductionAction::ManageParticipants));
        assert!(participation.allows(ProductionAction::ParticipateInRecording));
    }

    #[test]
    fn producer_may_also_be_participant() {
        let participation = Participation::with_roles(
            ParticipantId::new("producer-1"),
            [ParticipantRole::Producer, ParticipantRole::Participant],
        );

        assert!(participation.allows(ProductionAction::ManageRecordings));
        assert!(participation.allows(ProductionAction::ParticipateInRecording));
        assert!(!participation.has_role(ParticipantRole::Owner));
    }

    #[test]
    fn participant_does_not_gain_producer_or_owner_authority() {
        let participation = Participation::new(
            ParticipantId::new("participant-1"),
            ParticipantRole::Participant,
        );

        assert!(!participation.is_producer());
        assert!(!participation.is_owner());
        assert!(!participation.allows(ProductionAction::ManageParticipants));
    }

    #[test]
    fn guest_has_limited_permissions() {
        let participation =
            Participation::new(ParticipantId::new("guest-1"), ParticipantRole::Guest);

        assert!(participation.is_guest());
        assert!(!participation.allows(ProductionAction::ManageRecordings));
        assert!(!participation.allows(ProductionAction::ParticipateInRecording));
    }
}
