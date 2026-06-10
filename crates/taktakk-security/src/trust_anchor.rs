//! Trust anchor management.
//!
//! Trust anchors are Ed25519 public keys authorised to publish taktakk
//! content packages. They are stored in `core.sqlite`.
//!
//! **Privacy rule (RFC-047):** No organisation name or label is stored
//! on the device. Labels are operator metadata that must remain on the
//! distributor's seed-kit machine only. The on-device record contains
//! only the key ID, public key bytes, scope, and status.

use ed25519_dalek::VerifyingKey;
use serde::{Deserialize, Serialize};

/// A trust anchor: an Ed25519 public key authorised to sign packages.
///
/// No `label` field — organisation names are not stored on devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustAnchor {
    pub signing_key_id: String,
    /// Ed25519 public key bytes (32 bytes).
    pub public_key_bytes: Vec<u8>,
    pub added_at: i64,
    pub status: TrustAnchorStatus,
}

/// Status of a trust anchor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrustAnchorStatus {
    Active,   // renamed from Trusted for clarity
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

    /// `true` if this anchor can be used to verify new package signatures.
    pub fn is_active(&self) -> bool {
        self.status == TrustAnchorStatus::Active
    }
}
