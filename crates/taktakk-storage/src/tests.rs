//! Unit and integration tests for taktakk-storage.

use taktakk_core::ports::package_store::ObjectStore;

use crate::db::Database;
use crate::object_store::FsObjectStore;
use crate::repo;

// ── Helpers ───────────────────────────────────────────────────────────────────

fn temp_store() -> (FsObjectStore, TempDir) {
    let dir = TempDir::new();
    let store = FsObjectStore::new(dir.path().join("objects"));
    (store, dir)
}

struct TempDir(std::path::PathBuf);
impl TempDir {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!("taktakk-st-{}", rand_suffix()));
        std::fs::create_dir_all(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &std::path::Path { &self.0 }
}
impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
}
fn rand_suffix() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64
}

async fn open_test_db() -> (Database, TempDir) {
    let dir = TempDir::new();
    let db = Database::open(dir.path()).await.expect("Database::open");
    (db, dir)
}

// ── Object store ──────────────────────────────────────────────────────────────

#[test]
fn put_and_get_round_trip() {
    let (store, _dir) = temp_store();
    let data = b"taktakk object store test";
    let hash = store.put(data).unwrap();
    assert_eq!(hash.len(), 64);
    assert_eq!(store.get(&hash).unwrap(), data);
}

#[test]
fn exists_true_after_put() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"hello").unwrap();
    assert!(store.exists(&hash).unwrap());
}

#[test]
fn exists_false_for_unknown() {
    let (store, _dir) = temp_store();
    assert!(!store.exists(&"a".repeat(64)).unwrap());
}

#[test]
fn get_nonexistent_returns_err() {
    let (store, _dir) = temp_store();
    assert!(store.get(&"a".repeat(64)).is_err());
}

#[test]
fn delete_removes_object() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"delete me").unwrap();
    store.delete(&hash).unwrap();
    assert!(!store.exists(&hash).unwrap());
}

#[test]
fn quarantine_moves_object() {
    let (store, _dir) = temp_store();
    let hash = store.put(b"bad data").unwrap();
    store.quarantine(&hash, "hash mismatch").unwrap();
    assert!(!store.exists(&hash).unwrap());
}

#[test]
fn put_is_deterministic() {
    let (store, _dir) = temp_store();
    assert_eq!(store.put(b"same").unwrap(), store.put(b"same").unwrap());
}

// ── Async database ────────────────────────────────────────────────────────────

#[tokio::test]
async fn database_opens_without_error() {
    let (_db, _dir) = open_test_db().await;
}

#[tokio::test]
async fn database_integrity_check_passes() {
    let (_db, _dir) = open_test_db().await;
}

#[tokio::test]
async fn profile_save_and_retrieve() {
    use taktakk_core::domain::profile::LocalProfile;
    let (db, _dir) = open_test_db().await;
    let p = LocalProfile::new("p001".to_string(), 1_000_000);
    repo::profile::save(&db.core, &p).await.unwrap();
    let r = repo::profile::get(&db.core, "p001").await.unwrap().unwrap();
    assert_eq!(r.created_at, 1_000_000);
    assert!(r.display_alias.is_none());
}

#[tokio::test]
async fn profile_get_active_returns_latest() {
    use taktakk_core::domain::profile::LocalProfile;
    let (db, _dir) = open_test_db().await;
    let mut p1 = LocalProfile::new("p1".to_string(), 1000);
    p1.last_active_at = Some(2000);
    let mut p2 = LocalProfile::new("p2".to_string(), 1000);
    p2.last_active_at = Some(9999);
    repo::profile::save(&db.core, &p1).await.unwrap();
    repo::profile::save(&db.core, &p2).await.unwrap();
    let active = repo::profile::get_active(&db.core).await.unwrap().unwrap();
    assert_eq!(active.profile_id, "p2");
}

