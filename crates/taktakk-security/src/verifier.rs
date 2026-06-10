//! Signature verifier and unlock-slot verifier.

use ed25519_dalek::{Signature, Verifier};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use taktakk_core::error::{CoreError, CoreResult};
use taktakk_core::ports::crypto::{SignatureVerifier, UnlockVerifier};

use crate::trust_anchor::{TrustAnchor, TrustAnchorStatus};

/// Ed25519 signature verifier backed by `ed25519-dalek`.
///
/// Trust anchors are loaded at startup from the embedded binary list and
/// any additional anchors stored in `core.sqlite`.
pub struct Ed25519Verifier {
    anchors: Vec<TrustAnchor>,
}

impl Ed25519Verifier {
    pub fn new(anchors: Vec<TrustAnchor>) -> Self {
        Self { anchors }
    }
}

impl SignatureVerifier for Ed25519Verifier {
    fn verify_ed25519(
        &self,
        signer_id: &str,
        message: &[u8],
        signature: &[u8],
    ) -> CoreResult<()> {
        let anchor = self
            .anchors
            .iter()
            .find(|a| a.signing_key_id == signer_id && a.status == TrustAnchorStatus::Active)
            .ok_or_else(|| CoreError::SignatureVerificationFailed)?;

        let vk = anchor
            .verifying_key()
            .map_err(|_| CoreError::SignatureVerificationFailed)?;

        let sig_bytes: [u8; 64] = signature
            .try_into()
            .map_err(|_| CoreError::SignatureVerificationFailed)?;
        let sig = Signature::from_bytes(&sig_bytes);

        vk.verify(message, &sig)
            .map_err(|_| CoreError::SignatureVerificationFailed)
    }
}

/// KDF algorithm identifier stored in unlock slots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum KdfAlg {
    Argon2id,
}

/// An unlock slot record (stored in `facade.sqlite` with obfuscated column names).
///
/// Never stores the raw unlock sequence or passcode; only the KDF verifier hash.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockSlot {
    pub unlock_slot_id: String,
    pub kdf_alg: KdfAlg,
    /// Argon2id parameters encoded as JSON.
    pub kdf_params_json: String,
    /// KDF salt (16–32 bytes).
    pub salt: Vec<u8>,
    /// The expected KDF output; compared in constant time.
    pub verifier_hash: Vec<u8>,
    /// ID of the wrapped key this slot unlocks.
    pub wrapped_key_id: Option<String>,
    pub failure_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Argon2id parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Argon2Params {
    pub m_cost: u32,
    pub t_cost: u32,
    pub p_cost: u32,
    pub output_len: usize,
}

impl Default for Argon2Params {
    fn default() -> Self {
        // Deliberately modest parameters for low-end hardware (ARMv7).
        Self {
            m_cost: 32 * 1024, // 32 MiB
            t_cost: 2,
            p_cost: 1,
            output_len: 32,
        }
    }
}

/// Argon2id-backed unlock verifier.
pub struct Argon2Verifier {
    pub slot: UnlockSlot,
    pub duress_slot: Option<UnlockSlot>,
}

impl UnlockVerifier for Argon2Verifier {
    fn verify_unlock_sequence(&self, sequence_bytes: &[u8]) -> CoreResult<bool> {
        verify_against_slot(&self.slot, sequence_bytes)
    }

    fn verify_duress_sequence(&self, sequence_bytes: &[u8]) -> CoreResult<bool> {
        match &self.duress_slot {
            Some(slot) => verify_against_slot(slot, sequence_bytes),
            None => Ok(false),
        }
    }
}

fn verify_against_slot(slot: &UnlockSlot, sequence_bytes: &[u8]) -> CoreResult<bool> {
    let params: Argon2Params = serde_json::from_str(&slot.kdf_params_json)
        .map_err(|e| CoreError::Crypto(e.to_string()))?;

    let argon2_params = argon2::Params::new(
        params.m_cost,
        params.t_cost,
        params.p_cost,
        Some(params.output_len),
    )
    .map_err(|e| CoreError::Crypto(e.to_string()))?;

    let argon2 = argon2::Argon2::new(
        argon2::Algorithm::Argon2id,
        argon2::Version::V0x13,
        argon2_params,
    );

    let mut derived = vec![0u8; params.output_len];
    argon2
        .hash_password_into(sequence_bytes, &slot.salt, &mut derived)
        .map_err(|e| CoreError::Crypto(e.to_string()))?;

    // Constant-time comparison to avoid timing side-channels.
    let matched = constant_time_eq(&derived, &slot.verifier_hash);
    derived.zeroize();
    Ok(matched)
}

/// Constant-time byte slice comparison.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
