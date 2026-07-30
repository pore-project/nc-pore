//! Legacy session prototype model.
//!
//! This module contains the early session model created during initial
//! recorder experiments.
//!
//! Currently this module is not actively used by the production workflow.
//! It remains temporarily until the architectural responsibility of the
//! recorder component has been finalized.
//!
//! It is intentionally kept separate from the NC-PoRe core domain model.
//!
//! The authoritative production session implementation is:
//!
//! - core::session::ProductionSession
//!
//! The core model contains:
//!
//! - production lifecycle management
//! - participant handling
//! - ownership rules
//! - recording associations
//! - activity history
//!
//! This module may be removed or replaced once the recorder component
//! has a defined architectural role.
//!
//! See:
//! - ADR-038 Core Implementation Structure and Module Organization
//! - ADR-031 Session Lifecycle and Ownership Rules

#[derive(Debug)]
pub enum SessionStatus {
    Created,
    Recording,
    Stopped,
    Stored,
    Failed,
}

#[derive(Debug)]
pub struct RecordingSession {
    pub id: String,
    pub status: SessionStatus,
}
