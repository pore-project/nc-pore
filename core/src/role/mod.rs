/// Defines the role of a participant within a production session.
///
/// Roles describe responsibility inside a production context.
///
/// See ADR-006 and ADR-031.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParticipantRole {
    /// Owns and is responsible for the production session.
    Owner,

    /// Supports production management and coordination.
    Producer,

    /// Actively participates in the production.
    Participant,

    /// External participant with limited permissions.
    Guest,
}

/// Domain operations that require a production role.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionAction {
    StartSession,
    CompleteSession,
    ManageParticipants,
    ManageRecordings,
    ParticipateInRecording,
}

impl ParticipantRole {
    /// Returns whether this role is sufficient for the requested operation.
    ///
    /// The permission direction is intentionally one-way: higher-responsibility
    /// roles may exercise lower-responsibility capabilities, but lower roles do
    /// not gain higher-role authority.
    pub fn allows(self, action: ProductionAction) -> bool {
        match self {
            Self::Owner => true,
            Self::Producer => matches!(
                action,
                ProductionAction::StartSession
                    | ProductionAction::CompleteSession
                    | ProductionAction::ManageParticipants
                    | ProductionAction::ManageRecordings
                    | ProductionAction::ParticipateInRecording
            ),
            Self::Participant => {
                matches!(action, ProductionAction::ParticipateInRecording)
            }
            Self::Guest => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn owner_has_all_v1_capabilities() {
        assert!(ParticipantRole::Owner.allows(ProductionAction::StartSession));
        assert!(ParticipantRole::Owner.allows(ProductionAction::ManageParticipants));
        assert!(ParticipantRole::Owner.allows(ProductionAction::ParticipateInRecording));
    }

    #[test]
    fn producer_has_participant_capabilities_but_not_owner_only_semantics() {
        assert!(ParticipantRole::Producer.allows(ProductionAction::ManageRecordings));
        assert!(ParticipantRole::Producer.allows(ProductionAction::ParticipateInRecording));
    }

    #[test]
    fn participant_cannot_manage_production() {
        assert!(!ParticipantRole::Participant.allows(ProductionAction::ManageParticipants));
        assert!(!ParticipantRole::Participant.allows(ProductionAction::StartSession));
        assert!(ParticipantRole::Participant.allows(ProductionAction::ParticipateInRecording));
    }

    #[test]
    fn guest_has_no_management_or_recording_capabilities() {
        assert!(!ParticipantRole::Guest.allows(ProductionAction::ManageRecordings));
        assert!(!ParticipantRole::Guest.allows(ProductionAction::ParticipateInRecording));
    }
}