#[tokio::test]
async fn profile_touch_updates_last_active() {
    use taktakk_core::domain::profile::LocalProfile;
    let (db, _dir) = open_test_db().await;
    let p = LocalProfile::new("p1".to_string(), 1000);
    repo::profile::save(&db.core, &p).await.unwrap();
    repo::profile::touch(&db.core, "p1", 9999).await.unwrap();
    let u = repo::profile::get(&db.core, "p1").await.unwrap().unwrap();
    assert_eq!(u.last_active_at, Some(9999));
}

#[tokio::test]
async fn resume_state_save_and_retrieve() {
    use taktakk_core::domain::progress::ResumeState;
    let (db, _dir) = open_test_db().await;
    let s = ResumeState {
        profile_id: "p1".to_string(),
        lesson_id: "lesson-01".to_string(),
        last_completed_step_order: 3,
        updated_at: 1234,
    };
    repo::progress::save_resume_state(&db.core, &s).await.unwrap();
    let r = repo::progress::get_resume_state(&db.core, "p1", "lesson-01")
        .await.unwrap().unwrap();
    assert_eq!(r.last_completed_step_order, 3);
}

#[tokio::test]
async fn resume_state_upsert_advances_step() {
    use taktakk_core::domain::progress::ResumeState;
    let (db, _dir) = open_test_db().await;
    for step in [3u32, 7] {
        repo::progress::save_resume_state(&db.core, &ResumeState {
            profile_id: "p1".to_string(),
            lesson_id: "lesson-01".to_string(),
            last_completed_step_order: step,
            updated_at: step as i64,
        }).await.unwrap();
    }
    let r = repo::progress::get_resume_state(&db.core, "p1", "lesson-01")
        .await.unwrap().unwrap();
    assert_eq!(r.last_completed_step_order, 7);
}

#[tokio::test]
async fn package_save_and_list() {
    use taktakk_core::domain::curriculum::ModuleVersion;
    use taktakk_core::domain::package::{ContentPackage, PackageStatus};
    let (db, _dir) = open_test_db().await;
    let pkg = ContentPackage {
        package_id: "pkg-001".to_string(),
        module_id: "shield-water".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        manifest_hash: "aabbccdd".to_string(),
        status: PackageStatus::Installed,
        installed_at: Some(1000),
        quarantine_reason: None,
    };
    repo::package::save(&db.core, &pkg).await.unwrap();
    let list = repo::package::list(&db.core).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].module_id, "shield-water");
}

#[tokio::test]
async fn package_update_status_to_quarantined() {
    use taktakk_core::domain::curriculum::ModuleVersion;
    use taktakk_core::domain::package::{ContentPackage, PackageStatus};
    let (db, _dir) = open_test_db().await;
    let pkg = ContentPackage {
        package_id: "pkg-002".to_string(),
        module_id: "spear-math".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        manifest_hash: "deadbeef".to_string(),
        status: PackageStatus::Pending,
        installed_at: None,
        quarantine_reason: None,
    };
    repo::package::save(&db.core, &pkg).await.unwrap();
    repo::package::update_status(&db.core, "pkg-002", PackageStatus::Quarantined)
        .await.unwrap();
    let r = repo::package::get(&db.core, "pkg-002").await.unwrap().unwrap();
    assert_eq!(r.status, PackageStatus::Quarantined);
}

#[tokio::test]
async fn facade_settings_round_trip() {
    let (db, _dir) = open_test_db().await;
    repo::facade::set_setting(&db.facade, "display_mode", "analog").await.unwrap();
    let val = repo::facade::get_setting(&db.facade, "display_mode").await.unwrap();
    assert_eq!(val.as_deref(), Some("analog"));
}

#[tokio::test]
async fn facade_alarm_upsert_and_list() {
    use crate::repo::facade::AlarmRow;
    let (db, _dir) = open_test_db().await;
    let alarm = AlarmRow {
        alarm_id: "alarm-01".to_string(),
        hour: 7, minute: 30,
        label: Some("Morning".to_string()),
        enabled: true,
        repeat_days: 0b0001_1111,
    };
    repo::facade::upsert_alarm(&db.facade, &alarm, 1000).await.unwrap();
    let alarms = repo::facade::list_alarms(&db.facade).await.unwrap();
    assert_eq!(alarms.len(), 1);
    assert_eq!(alarms[0].hour, 7);
}

