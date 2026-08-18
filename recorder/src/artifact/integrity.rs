//! Integrity primitives for recording artifacts.
//!
//! Integrity is part of the artifact model and deliberately independent of
//! any concrete persistence provider. The hashing primitive is kept small so
//! the capture path can later feed it incrementally without coupling capture
//! to storage latency.

use sha2::{Digest, Sha256};

/// SHA-256 digest of technical recording data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PayloadHash([u8; 32]);

impl PayloadHash {
    /// Calculates the SHA-256 digest of a complete byte slice.
    ///
    /// This convenience constructor is suitable for already-buffered data.
    /// Streaming capture should use an incremental `Sha256` hasher and create
    /// the resulting `PayloadHash` once the chunk is complete.
    pub fn from_bytes(data: &[u8]) -> Self {
        let digest = Sha256::digest(data);
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(&digest);
        Self(bytes)
    }

    /// Returns the raw 32-byte SHA-256 digest.
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_hash_is_deterministic() {
        let first = PayloadHash::from_bytes(b"nc-pore");
        let second = PayloadHash::from_bytes(b"nc-pore");

        assert_eq!(first, second);
    }

    #[test]
    fn different_payloads_have_different_hashes() {
        let first = PayloadHash::from_bytes(b"nc-pore");
        let second = PayloadHash::from_bytes(b"nc-pore!");

        assert_ne!(first, second);
    }
}
