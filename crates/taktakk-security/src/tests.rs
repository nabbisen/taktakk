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

// ── Wipe: 7-pass overwrite ────────────────────────────────────────────────────

#[test]
fn seven_pass_overwrite_differs_from_original() {
    let mut slot = make_slot("k1", KeyStatus::Active);
    let original = slot.wrapped_key.clone();
    overwrite_key_slot(&mut slot);
    // After 7 passes the final value is (overwhelmingly) different from 0xAA fill.
    assert_ne!(slot.wrapped_key, original);
    assert_eq!(slot.status, KeyStatus::Destroyed);
}

#[test]
fn overwrite_all_skips_destroyed_slots() {
    let mut slots = vec![
        make_slot("k1", KeyStatus::Active),
        make_slot("k2", KeyStatus::Destroyed),
    ];
    let count = overwrite_all_keys(&mut slots);
    assert_eq!(count, 1);
    assert_eq!(slots[1].status, KeyStatus::Destroyed); // unchanged
}

// ── Log tag safety ────────────────────────────────────────────────────────────

use crate::wipe::is_safe_log_tag;

#[test]
fn approved_bucket_tags_are_safe() {
    assert!(is_safe_log_tag("s.open"));
    assert!(is_safe_log_tag("pkg.ok"));
    assert!(is_safe_log_tag("wipe.ok"));
    assert!(is_safe_log_tag("integ.fail"));
}

#[test]
fn domain_words_rejected() {
    assert!(!is_safe_log_tag("shield-water"));
    assert!(!is_safe_log_tag("module.open"));
    assert!(!is_safe_log_tag("learn.step"));
    assert!(!is_safe_log_tag("user.profile.123"));
}

#[test]
fn very_long_tag_rejected() {
    assert!(!is_safe_log_tag(&"x".repeat(25)));
}

// ── Idempotency: overwriting already-destroyed slots ─────────────────────────

#[test]
fn wipe_idempotent_on_empty_slots() {
    let mut slots: Vec<CryptoKeySlot> = vec![];
    assert_eq!(overwrite_all_keys(&mut slots), 0);
}

#[test]
fn wipe_idempotent_called_twice() {
    let mut slots = vec![make_slot("k1", KeyStatus::Active)];
    overwrite_all_keys(&mut slots);
    // Second call: slot is already destroyed, should return 0.
    let count = overwrite_all_keys(&mut slots);
    assert_eq!(count, 0);
}

// ── Security audit (M7) ───────────────────────────────────────────────────────

use crate::audit::{run_security_audit, SecurityCategory};

#[test]
fn all_security_checks_pass() {
    let report = run_security_audit();
    let failures: Vec<_> = report.checks.iter().filter(|c| !c.passed).collect();
    for f in &failures {
        eprintln!("FAILED: {} — {:?}", f.id, f.detail);
    }
    assert!(report.all_passed(), "security audit: {}", report.summary());
}

#[test]
fn no_telemetry_check_passes() {
    let report = run_security_audit();
    let check = report.checks.iter().find(|c| c.id == "no_telemetry_by_default").unwrap();
    assert!(check.passed);
}

#[test]
fn key_destruction_order_verified() {
    let report = run_security_audit();
    let check = report.checks.iter()
        .find(|c| c.id == "key_destruction_precedes_file_deletion").unwrap();
    assert!(check.passed);
}

#[test]
fn facade_neutral_naming_verified() {
    let report = run_security_audit();
    let check = report.checks.iter()
        .find(|c| c.id == "facade_table_names_are_neutral").unwrap();
    assert!(check.passed);
}

#[test]
fn quarantine_on_failure_verified() {
    let report = run_security_audit();
    let check = report.checks.iter()
        .find(|c| c.id == "quarantine_on_any_verification_failure").unwrap();
    assert!(check.passed);
}

#[test]
fn security_summary_format() {
    let report = run_security_audit();
    assert!(report.summary().contains("checks passed"));
}

#[test]
fn failures_by_category_returns_correct_subset() {
    let report = run_security_audit();
    // For a passing audit, each category should have 0 failures.
    for cat in &[
        SecurityCategory::Privacy,
        SecurityCategory::Cryptography,
        SecurityCategory::FacadeSafety,
    ] {
        let failures = report.failures_by_category(cat);
        assert!(failures.is_empty(), "unexpected failure in {:?}: {:?}", cat, failures);
    }
}
