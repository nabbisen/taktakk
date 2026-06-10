//! Database connection management and migration runner.
//!
//! Each DDL statement is executed individually; sqlx::query()
//! accepts only a single statement at a time.

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::{StorageError, StorageResult};

pub struct Database {
    pub facade: SqlitePool,
    pub core:   SqlitePool,
}

impl Database {
    pub async fn open(base_dir: &Path) -> StorageResult<Self> {
        std::fs::create_dir_all(base_dir)?;
        let facade = open_pool(base_dir.join("facade.sqlite")).await?;
        let core   = open_pool(base_dir.join("core.sqlite")).await?;
        run_facade_migrations(&facade).await?;
        run_core_migrations(&core).await?;
        recover_staging_dirs(base_dir);
        Ok(Self { facade, core })
    }
}

async fn open_pool(path: PathBuf) -> StorageResult<SqlitePool> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .map_err(StorageError::Database)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);
    let pool = SqlitePool::connect_with(options).await.map_err(StorageError::Database)?;
    sqlx::query("PRAGMA page_size = 4096")
        .execute(&pool).await.map_err(StorageError::Database)?;
    Ok(pool)
}

/// Execute a list of DDL statements in one transaction.
async fn exec_ddl(pool: &SqlitePool, stmts: &[&str]) -> StorageResult<()> {
    let mut tx = pool.begin().await.map_err(StorageError::Database)?;
    for stmt in stmts {
        sqlx::query(stmt).execute(&mut *tx).await.map_err(StorageError::Database)?;
    }
    tx.commit().await.map_err(StorageError::Database)?;
    Ok(())
}

async fn run_facade_migrations(pool: &SqlitePool) -> StorageResult<()> {
    exec_ddl(pool, &[
        "CREATE TABLE IF NOT EXISTS clock_settings (
            key TEXT PRIMARY KEY NOT NULL, value TEXT NOT NULL)",
        "CREATE TABLE IF NOT EXISTS alarm_entries (
            alarm_id TEXT PRIMARY KEY NOT NULL,
            hour INTEGER NOT NULL, minute INTEGER NOT NULL,
            label TEXT, enabled INTEGER NOT NULL DEFAULT 1,
            repeat_days INTEGER NOT NULL DEFAULT 0, created_at INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS slot_config (
            slot_id TEXT PRIMARY KEY NOT NULL, alg_id TEXT NOT NULL,
            alg_params TEXT NOT NULL, salt_blob BLOB NOT NULL,
            verifier_blob BLOB NOT NULL, key_ref TEXT,
            fail_count INTEGER NOT NULL DEFAULT 0,
            ts_created INTEGER NOT NULL, ts_updated INTEGER NOT NULL)",
        "CREATE TABLE IF NOT EXISTS key_registry (
            key_id TEXT PRIMARY KEY NOT NULL, purpose_tag TEXT NOT NULL,
            wrapped_blob BLOB NOT NULL, alg_tag TEXT NOT NULL,
            ts_created INTEGER NOT NULL, ts_rotated INTEGER, state_tag TEXT NOT NULL)",
    ]).await
}

