//! Crypto key slot model.
//!
//! Key material is never stored in plaintext. Each slot stores a wrapped
//! (encrypted) key that is only usable after the correct unlock sequence
//! has been verified.
//!
//! On panic wipe, key slots are overwritten with random bytes before
//! any slower deletion tasks run.

use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Purpose of a crypto key slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyPurpose {
    /// Encrypts the core SQLite database.
    State,
    /// Encrypts the content catalog metadata.
    Catalog,
    /// Encrypts individual content objects.
    Object,
    /// Used during sync to wrap per-session transfer keys.
    Sync,
}

/// Lifecycle status of a key slot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    Active,
    Retired,
    Destroyed,
}

/// A persisted wrapped-key record (stored in `facade.sqlite`).
///
/// The actual key material is never stored here in plaintext; only the
/// wrapped (encrypted) form is persisted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoKeySlot {
    pub key_id: String,
    pub purpose: KeyPurpose,
    /// The wrapped key bytes. Overwritten with random noise during wipe.
    pub wrapped_key: Vec<u8>,
    pub alg: String,
    pub created_at: i64,
    pub rotated_at: Option<i64>,
    pub status: KeyStatus,
}

impl Drop for CryptoKeySlot {
    fn drop(&mut self) {
        // Zeroize wrapped key on drop to limit key material lifetime in RAM.
        self.wrapped_key.zeroize();
    }
}

/// In-memory representation of a derived (unwrapped) symmetric key.
///
/// Zeroized on drop. Never serialised or written to disk.
#[derive(Clone, Zeroize)]
#[zeroize(drop)]
pub struct DerivedKey(pub [u8; 32]);

impl std::fmt::Debug for DerivedKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DerivedKey([REDACTED])")
    }
}
