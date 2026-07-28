/// Describes the responsibility of a participant within a production session.
///
/// Roles express responsibilities inside a production,
/// not the identity of a person.
///
/// See ADR-031.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantRole {
    /// Owns the production session.
    Owner,

    /// Coordinates and manages the production.
    Producer,

    /// Regular participant of the production.
    Participant,

    /// Limited external participant.
    Guest,
}