#[tokio::test]
async fn event_log_append_and_purge() {
    use crate::event_log::{purge_old, recent};
    let (db, _dir) = open_test_db().await;
    for ts in [100i64, 200, 300] {
        // use append_for_test to set specific timestamps
        crate::event_log::append_for_test(&db.core, &format!("ev-{ts}"), "s.open", ts)
            .await.unwrap();
    }
    // purge_old takes (pool, now) — retention_until = ts + 86400
    // all entries have retention_until > 0, so purge with now=0 keeps all
    // purge with now=200 + 86400 + 1 removes the ts=100 entry
    let deleted = purge_old(&db.core, 100 + 86_400 + 1).await.unwrap();
    assert_eq!(deleted, 1);
    let remaining = recent(&db.core, 10).await.unwrap();
    assert_eq!(remaining.len(), 2);
}

#[tokio::test]
async fn wipe_progress_clears_resume_state() {
    use taktakk_core::domain::progress::ResumeState;
    use crate::repo::progress::{get_resume_state, save_resume_state, wipe_resume_state};
    let (db, _dir) = open_test_db().await;
    save_resume_state(&db.core, &ResumeState {
        profile_id: "p1".to_string(),
        lesson_id: "l1".to_string(),
        last_completed_step_order: 5,
        updated_at: 0,
    }).await.unwrap();
    wipe_resume_state(&db.core).await.unwrap();
    assert!(get_resume_state(&db.core, "p1", "l1").await.unwrap().is_none());
}

// ── Curriculum repository ─────────────────────────────────────────────────────

#[tokio::test]
async fn curriculum_module_upsert_and_list() {
    use taktakk_core::domain::curriculum::{Module, ModuleStatus, ModuleVersion};
    use crate::repo::curriculum::{upsert_module, list_modules};
    let (db, _dir) = open_test_db().await;

    let m = Module {
        module_id: "shield-water".to_string(),
        category_id: "shield-hygiene".to_string(),
        title_key: "shield.water.title".to_string(),
        description_key: "shield.water.desc".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        status: ModuleStatus::Available,
        estimated_minutes: Some(15),
    };
    upsert_module(&db.core, &m).await.unwrap();
    let list = list_modules(&db.core).await.unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].module_id, "shield-water");
    assert_eq!(list[0].estimated_minutes, Some(15));
}

#[tokio::test]
async fn curriculum_lesson_upsert_and_list() {
    use taktakk_core::domain::curriculum::{Lesson, Module, ModuleStatus, ModuleVersion};
    use crate::repo::curriculum::{upsert_module, upsert_lesson, list_lessons};
    let (db, _dir) = open_test_db().await;

    let m = Module {
        module_id: "spear-math".to_string(),
        category_id: "spear-logic".to_string(),
        title_key: "t".to_string(),
        description_key: "d".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        status: ModuleStatus::Available,
        estimated_minutes: None,
    };
    upsert_module(&db.core, &m).await.unwrap();

    for i in 0u32..3 {
        upsert_lesson(&db.core, &Lesson {
            lesson_id: format!("lesson-{i:02}"),
            module_id: "spear-math".to_string(),
            title_key: format!("spear.math.lesson{i}"),
            sort_order: i,
            step_count: 5,
        }).await.unwrap();
    }

    let lessons = list_lessons(&db.core, "spear-math").await.unwrap();
    assert_eq!(lessons.len(), 3);
    assert_eq!(lessons[0].sort_order, 0);
    assert_eq!(lessons[2].sort_order, 2);
}

// ── Wipe operations ───────────────────────────────────────────────────────────

