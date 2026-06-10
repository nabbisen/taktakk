//! Hash helpers implementing the [`HashProvider`] port.

use sha2::{Digest, Sha256};
use taktakk_core::ports::crypto::HashProvider;

/// SHA-256 hash provider backed by the `sha2` crate.
pub struct Sha256Hasher;

impl HashProvider for Sha256Hasher {
    fn sha256_hex(&self, data: &[u8]) -> String {
        let hash = Sha256::digest(data);
        hex::encode(hash)
    }
}

/// Compute a SHA-256 hash without constructing the struct.
pub fn sha256_hex(data: &[u8]) -> String {
    Sha256Hasher.sha256_hex(data)
}
