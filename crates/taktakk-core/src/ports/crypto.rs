//! Cryptography port: hashing, signature verification, and key management.

use crate::error::CoreResult;

/// SHA-256 and BLAKE3 hashing.
pub trait HashProvider: Send + Sync {
    /// Compute the SHA-256 hash of `data`, returning a lowercase hex string.
    fn sha256_hex(&self, data: &[u8]) -> String;
}

/// Ed25519 signature verification.
pub trait SignatureVerifier: Send + Sync {
    /// Verify that `signature` was produced over `message` by the key
    /// identified by `signer_id`. Returns `Ok(())` on success.
    fn verify_ed25519(
        &self,
        signer_id: &str,
        message: &[u8],
        signature: &[u8],
    ) -> CoreResult<()>;
}

/// Key-derivation and unlock-slot verification.
pub trait UnlockVerifier: Send + Sync {
    /// Returns `true` if the provided sequence matches the stored verifier.
    fn verify_unlock_sequence(&self, sequence_bytes: &[u8]) -> CoreResult<bool>;

    /// Returns `true` if the provided sequence is a duress trigger.
    fn verify_duress_sequence(&self, sequence_bytes: &[u8]) -> CoreResult<bool>;
}

/// Wipe coordinator: responsible for key destruction.
pub trait WipeCoordinator: Send + Sync {
    /// Destroy all crypto key slots.
    /// Must complete before any slower deletion tasks are started.
    fn destroy_keys(&self) -> CoreResult<()>;
}