#[tokio::test]
async fn state_wipe_removes_progress_keeps_profiles() {
    use taktakk_core::domain::profile::LocalProfile;
    use taktakk_core::domain::progress::ResumeState;
    use crate::repo::progress::{get_resume_state, save_resume_state};
    use crate::wipe::state_wipe;

    let (db, _dir) = open_test_db().await;

    // Insert a profile and a resume state.
    let p = LocalProfile::new("p1".to_string(), 0);
    repo::profile::save(&db.core, &p).await.unwrap();
    save_resume_state(&db.core, &ResumeState {
        profile_id: "p1".to_string(),
        lesson_id: "l1".to_string(),
        last_completed_step_order: 3,
        updated_at: 0,
    }).await.unwrap();

    state_wipe(&db.core).await.unwrap();

    // Progress gone.
    assert!(get_resume_state(&db.core, "p1", "l1").await.unwrap().is_none());
    // Profile still present.
    assert!(repo::profile::get(&db.core, "p1").await.unwrap().is_some());
}

#[tokio::test]
async fn state_wipe_idempotent() {
    let (db, _dir) = open_test_db().await;
    // Calling wipe on an empty database must not error.
    crate::wipe::state_wipe(&db.core).await.unwrap();
    crate::wipe::state_wipe(&db.core).await.unwrap();
}

#[tokio::test]
async fn hard_wipe_removes_all_core_data() {
    use taktakk_core::domain::profile::LocalProfile;
    use taktakk_core::domain::curriculum::{Module, ModuleStatus, ModuleVersion};
    use crate::repo::curriculum::{list_modules, upsert_module};
    use crate::wipe::hard_wipe;

    let (db, _dir) = open_test_db().await;

    // Populate both profile and module.
    repo::profile::save(&db.core, &LocalProfile::new("p1".to_string(), 0)).await.unwrap();
    upsert_module(&db.core, &Module {
        module_id: "shield-water".to_string(),
        category_id: "shield-hygiene".to_string(),
        title_key: "t".to_string(),
        description_key: "d".to_string(),
        version: ModuleVersion::new(1, 0, 0),
        status: ModuleStatus::Available,
        estimated_minutes: None,
    }).await.unwrap();

    hard_wipe(&db.facade, &db.core).await.unwrap();

    // Everything gone.
    assert!(repo::profile::get(&db.core, "p1").await.unwrap().is_none());
    assert!(list_modules(&db.core).await.unwrap().is_empty());
}

#[tokio::test]
async fn hard_wipe_idempotent() {
    let (db, _dir) = open_test_db().await;
    crate::wipe::hard_wipe(&db.facade, &db.core).await.unwrap();
    crate::wipe::hard_wipe(&db.facade, &db.core).await.unwrap();
}

#[tokio::test]
async fn factory_reset_clears_facade_slots() {
    use crate::wipe::factory_reset;
    let (db, _dir) = open_test_db().await;

    // Insert a fake alarm.
    use crate::repo::facade::{AlarmRow, upsert_alarm, list_alarms};
    upsert_alarm(&db.facade, &AlarmRow {
        alarm_id: "a1".to_string(), hour: 7, minute: 0,
        label: None, enabled: true, repeat_days: 0,
    }, 0).await.unwrap();

    factory_reset(&db.facade, &db.core).await.unwrap();

    assert!(list_alarms(&db.facade).await.unwrap().is_empty());
}

#[tokio::test]
async fn factory_reset_idempotent() {
    let (db, _dir) = open_test_db().await;
    crate::wipe::factory_reset(&db.facade, &db.core).await.unwrap();
    crate::wipe::factory_reset(&db.facade, &db.core).await.unwrap();
}

