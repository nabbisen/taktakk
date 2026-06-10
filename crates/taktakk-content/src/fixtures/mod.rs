//! Test fixtures: a deterministic Ed25519 keypair and a sample signed package.
//!
//! **Never use these keys in production.** They exist solely for unit and
//! integration testing of the package verification pipeline.

use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use taktakk_core::domain::curriculum::ModuleVersion;
use taktakk_core::domain::package::{ObjectType, PackageManifest};
use taktakk_security::trust_anchor::{TrustAnchor, TrustAnchorStatus};

use crate::nmp::writer::NmpWriter;
use crate::nmp::error::ContentResult;

/// The constant seed for the test signing key (32 bytes of 0xAB…).
///
/// This is a well-known test value; rotate for any real deployment.
pub const TEST_KEY_SEED: [u8; 32] = [0xABu8; 32];

/// Build the deterministic test `SigningKey`.
pub fn test_signing_key() -> SigningKey {
    SigningKey::from_bytes(&TEST_KEY_SEED)
}

/// Build the corresponding `VerifyingKey`.
pub fn test_verifying_key() -> VerifyingKey {
    test_signing_key().verifying_key()
}

/// The signer ID used in test packages.
pub const TEST_SIGNER_ID: &str = "test-signer-001";

/// Build a `TrustAnchor` containing the test verifying key.
pub fn test_trust_anchor() -> TrustAnchor {
    TrustAnchor {
        signing_key_id: TEST_SIGNER_ID.to_string(),
        public_key_bytes: test_verifying_key().to_bytes().to_vec(),
        added_at: 0,
        status: TrustAnchorStatus::Active,
    }
}

/// Build a minimal signed `.nmp` package for testing.
///
/// `module_id`: e.g. `"shield-water-purification"`
/// `objects`: list of `(path, data)` pairs
pub fn build_test_package(
    module_id: &str,
    objects: Vec<(&str, Vec<u8>)>,
) -> ContentResult<Vec<u8>> {
    let manifest = PackageManifest {
        module_id: module_id.to_string(),
        version: ModuleVersion::new(1, 0, 0),
        min_core_version: ModuleVersion::new(0, 3, 0),
        signer_id: TEST_SIGNER_ID.to_string(),
        objects: vec![], // filled by NmpWriter::add_object
        locales: vec!["en".to_string()],
    };

    let mut writer = NmpWriter::new(manifest);
    for (path, data) in objects {
        writer.add_object(path, data, ObjectType::Json);
    }

    let sk = test_signing_key();
    writer.build(|manifest_bytes| {
        let sig = sk.sign(manifest_bytes);
        sig.to_bytes()
    })
}

/// Build a package signed with a key that is NOT in the trust store.
pub fn build_untrusted_package(module_id: &str) -> ContentResult<Vec<u8>> {
    // Use a different seed so the key doesn't match TEST_KEY_SEED.
    let bad_seed = [0xFFu8; 32];
    let bad_sk = SigningKey::from_bytes(&bad_seed);

    let manifest = PackageManifest {
        module_id: module_id.to_string(),
        version: ModuleVersion::new(1, 0, 0),
        min_core_version: ModuleVersion::new(0, 3, 0),
        signer_id: TEST_SIGNER_ID.to_string(), // claims to be test-signer but isn't
        objects: vec![],
        locales: vec!["en".to_string()],
    };

    let mut writer = NmpWriter::new(manifest);
    writer.add_object("dummy.json", b"{}".to_vec(), ObjectType::Json);

    writer.build(|manifest_bytes| {
        let sig = bad_sk.sign(manifest_bytes);
        sig.to_bytes()
    })
}

/// Build a package where one object's data was tampered after signing.
pub fn build_tampered_package(module_id: &str) -> ContentResult<Vec<u8>> {
    let mut pkg = build_test_package(
        module_id,
        vec![("data.json", b"{\"v\":1}".to_vec())],
    )?;

    // Flip some bytes in the last object's data section (after all headers).
    let len = pkg.len();
    if len >= 4 {
        pkg[len - 2] ^= 0xFF;
        pkg[len - 1] ^= 0xFF;
    }
    Ok(pkg)
}
