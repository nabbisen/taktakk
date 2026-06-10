//! Unit tests for taktakk-security.

use crate::hash::sha256_hex;
use crate::key_slot::{CryptoKeySlot, KeyPurpose, KeyStatus};
use crate::wipe::{overwrite_all_keys, overwrite_key_slot, redact};

fn make_slot(id: &str, status: KeyStatus) -> CryptoKeySlot {
    CryptoKeySlot {
        key_id: id.to_string(),
        purpose: KeyPurpose::State,
        wrapped_key: vec![0xAA; 32],
        alg: "xchacha20poly1305".to_string(),
        created_at: 0,
        rotated_at: None,
        status,
    }
}

// --- SHA-256 ---

#[test]
fn sha256_known_vector() {
    // SHA-256("") = e3b0c44298fc1c149afb...
    let digest = sha256_hex(b"");
    assert!(digest.starts_with("e3b0c44298fc1c14"));
    assert_eq!(digest.len(), 64);
}

#[test]
fn sha256_deterministic() {
    let a = sha256_hex(b"taktakk");
    let b = sha256_hex(b"taktakk");
    assert_eq!(a, b);
}

#[test]
fn sha256_different_input_different_hash() {
    assert_ne!(sha256_hex(b"a"), sha256_hex(b"b"));
}

// --- Key slot wipe ---

#[test]
fn overwrite_key_slot_randomises_bytes() {
    let mut slot = make_slot("k1", KeyStatus::Active);
    let original = slot.wrapped_key.clone();
    overwrite_key_slot(&mut slot);
    // After overwrite, key material should differ from the zeroed original.
    // (With overwhelming probability; could theoretically be equal by chance.)
    assert_ne!(slot.wrapped_key, original);
    assert_eq!(slot.status, KeyStatus::Destroyed);
}

#[test]
fn overwrite_all_keys_counts_correctly() {
    let mut slots = vec![
        make_slot("k1", KeyStatus::Active),
        make_slot("k2", KeyStatus::Active),
        make_slot("k3", KeyStatus::Destroyed),
    ];
    let count = overwrite_all_keys(&mut slots);
    assert_eq!(count, 2);
    assert_eq!(slots[0].status, KeyStatus::Destroyed);
    assert_eq!(slots[1].status, KeyStatus::Destroyed);
    // Already-destroyed slot stays destroyed.
    assert_eq!(slots[2].status, KeyStatus::Destroyed);
}

#[test]
fn overwrite_retired_slot() {
    let mut slots = vec![make_slot("k1", KeyStatus::Retired)];
    let count = overwrite_all_keys(&mut slots);
    assert_eq!(count, 1);
}

// --- Log redaction ---

#[test]
fn redact_removes_content() {
    let sensitive = "module: shield-first-aid, user: abc";
    let redacted = redact(sensitive);
    assert!(!redacted.contains("shield"));
    assert!(!redacted.contains("user"));
    assert_eq!(redacted.len(), sensitive.len());
}

#[test]
fn redact_preserves_newlines() {
    let s = "line1\nline2";
    let r = redact(s);
    assert!(r.contains('\n'));
}
