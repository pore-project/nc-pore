//! Technical recorder library.
//!
//! The recorder crate contains the concrete capture and artifact pipeline.
//! It deliberately remains independent from the domain Core and from the
//! application orchestration layer.

pub mod application;
pub mod artifact;
pub mod audio;
pub mod export;
pub mod metadata;
pub mod persistence;
pub mod session;
pub mod storage;
pub mod workflow;
