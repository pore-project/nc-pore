#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantRole {
    Owner,
    Producer,
    Participant,
    Guest,
}
