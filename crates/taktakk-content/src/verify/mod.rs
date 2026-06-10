//! Verification layer: Ed25519 signature check and per-object hash check.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};

use taktakk_security::trust_anchor::{TrustAnchor, TrustAnchorStatus};

use crate::nmp::error::{ContentError, ContentResult};
use crate::nmp::reader::ParsedPackage;

/// Verify the Ed25519 signature of a parsed package against the provided
/// trust anchors.
///
/// The signature is over the **manifest bytes only**; object data is
/// verified separately by [`verify_objects`].
pub fn verify_signature(
    pkg: &ParsedPackage,
    trust_anchors: &[TrustAnchor],
) -> ContentResult<()> {
    let signer_id = &pkg.manifest.signer_id;

    let anchor = trust_anchors
        .iter()
        .find(|a| &a.signing_key_id == signer_id && a.status == TrustAnchorStatus::Active)
        .ok_or_else(|| ContentError::UnknownSigner(signer_id.clone()))?;

    let key_bytes: [u8; 32] = anchor
        .public_key_bytes
        .as_slice()
        .try_into()
        .map_err(|_| ContentError::SignatureFailed)?;

    let vk = VerifyingKey::from_bytes(&key_bytes)
        .map_err(|_| ContentError::SignatureFailed)?;

    let sig = Signature::from_bytes(&pkg.signature);

    vk.verify(&pkg.manifest_bytes, &sig)
        .map_err(|_| ContentError::SignatureFailed)
}

/// Verify the SHA-256 hash of each extracted object against its manifest entry.
///
/// Returns the index of the first failing object on error.
pub fn verify_objects(pkg: &ParsedPackage) -> ContentResult<()> {
    for (entry, data) in pkg.manifest.objects.iter().zip(pkg.objects.iter()) {
        let actual = hex::encode(Sha256::digest(data));
        if actual != entry.sha256 {
            return Err(ContentError::HashMismatch {
                path: entry.path.clone(),
                expected: entry.sha256.clone(),
                actual,
            });
        }
    }
    Ok(())
}

/// Run both signature and object-hash checks.
pub fn verify_all(pkg: &ParsedPackage, trust_anchors: &[TrustAnchor]) -> ContentResult<()> {
    verify_signature(pkg, trust_anchors)?;
    verify_objects(pkg)
}

/// Verify a raw Ed25519 signature against manifest bytes and trust anchors.
///
/// Used by `NmpStreamReader` which has the raw bytes and signature array
/// before constructing a `ParsedPackage`.
pub fn verify_signature_bytes(
    manifest_bytes: &[u8],
    signature: &[u8; 64],
    manifest: &taktakk_core::domain::package::PackageManifest,
    trust_anchors: &[TrustAnchor],
) -> ContentResult<()> {
    use ed25519_dalek::{Signature, Verifier};

    let anchor = trust_anchors
        .iter()
        .find(|a| a.signing_key_id == manifest.signer_id && a.is_active())
        .ok_or_else(|| ContentError::UnknownSigner(manifest.signer_id.clone()))?;

    let verifying_key = anchor
        .verifying_key()
        .map_err(|e| ContentError::SignatureVerification(e.to_string()))?;

    let sig = Signature::from_bytes(signature);
    verifying_key
        .verify(manifest_bytes, &sig)
        .map_err(|e| ContentError::SignatureVerification(e.to_string()))
}
