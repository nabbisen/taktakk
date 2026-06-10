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
    use crate::event_log::{append, purge_old, recent, EventRecord};
    let (db, _dir) = open_test_db().await;
    for (i, ts) in [100i64, 200, 300].iter().enumerate() {
        append(&db.core, &EventRecord {
            event_id: format!("ev-{i}"),
            event_tag: "session.start".to_string(),
            ts: *ts,
            detail_json: None,
        }).await.unwrap();
    }
    // cutoff = 400 - 150 = 250; deletes ts=100 and ts=200
    let deleted = purge_old(&db.core, 400, 150).await.unwrap();
    assert_eq!(deleted, 2);
    let remaining = recent(&db.core, 10).await.unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].ts, 300);
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