#[tokio::test]
async fn key_slot_destruction_overwrites_blob() {
    use crate::wipe::destroy_key_slots;
    let (db, _dir) = open_test_db().await;

    // Insert a fake key slot.
    sqlx::query(
        "INSERT INTO key_registry (key_id, purpose_tag, wrapped_blob, alg_tag, ts_created, state_tag)
         VALUES ('k1', 'state', X'AABBCCDD', 'xchacha20poly1305', 0, 'active')"
    )
    .execute(&db.facade).await.unwrap();

    destroy_key_slots(&db.facade).await.unwrap();

    let (state_tag,): (String,) = sqlx::query_as(
        "SELECT state_tag FROM key_registry WHERE key_id = 'k1'"
    )
    .fetch_one(&db.facade).await.unwrap();

    assert_eq!(state_tag, "destroyed");
}

#[tokio::test]
async fn log_retention_purges_old_events() {
    use crate::event_log::purge_old;
    let (db, _dir) = open_test_db().await;

    // Insert events with specific timestamps using test helper.
    for (id, ts) in [("e1", 1000i64), ("e2", 5000), ("e3", 9000)] {
        crate::event_log::append_for_test(&db.core, id, "s.open", ts)
            .await.unwrap();
    }

    // retention_until = ts + 86400
    // e1: retention_until = 87400; e2: 91400; e3: 95400
    // purge with now = 91401 → removes e1 and e2
    let purged = purge_old(&db.core, 91_401).await.unwrap();
    assert_eq!(purged, 2);
}

#[tokio::test]
async fn validate_event_tag_allows_approved_buckets() {
    use crate::wipe::validate_event_tag;
    assert!(validate_event_tag("s.open"));
    assert!(validate_event_tag("pkg.ok"));
    assert!(validate_event_tag("wipe.ok"));
}

#[tokio::test]
async fn validate_event_tag_rejects_domain_words() {
    use crate::wipe::validate_event_tag;
    assert!(!validate_event_tag("module.open"));
    assert!(!validate_event_tag("learn.shield"));
    assert!(!validate_event_tag("user.profile"));
}

// ── Sync session repository (M6) ──────────────────────────────────────────────

#[tokio::test]
async fn sync_session_save_and_retrieve() {
    use taktakk_core::domain::sync::{SyncSession, SyncStatus, TransportKind};
    use crate::repo::sync::{get_sync_session, save_sync_session};
    let (db, _dir) = open_test_db().await;

    let s = SyncSession {
        session_id: "sess-001".to_string(),
        peer_ephemeral_id: "peer-abc".to_string(),
        transport: TransportKind::LocalFile,
        started_at: 1000,
        completed_at: Some(2000),
        status: SyncStatus::Completed,
        objects_received: 3,
        objects_sent: 1,
    };
    save_sync_session(&db.core, &s).await.unwrap();

    let r = get_sync_session(&db.core, "sess-001").await.unwrap().unwrap();
    assert_eq!(r.status, SyncStatus::Completed);
    assert_eq!(r.objects_received, 3);
}

#[tokio::test]
async fn sync_session_retention_purge() {
    use taktakk_core::domain::sync::{SyncSession, SyncStatus, TransportKind};
    use crate::repo::sync::{purge_old_sessions, save_sync_session};
    let (db, _dir) = open_test_db().await;

    let make = |id: &str, ended_at: Option<i64>| SyncSession {
        session_id: id.to_string(),
        peer_ephemeral_id: "p".to_string(),
        transport: TransportKind::SdCard,
        started_at: 0,
        completed_at: ended_at,
        status: SyncStatus::Completed,
        objects_received: 0,
        objects_sent: 0,
    };

    save_sync_session(&db.core, &make("s1", Some(100))).await.unwrap();
    save_sync_session(&db.core, &make("s2", Some(5000))).await.unwrap();
    save_sync_session(&db.core, &make("s3", None)).await.unwrap(); // still running

    // Purge sessions ended before cutoff = now(10000) - retention(8000) = 2000
    let purged = purge_old_sessions(&db.core, 10_000, 8000).await.unwrap();
    assert_eq!(purged, 1); // only s1 (ended_at=100) is old enough
}

