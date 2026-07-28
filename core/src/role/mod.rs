/// Defines the role of a participant within a production session.
///
/// Roles describe responsibility inside a production context.
///
/// See ADR-006 and ADR-031.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
