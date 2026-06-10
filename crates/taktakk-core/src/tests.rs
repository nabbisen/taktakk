//! Unit tests for taktakk-core.

use crate::domain::{
    curriculum::{CurriculumAxis, ModuleVersion},
    package::{check_magic, NMP_MAGIC, NMP_FORMAT_VERSION},
};
use crate::use_cases::{
    panic_wipe::{WipeScope},
    resume_learning::resolve_resume_point,
    start_sync::{build_local_inventory, plan_download},
    verify_package::{check_nmp_header, verify_object_hash},
};
use crate::error::CoreResult;
use crate::ports::{
    crypto::{HashProvider, WipeCoordinator},
    storage::ProgressRepository,
};
use crate::domain::progress::{ExerciseAttempt, LearningSession, LessonProgress, ResumeState};
use std::sync::atomic::{AtomicBool, Ordering};

// --- Stubs for port traits used in tests ---

struct FakeHasher;
impl HashProvider for FakeHasher {
    fn sha256_hex(&self, data: &[u8]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        data.hash(&mut h);
        format!("{:016x}", h.finish())
    }
}

struct FakeWipeCoordinator { pub destroyed: AtomicBool }
impl FakeWipeCoordinator {
    fn new() -> Self { Self { destroyed: AtomicBool::new(false) } }
}
impl WipeCoordinator for FakeWipeCoordinator {
    fn destroy_keys(&self) -> CoreResult<()> {
        self.destroyed.store(true, Ordering::SeqCst);
        Ok(())
    }
}

struct FakeProgressRepo {
    pub resume_state: Option<ResumeState>,
}
impl ProgressRepository for FakeProgressRepo {
    fn save_resume_state(&self, _state: &ResumeState) -> CoreResult<()> { Ok(()) }
    fn get_resume_state(&self, _profile_id: &str, _lesson_id: &str)
        -> CoreResult<Option<ResumeState>>
    {
        Ok(self.resume_state.clone())
    }
    fn save_lesson_progress(&self, _p: &LessonProgress) -> CoreResult<()> { Ok(()) }
    fn get_lesson_progress(&self, _profile_id: &str, _lesson_id: &str)
        -> CoreResult<Option<LessonProgress>> { Ok(None) }
    fn save_exercise_attempt(&self, _a: &ExerciseAttempt) -> CoreResult<()> { Ok(()) }
    fn save_session(&self, _s: &LearningSession) -> CoreResult<()> { Ok(()) }
    fn end_session(&self, _id: &str, _ended_at: i64) -> CoreResult<()> { Ok(()) }
}

// --- Domain tests ---

#[test]
fn module_version_ordering() {
    let v1 = ModuleVersion::new(1, 0, 0);
    let v2 = ModuleVersion::new(1, 0, 1);
    let v3 = ModuleVersion::new(2, 0, 0);
    assert!(v1 < v2);
    assert!(v2 < v3);
    assert!(v1 < v3);
}

#[test]
fn module_version_display() {
    let v = ModuleVersion::new(1, 2, 3);
    assert_eq!(v.to_string(), "1.2.3");
}

#[test]
fn curriculum_axis_variants_distinct() {
    assert_ne!(CurriculumAxis::Shield, CurriculumAxis::Spear);
}

// --- Package magic bytes ---

#[test]
fn check_magic_valid() {
    let mut data = NMP_MAGIC.to_vec();
    data.push(NMP_FORMAT_VERSION);
    data.push(0); // placeholder
    assert!(check_magic(&data));
}

#[test]
fn check_magic_invalid() {
    let data = b"JPEG\x01\x00".as_ref();
    assert!(!check_magic(data));
}

#[test]
fn check_magic_too_short() {
    assert!(!check_magic(&[0x54, 0x41]));
}

#[test]
fn nmp_header_valid() {
    let mut data = NMP_MAGIC.to_vec();
    data.push(NMP_FORMAT_VERSION); // format version
    data.push(0);                  // padding
    check_nmp_header(&data).expect("valid header should pass");
}

#[test]
fn nmp_header_wrong_version() {
    let mut data = NMP_MAGIC.to_vec();
    data.push(99); // unknown version
    data.push(0);
    assert!(check_nmp_header(&data).is_err());
}

// --- Hash verification ---

#[test]
fn verify_object_hash_match() {
    let hasher = FakeHasher;
    let data = b"hello taktakk";
    let expected = hasher.sha256_hex(data);
    verify_object_hash(&hasher, data, &expected)
        .expect("matching hash should pass verification");
}

#[test]
fn verify_object_hash_mismatch() {
    let hasher = FakeHasher;
    let data = b"hello taktakk";
    let wrong = "deadbeefdeadbeef";
    assert!(verify_object_hash(&hasher, data, wrong).is_err());
}

