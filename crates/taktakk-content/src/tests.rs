//! Unit and integration tests for taktakk-content (M3).

use crate::fixtures::{
    build_tampered_package, build_test_package, build_untrusted_package, test_trust_anchor,
    TEST_SIGNER_ID,
};
use crate::install::{install_package, InstallOutcome};
use crate::nmp::reader::parse;
use crate::verify;

// ── Fake object store ─────────────────────────────────────────────────────────

use std::collections::HashMap;
use std::sync::Mutex;
use taktakk_core::error::{CoreError, CoreResult};
use taktakk_core::ports::package_store::ObjectStore;

struct FakeStore(Mutex<HashMap<String, Vec<u8>>>);
impl FakeStore {
    fn new() -> Self { Self(Mutex::new(HashMap::new())) }
}
impl ObjectStore for FakeStore {
    fn put(&self, data: &[u8]) -> CoreResult<String> {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(data));
        self.0.lock().unwrap().insert(hash.clone(), data.to_vec());
        Ok(hash)
    }
    fn get(&self, hash: &str) -> CoreResult<Vec<u8>> {
        self.0.lock().unwrap().get(hash)
            .cloned()
            .ok_or_else(|| CoreError::Storage(format!("not found: {hash}")))
    }
    fn exists(&self, hash: &str) -> CoreResult<bool> {
        Ok(self.0.lock().unwrap().contains_key(hash))
    }
    fn quarantine(&self, hash: &str, _: &str) -> CoreResult<()> {
        self.0.lock().unwrap().remove(hash);
        Ok(())
    }
    fn delete(&self, hash: &str) -> CoreResult<()> {
        self.0.lock().unwrap().remove(hash);
        Ok(())
    }
    // Staging: for FakeStore, staging and live store are the same map.
    fn stage(&self, _install_id: &str, data: &[u8]) -> CoreResult<String> {
        self.put(data)
    }
    fn commit_staging(&self, _install_id: &str) -> CoreResult<()> { Ok(()) }
    fn abort_staging(&self, _install_id: &str) -> CoreResult<()> { Ok(()) }
    fn list_staging_ids(&self) -> CoreResult<Vec<String>> { Ok(vec![]) }
}

// ── .nmp parse tests ──────────────────────────────────────────────────────────

#[test]
fn parse_valid_package() {
    let pkg = build_test_package(
        "shield-water-purification",
        vec![("lesson-01.json", b"{\"step\":1}".to_vec())],
    ).unwrap();
    let parsed = parse(&pkg).unwrap();
    assert_eq!(parsed.manifest.module_id, "shield-water-purification");
    assert_eq!(parsed.manifest.signer_id, TEST_SIGNER_ID);
    assert_eq!(parsed.objects.len(), 1);
    assert_eq!(parsed.objects[0], b"{\"step\":1}");
}

#[test]
fn parse_bad_magic_returns_error() {
    let mut pkg = build_test_package("test", vec![]).unwrap();
    pkg[0] = 0xFF;
    assert!(parse(&pkg).is_err());
}

#[test]
fn parse_wrong_version_returns_error() {
    let mut pkg = build_test_package("test", vec![]).unwrap();
    pkg[4] = 99; // format version byte
    assert!(parse(&pkg).is_err());
}

#[test]
fn parse_empty_slice_returns_error() {
    assert!(parse(&[]).is_err());
}

#[test]
fn parse_multiple_objects() {
    let pkg = build_test_package(
        "spear-math",
        vec![
            ("lesson-01.json", b"data-one".to_vec()),
            ("lesson-02.json", b"data-two".to_vec()),
            ("audio.opus",     b"audio-bytes".to_vec()),
        ],
    ).unwrap();
    let parsed = parse(&pkg).unwrap();
    assert_eq!(parsed.objects.len(), 3);
    assert_eq!(&parsed.objects[1], b"data-two");
}

// ── Signature verification ────────────────────────────────────────────────────

#[test]
fn valid_signature_passes() {
    let pkg = build_test_package(
        "shield-first-aid",
        vec![("info.json", b"{\"ok\":true}".to_vec())],
    ).unwrap();
    let parsed = parse(&pkg).unwrap();
    let anchors = vec![test_trust_anchor()];
    verify::verify_signature(&parsed, &anchors).expect("valid signature should pass");
}

#[test]
fn untrusted_signer_fails() {
    let pkg = build_untrusted_package("shield-first-aid").unwrap();
    let parsed = parse(&pkg).unwrap();
    let anchors = vec![test_trust_anchor()];
    assert!(verify::verify_signature(&parsed, &anchors).is_err());
}

#[test]
fn empty_trust_store_fails() {
    let pkg = build_test_package("test", vec![]).unwrap();
    let parsed = parse(&pkg).unwrap();
    assert!(verify::verify_signature(&parsed, &[]).is_err());
}

#[test]
fn revoked_anchor_fails() {
    use taktakk_security::trust_anchor::TrustAnchorStatus;
    let pkg = build_test_package("test", vec![]).unwrap();
    let parsed = parse(&pkg).unwrap();

    let mut anchor = test_trust_anchor();
    anchor.status = TrustAnchorStatus::Revoked;
    assert!(verify::verify_signature(&parsed, &[anchor]).is_err());
}

