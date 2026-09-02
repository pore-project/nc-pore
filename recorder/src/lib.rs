//! Technical recorder library.
//!
//! The recorder crate contains the concrete capture, preservation, transport
//! and artifact pipeline. It deliberately remains independent from the domain
//! Core and from the application orchestration layer.

pub mod application;
pub mod artifact;
pub mod audio;
pub mod completion;
pub mod export;
pub mod host;
pub mod metadata;
pub mod persistence;
pub mod preservation;
pub mod remote;
pub mod session;
pub mod storage;
pub mod transport;
pub mod workflow;