async fn run_core_migrations(pool: &SqlitePool) -> StorageResult<()> {
    exec_ddl(pool, &[
        // Profiles
        "CREATE TABLE IF NOT EXISTS local_profiles (
            profile_id TEXT PRIMARY KEY NOT NULL, display_alias TEXT,
            locale TEXT, created_at INTEGER NOT NULL, last_active_at INTEGER)",
        // Packages
        "CREATE TABLE IF NOT EXISTS content_packages (
            package_id TEXT PRIMARY KEY NOT NULL, module_id TEXT NOT NULL,
            version_major INTEGER NOT NULL, version_minor INTEGER NOT NULL,
            version_patch INTEGER NOT NULL, manifest_hash TEXT NOT NULL,
            status TEXT NOT NULL, installed_at INTEGER, quarantine_reason TEXT)",
        "CREATE TABLE IF NOT EXISTS trust_anchors (
            signing_key_id   TEXT PRIMARY KEY NOT NULL,
            public_key_bytes BLOB NOT NULL,
            scope            TEXT NOT NULL DEFAULT 'content',
            added_at         INTEGER NOT NULL,
            status           TEXT NOT NULL,
            revoked_at       INTEGER
        )",
        // Curriculum
        "CREATE TABLE IF NOT EXISTS modules (
            module_id TEXT PRIMARY KEY NOT NULL, category_id TEXT NOT NULL,
            title_key TEXT NOT NULL, description_key TEXT NOT NULL,
            version_major INTEGER NOT NULL, version_minor INTEGER NOT NULL,
            version_patch INTEGER NOT NULL, status TEXT NOT NULL,
            estimated_minutes INTEGER, package_id TEXT)",
        "CREATE TABLE IF NOT EXISTS lessons (
            lesson_id TEXT PRIMARY KEY NOT NULL, module_id TEXT NOT NULL,
            title_key TEXT NOT NULL, sort_order INTEGER NOT NULL,
            step_count INTEGER NOT NULL DEFAULT 0,
            FOREIGN KEY (module_id) REFERENCES modules(module_id))",
        "CREATE TABLE IF NOT EXISTS lesson_steps (
            step_id TEXT PRIMARY KEY NOT NULL, lesson_id TEXT NOT NULL,
            sort_order INTEGER NOT NULL, content_type TEXT NOT NULL,
            content_obj_hash TEXT, text_key TEXT,
            FOREIGN KEY (lesson_id) REFERENCES lessons(lesson_id))",
        "CREATE INDEX IF NOT EXISTS lessons_module ON lessons (module_id, sort_order)",
        "CREATE INDEX IF NOT EXISTS lesson_steps_lesson ON lesson_steps (lesson_id, sort_order)",
        // Progress
        "CREATE TABLE IF NOT EXISTS resume_state (
            profile_id TEXT NOT NULL, lesson_id TEXT NOT NULL,
            last_completed_step_order INTEGER NOT NULL, updated_at INTEGER NOT NULL,
            PRIMARY KEY (profile_id, lesson_id))",
        "CREATE TABLE IF NOT EXISTS lesson_progress (
            profile_id TEXT NOT NULL, lesson_id TEXT NOT NULL,
            status TEXT NOT NULL, steps_completed INTEGER NOT NULL DEFAULT 0,
            steps_total INTEGER NOT NULL DEFAULT 0, started_at INTEGER NOT NULL,
            completed_at INTEGER, PRIMARY KEY (profile_id, lesson_id))",
        "CREATE TABLE IF NOT EXISTS learning_sessions (
            session_id TEXT PRIMARY KEY NOT NULL, profile_id TEXT NOT NULL,
            started_at INTEGER NOT NULL, ended_at INTEGER,
            FOREIGN KEY (profile_id) REFERENCES local_profiles(profile_id))",
        "CREATE TABLE IF NOT EXISTS exercise_attempts (
            attempt_id TEXT PRIMARY KEY NOT NULL, profile_id TEXT NOT NULL,
            step_id TEXT NOT NULL, correct INTEGER NOT NULL,
            attempt_number INTEGER NOT NULL, attempted_at INTEGER NOT NULL,
            FOREIGN KEY (profile_id) REFERENCES local_profiles(profile_id))",
        "CREATE INDEX IF NOT EXISTS learning_sessions_profile ON learning_sessions (profile_id)",
        "CREATE INDEX IF NOT EXISTS exercise_attempts_profile ON exercise_attempts (profile_id, step_id)",
        // Content objects
        "CREATE TABLE IF NOT EXISTS content_objects (
            object_hash TEXT PRIMARY KEY NOT NULL, object_type TEXT NOT NULL,
            storage_uri TEXT NOT NULL, byte_size INTEGER NOT NULL,
            compression TEXT, created_at INTEGER NOT NULL, last_accessed_at INTEGER)",
        "CREATE TABLE IF NOT EXISTS package_objects (
            package_id TEXT NOT NULL, object_hash TEXT NOT NULL,
            role TEXT NOT NULL, required INTEGER NOT NULL DEFAULT 1, sort_order INTEGER,
            PRIMARY KEY (package_id, object_hash),
            FOREIGN KEY (package_id) REFERENCES content_packages(package_id),
            FOREIGN KEY (object_hash) REFERENCES content_objects(object_hash))",
        "CREATE TABLE IF NOT EXISTS integrity_checks (
            check_id TEXT PRIMARY KEY NOT NULL, object_hash TEXT NOT NULL,
            check_result TEXT NOT NULL, detail TEXT, checked_at INTEGER NOT NULL)",
        // Event log
        "CREATE TABLE IF NOT EXISTS event_log (
            event_id         TEXT PRIMARY KEY NOT NULL,
            event_tag        TEXT NOT NULL,
            ts               INTEGER NOT NULL,
            retention_until  INTEGER NOT NULL,
            detail_json      TEXT
        )",
        "CREATE INDEX IF NOT EXISTS event_log_ts ON event_log (ts)",
        // ── Sync & import ─────────────────────────────────────────────────
        "CREATE TABLE IF NOT EXISTS sync_sessions (
            sync_session_id     TEXT PRIMARY KEY NOT NULL,
            transport           TEXT NOT NULL,
            peer_ephemeral_hash TEXT,
            started_at          INTEGER NOT NULL,
            ended_at            INTEGER,
            status              TEXT NOT NULL,
            packages_received   INTEGER NOT NULL DEFAULT 0,
            packages_sent       INTEGER NOT NULL DEFAULT 0,
            retention_until     INTEGER
        )",
        "CREATE TABLE IF NOT EXISTS sync_manifest_items (
            sync_session_id      TEXT NOT NULL,
            package_id           TEXT NOT NULL,
            local_manifest_hash  TEXT,
            remote_manifest_hash TEXT,
            action               TEXT NOT NULL,
            status               TEXT NOT NULL,
            PRIMARY KEY (sync_session_id, package_id),
            FOREIGN KEY (sync_session_id) REFERENCES sync_sessions(sync_session_id)
        )",
        "CREATE TABLE IF NOT EXISTS transfer_chunks (
            transfer_id  TEXT NOT NULL,
            object_hash  TEXT NOT NULL,
            chunk_index  INTEGER NOT NULL,
            chunk_hash   TEXT NOT NULL,
            byte_size    INTEGER NOT NULL,
            status       TEXT NOT NULL,
            updated_at   INTEGER NOT NULL,
            PRIMARY KEY (transfer_id, chunk_index)
        )",
        "CREATE TABLE IF NOT EXISTS import_jobs (
            import_job_id     TEXT PRIMARY KEY NOT NULL,
            source_kind       TEXT NOT NULL,
            source_label_hash TEXT,
            started_at        INTEGER NOT NULL,
            completed_at      INTEGER,
            status            TEXT NOT NULL,
            found_count       INTEGER NOT NULL DEFAULT 0,
            installed_count   INTEGER NOT NULL DEFAULT 0
        )",
        "CREATE TABLE IF NOT EXISTS import_job_items (
            import_job_id          TEXT NOT NULL,
            package_id             TEXT NOT NULL,
            detected_manifest_hash TEXT NOT NULL,
            verify_result          TEXT NOT NULL,
            install_result         TEXT,
            PRIMARY KEY (import_job_id, package_id),
            FOREIGN KEY (import_job_id) REFERENCES import_jobs(import_job_id)
        )",
    ]).await?;

    // Post-migration integrity check (RFC 006).
    let (result,): (String,) = sqlx::query_as("PRAGMA integrity_check")
        .fetch_one(pool).await.map_err(StorageError::Database)?;
    if result != "ok" {
        return Err(StorageError::Migration(format!("integrity_check failed: {result}")));
    }
    Ok(())
}

/// On startup, remove any orphaned `staging/<install_id>/` directories
/// that were left by an interrupted install (RFC-040 crash recovery).
///
/// Objects in staging are unverified or partially written. Removing them
/// is safe — the install will need to be retried from the beginning.
fn recover_staging_dirs(base_dir: &std::path::Path) {
    let staging_root = base_dir.join("objects").join("staging");
    if !staging_root.exists() {
        return;
    }
    if let Ok(entries) = std::fs::read_dir(&staging_root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let _ = std::fs::remove_dir_all(entry.path());
            }
        }
    }
}