// --- Panic wipe ---

#[test]
fn panic_wipe_keys_only_destroys_keys() {
    let coord = FakeWipeCoordinator::new();
    let result = crate::use_cases::panic_wipe::execute_panic_wipe(&coord, WipeScope::KeysOnly)
        .expect("wipe should succeed");
    assert!(result.keys_destroyed);
    assert!(coord.destroyed.load(Ordering::SeqCst));
    assert_eq!(result.scope, WipeScope::KeysOnly);
}

#[test]
fn panic_wipe_full_destroys_keys() {
    let coord = FakeWipeCoordinator::new();
    let result = crate::use_cases::panic_wipe::execute_panic_wipe(&coord, WipeScope::Full)
        .expect("full wipe should succeed");
    assert!(result.keys_destroyed);
    assert_eq!(result.scope, WipeScope::Full);
}

// --- Resume learning ---

#[test]
fn resume_with_no_prior_progress_starts_at_zero() {
    let repo = FakeProgressRepo { resume_state: None };
    let point = resolve_resume_point(&repo, "profile-1", "lesson-1")
        .expect("should resolve");
    assert_eq!(point.next_step_order, 0);
}

#[test]
fn resume_after_completing_step_two_continues_at_three() {
    let repo = FakeProgressRepo {
        resume_state: Some(ResumeState {
            profile_id: "profile-1".to_string(),
            lesson_id: "lesson-1".to_string(),
            last_completed_step_order: 2,
            updated_at: 0,
        }),
    };
    let point = resolve_resume_point(&repo, "profile-1", "lesson-1")
        .expect("should resolve");
    assert_eq!(point.next_step_order, 3);
}

// --- Sync planning ---

#[test]
fn plan_download_returns_missing_items() {
    let remote = build_local_inventory(vec![
        ("pkg-a".to_string(), "1.0.0".to_string(), "aaa".to_string()),
        ("pkg-b".to_string(), "1.0.0".to_string(), "bbb".to_string()),
    ]);
    let local = build_local_inventory(vec![
        ("pkg-a".to_string(), "1.0.0".to_string(), "aaa".to_string()),
    ]);
    let missing = plan_download(&remote, &local);
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].package_id, "pkg-b");
}

#[test]
fn plan_download_no_missing_returns_empty() {
    let items = build_local_inventory(vec![
        ("pkg-a".to_string(), "1.0.0".to_string(), "aaa".to_string()),
    ]);
    let missing = plan_download(&items, &items);
    assert!(missing.is_empty());
}

// ── Safety settings ───────────────────────────────────────────────────────────

use crate::use_cases::safety_settings::{
    DuressAction, EventBucket, LogRetentionPolicy, SafetySettings,
};

#[test]
fn safety_settings_defaults_are_conservative() {
    let s = SafetySettings::default();
    assert_eq!(s.duress_action, DuressAction::CryptoErase);
    assert!(s.log_policy.purge_on_start);
    assert!(s.log_policy.anonymise_tags);
    assert_eq!(s.log_policy.max_age_seconds, 86_400);
}

#[test]
fn log_retention_policy_valid_range() {
    let ok = LogRetentionPolicy { max_age_seconds: 3600, ..Default::default() };
    assert!(ok.validate().is_ok());
    let too_short = LogRetentionPolicy { max_age_seconds: 100, ..Default::default() };
    assert!(too_short.validate().is_err());
    let too_long = LogRetentionPolicy { max_age_seconds: 700_000, ..Default::default() };
    assert!(too_long.validate().is_err());
}

#[test]
fn event_bucket_tags_are_approved() {
    for bucket in &[
        EventBucket::SessionOpen, EventBucket::SessionClose,
        EventBucket::InstallOk, EventBucket::InstallFail,
        EventBucket::WipeOk, EventBucket::SyncOk, EventBucket::SyncFail,
        EventBucket::ImportOk, EventBucket::ImportFail, EventBucket::IntegrityFail,
    ] {
        assert!(EventBucket::is_approved(bucket.tag()),
            "bucket {:?} tag '{}' should be approved", bucket, bucket.tag());
    }
}

#[test]
fn unknown_event_tag_is_rejected() {
    assert!(!EventBucket::is_approved("shield-water-purification"));
    assert!(!EventBucket::is_approved("user.action"));
    assert!(!EventBucket::is_approved("module.open"));
}

#[test]
fn state_wipe_scope_variants_distinct() {
    use crate::use_cases::safety_settings::StateWipeScope;
    assert_ne!(StateWipeScope::ProgressOnly, StateWipeScope::ProfilesAndProgress);
    assert_ne!(StateWipeScope::ProfilesAndProgress, StateWipeScope::AllUserData);
}
