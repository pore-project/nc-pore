//! Audio transport boundary.
//!
//! Transport representations are deliberately separate from capture and
//! preservation. The V1 default transport is lossless FLAC.

mod flac;

pub use flac::{FlacEncodeError, encode_chunks as encode_flac};
