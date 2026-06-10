//! Trust anchor management.
//!
//! Trust anchors are Ed25519 public keys of organisations authorised to
//! publish taktakk content packages. They are embedded in the binary and
//! stored in `core.sqlite`.

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

/// A trust anchor: a named Ed25519 public key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAnchor {
    pub signing_key_id: String,
    pub label: String,
    /// Ed25519 public key bytes (32 bytes).
    pub public_key_bytes: Vec<u8>,
    pub added_at: i64,
    pub status: TrustAnchorStatus,
}

/// Status of a trust anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustAnchorStatus {
    Trusted,
    Revoked,
}

impl TrustAnchor {
    /// Parse and validate the stored public key bytes.
    pub fn verifying_key(&self) -> Result<VerifyingKey, ed25519_dalek::SignatureError> {
        let bytes: [u8; 32] = self
            .public_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| ed25519_dalek::SignatureError::from_source("invalid key length"))?;
        VerifyingKey::from_bytes(&bytes)
    }
}