// ── Hash verification ─────────────────────────────────────────────────────────

#[test]
fn valid_hashes_pass() {
    let pkg = build_test_package(
        "test",
        vec![("data.json", b"hello".to_vec())],
    ).unwrap();
    let parsed = parse(&pkg).unwrap();
    verify::verify_objects(&parsed).expect("hashes should match");
}

#[test]
fn tampered_object_fails_hash_check() {
    // A tampered package: data bytes are flipped after signing.
    let pkg = build_tampered_package("test").unwrap();
    // We expect either a parse error (if struct is broken) or a hash mismatch.
    let result = parse(&pkg).and_then(|p| {
        verify::verify_objects(&p).map(|_| p)
    });
    assert!(result.is_err(), "tampered package should fail verification");
}

// ── Install pipeline ──────────────────────────────────────────────────────────

#[test]
fn install_valid_package_succeeds() {
    let pkg = build_test_package(
        "shield-water-purification",
        vec![("lesson.json", b"{\"lesson\":1}".to_vec())],
    ).unwrap();
    let store = FakeStore::new();
    let anchors = vec![test_trust_anchor()];

    let outcome = install_package(&pkg, "pkg-001", &anchors, &store, 1000);
    match outcome {
        InstallOutcome::Installed { package } => {
            assert_eq!(package.module_id, "shield-water-purification");
            assert_eq!(package.status, taktakk_core::domain::package::PackageStatus::Installed);
            // Object should be in store.
            use sha2::{Digest, Sha256};
            let expected_hash = hex::encode(Sha256::digest(b"{\"lesson\":1}"));
            assert!(store.exists(&expected_hash).unwrap());
        }
        InstallOutcome::Quarantined { reason } => {
            panic!("expected Installed, got Quarantined: {reason}");
        }
    }
}

#[test]
fn install_untrusted_package_quarantines() {
    let pkg = build_untrusted_package("shield-first-aid").unwrap();
    let store = FakeStore::new();
    let anchors = vec![test_trust_anchor()];

    let outcome = install_package(&pkg, "pkg-002", &anchors, &store, 1000);
    assert!(
        matches!(outcome, InstallOutcome::Quarantined { .. }),
        "untrusted package should be quarantined"
    );
}

#[test]
fn install_tampered_package_quarantines() {
    let pkg = build_tampered_package("spear-comm").unwrap();
    let store = FakeStore::new();
    let anchors = vec![test_trust_anchor()];

    let outcome = install_package(&pkg, "pkg-003", &anchors, &store, 1000);
    assert!(
        matches!(outcome, InstallOutcome::Quarantined { .. }),
        "tampered package should be quarantined"
    );
}

#[test]
fn install_corrupt_bytes_quarantines() {
    let store = FakeStore::new();
    let anchors = vec![test_trust_anchor()];
    let outcome = install_package(b"JUNK_DATA_THAT_IS_NOT_NMP", "pkg-004", &anchors, &store, 1000);
    assert!(matches!(outcome, InstallOutcome::Quarantined { .. }));
}

#[test]
fn install_two_packages_stored_independently() {
    let pkg1 = build_test_package("shield-a", vec![("a.json", b"aaa".to_vec())]).unwrap();
    let pkg2 = build_test_package("shield-b", vec![("b.json", b"bbb".to_vec())]).unwrap();
    let store = FakeStore::new();
    let anchors = vec![test_trust_anchor()];

    assert!(matches!(
        install_package(&pkg1, "pkg-a", &anchors, &store, 1000),
        InstallOutcome::Installed { .. }
    ));
    assert!(matches!(
        install_package(&pkg2, "pkg-b", &anchors, &store, 1000),
        InstallOutcome::Installed { .. }
    ));
}

// ── Manifest validation ───────────────────────────────────────────────────────

#[test]
fn manifest_validate_empty_module_id_fails() {
    use crate::nmp::manifest::validate_manifest;
    use taktakk_core::domain::package::PackageManifest;
    use taktakk_core::domain::curriculum::ModuleVersion;

    let m = PackageManifest {
        module_id: "".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        min_core_version: ModuleVersion::new(0, 1, 0),
        signer_id: "s1".to_string(),
        objects: vec![],
        locales: vec![],
    };
    assert!(validate_manifest(&m).is_err());
}

#[test]
fn manifest_validate_bad_hash_fails() {
    use crate::nmp::manifest::validate_manifest;
    use taktakk_core::domain::package::{ObjectEntry, ObjectType, PackageManifest};
    use taktakk_core::domain::curriculum::ModuleVersion;

    let m = PackageManifest {
        module_id: "test".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        min_core_version: ModuleVersion::new(0, 1, 0),
        signer_id: "s1".to_string(),
        objects: vec![ObjectEntry {
            path: "f.json".to_string(),
            sha256: "not-a-hash".to_string(), // too short
            object_type: ObjectType::Json,
            required: true,
        }],
        locales: vec![],
    };
    assert!(validate_manifest(&m).is_err());
}
