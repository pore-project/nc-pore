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
            Self::Participant => matches!(action, ProductionAction::ParticipateInRecording),
            Self::Guest => false,
        }
    }
}
