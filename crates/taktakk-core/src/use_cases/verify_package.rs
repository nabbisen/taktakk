//! Package verification use case.

use crate::domain::package::{check_magic, NMP_FORMAT_VERSION};
use crate::error::{CoreError, CoreResult};
use crate::ports::crypto::{HashProvider, SignatureVerifier};

/// Lightweight verification result without full extraction.
#[derive(Debug)]
pub struct VerificationSummary {
    pub module_id: String,
    pub signer_id: String,
    pub object_count: usize,
}

/// Verify the magic bytes and format version of a raw `.nmp` buffer.
pub fn check_nmp_header(data: &[u8]) -> CoreResult<()> {
    if !check_magic(data) {
        return Err(CoreError::Internal("invalid magic bytes".to_string()));
    }
    if data.len() < 6 {
        return Err(CoreError::Internal("package too short".to_string()));
    }
    let format_version = data[4];
    if format_version != NMP_FORMAT_VERSION {
        return Err(CoreError::UnsupportedVersion { version: format_version });
    }
    Ok(())
}

/// Verify the Ed25519 signature over the manifest bytes.
pub fn verify_manifest_signature(
    verifier: &dyn SignatureVerifier,
    signer_id: &str,
    manifest_bytes: &[u8],
    signature_bytes: &[u8],
) -> CoreResult<()> {
    verifier.verify_ed25519(signer_id, manifest_bytes, signature_bytes)
}

/// Verify the SHA-256 hash of an extracted object.
pub fn verify_object_hash(
    hasher: &dyn HashProvider,
    data: &[u8],
    expected_hex: &str,
) -> CoreResult<()> {
    let actual = hasher.sha256_hex(data);
    if actual != expected_hex {
        return Err(CoreError::HashMismatch { hash: expected_hex.to_string() });
    }
    Ok(())
}