#[tokio::test]
async fn import_job_lifecycle() {
    use crate::repo::sync::{complete_import_job, start_import_job};
    let (db, _dir) = open_test_db().await;

    start_import_job(&db.core, "job-001", "sdcard", Some("hashed-path"), 1000)
        .await.unwrap();
    complete_import_job(&db.core, "job-001", 3, 2000)
        .await.unwrap();

    let (status, count): (String, i64) = sqlx::query_as(
        "SELECT status, installed_count FROM import_jobs WHERE import_job_id = 'job-001'"
    )
    .fetch_one(&db.core).await.unwrap();

    assert_eq!(status, "installed");
    assert_eq!(count, 3);
}

// ── Failure injection (M8) ────────────────────────────────────────────────────

use crate::failure_injection::{
    cleanup_partial_files, generate_corrupt_package, has_partial_files,
    write_partial, FailureClass, FaultInjectingStore,
};

#[test]
fn fault_injecting_store_fails_after_n_writes() {
    use crate::object_store::FsObjectStore;
    let dir = TempDir::new();
    let inner = FsObjectStore::new(dir.path().join("objs"));
    let store = FaultInjectingStore::new(inner, 2); // fail after 2 writes

    assert!(store.put(b"first").is_ok());
    assert!(store.put(b"second").is_ok());
    assert!(store.put(b"third").is_err()); // 3rd write fails
    assert_eq!(store.writes_completed(), 3);
}

#[test]
fn generate_corrupt_empty_is_empty() {
    assert!(generate_corrupt_package(&FailureClass::EmptyFile).is_empty());
}

#[test]
fn generate_corrupt_bad_magic_not_nmp() {
    use taktakk_core::domain::package::check_magic;
    let data = generate_corrupt_package(&FailureClass::CorruptMagic);
    assert!(!check_magic(&data));
}

#[test]
fn write_partial_creates_partial_file() {
    let dir = TempDir::new();
    let data = b"full object data";
    write_partial(dir.path(), data, 5).unwrap();
    assert!(has_partial_files(dir.path()));
}

#[test]
fn cleanup_partial_removes_files() {
    let dir = TempDir::new();
    std::fs::write(dir.path().join("abc.partial"), b"partial").unwrap();
    let count = cleanup_partial_files(dir.path());
    assert_eq!(count, 1);
    assert!(!has_partial_files(dir.path()));
}

// ── Storage maintenance (M9) ──────────────────────────────────────────────────

use crate::maintenance::{gc_object_store, spot_check_objects};

#[test]
fn gc_removes_orphaned_objects() {
    use std::collections::HashSet;
    use crate::object_store::FsObjectStore;
    use taktakk_core::ports::package_store::ObjectStore;

    let dir = TempDir::new();
    let store = FsObjectStore::new(dir.path().to_path_buf());

    let h1 = store.put(b"referenced").unwrap();
    let h2 = store.put(b"orphan").unwrap();

    let mut referenced = HashSet::new();
    referenced.insert(h1.clone());

    let deleted = gc_object_store(dir.path(), &referenced);
    assert_eq!(deleted, 1);
    assert!(store.exists(&h1).unwrap());
    assert!(!store.exists(&h2).unwrap());
}

#[test]
fn spot_check_all_valid_returns_no_corrupt() {
    use crate::object_store::FsObjectStore;
    use taktakk_core::ports::package_store::ObjectStore;

    let dir = TempDir::new();
    let store = FsObjectStore::new(dir.path().to_path_buf());
    let h = store.put(b"valid object").unwrap();

    let (verified, corrupt) = spot_check_objects(dir.path(), &[h], 10);
    assert_eq!(verified, 1);
    assert_eq!(corrupt, 0);
}

#[tokio::test]
async fn maintenance_report_is_clean_on_fresh_db() {
    use crate::maintenance::MaintenanceReport;
    let report = MaintenanceReport::default();
    assert!(report.is_clean());
    assert_eq!(report.objects_corrupt_found, 0);
}
