//! Database connection management and migration runner.
//!
//! Opens SQLite databases with WAL mode and foreign key enforcement,
//! then runs embedded migrations automatically.

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::error::{StorageError, StorageResult};

/// Handles to the two SQLite databases.
pub struct Database {
    /// `facade.sqlite` — clock settings, unlock slot hashes, key slots.
    pub facade: SqlitePool,
    /// `core.sqlite` — curriculum, progress, sync history.
    pub core: SqlitePool,
}

impl Database {
    /// Open (or create) both databases at the given base directory.
    ///
    /// Runs embedded migrations automatically.
    pub async fn open(base_dir: &Path) -> StorageResult<Self> {
        std::fs::create_dir_all(base_dir)?;

        let facade = open_pool(base_dir.join("facade.sqlite")).await?;
        let core = open_pool(base_dir.join("core.sqlite")).await?;

        run_facade_migrations(&facade).await?;
        run_core_migrations(&core).await?;

        Ok(Self { facade, core })
    }
}

async fn open_pool(path: PathBuf) -> StorageResult<SqlitePool> {
    let options = SqliteConnectOptions::from_str(&format!("sqlite://{}", path.display()))
        .map_err(StorageError::Database)?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options)
        .await
        .map_err(StorageError::Database)?;

    // Enforce a page size suitable for flash storage on low-end devices.
    sqlx::query("PRAGMA page_size = 4096;")
        .execute(&pool)
        .await
        .map_err(StorageError::Database)?;

    Ok(pool)
}

/// Run migrations for `facade.sqlite`.
/// Schema uses only neutral terms — no educational content.
async fn run_facade_migrations(pool: &SqlitePool) -> StorageResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS clock_settings (
            key   TEXT PRIMARY KEY NOT NULL,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS alarm_entries (
            alarm_id     TEXT PRIMARY KEY NOT NULL,
            hour         INTEGER NOT NULL,
            minute       INTEGER NOT NULL,
            label        TEXT,
            enabled      INTEGER NOT NULL DEFAULT 1,
            repeat_days  INTEGER NOT NULL DEFAULT 0,
            created_at   INTEGER NOT NULL
        );

        -- Unlock slot: stores verifier hash only, never raw passcode.
        -- Column names are intentionally generic.
        CREATE TABLE IF NOT EXISTS slot_config (
            slot_id          TEXT PRIMARY KEY NOT NULL,
            alg_id           TEXT NOT NULL,
            alg_params       TEXT NOT NULL,
            salt_blob        BLOB NOT NULL,
            verifier_blob    BLOB NOT NULL,
            key_ref          TEXT,
            fail_count       INTEGER NOT NULL DEFAULT 0,
            ts_created       INTEGER NOT NULL,
            ts_updated       INTEGER NOT NULL
        );

        -- Crypto key slots: wrapped key only.
        CREATE TABLE IF NOT EXISTS key_registry (
            key_id       TEXT PRIMARY KEY NOT NULL,
            purpose_tag  TEXT NOT NULL,
            wrapped_blob BLOB NOT NULL,
            alg_tag      TEXT NOT NULL,
            ts_created   INTEGER NOT NULL,
            ts_rotated   INTEGER,
            state_tag    TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await
    .map_err(StorageError::Database)?;

    Ok(())
}

/// Run migrations for `core.sqlite`.
/// Protected by encryption at rest; may use domain-specific names.
async fn run_core_migrations(pool: &SqlitePool) -> StorageResult<()> {
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS local_profiles (
            profile_id      TEXT PRIMARY KEY NOT NULL,
            display_alias   TEXT,
            locale          TEXT,
            created_at      INTEGER NOT NULL,
            last_active_at  INTEGER
        );

        CREATE TABLE IF NOT EXISTS content_packages (
            package_id        TEXT PRIMARY KEY NOT NULL,
            module_id         TEXT NOT NULL,
            version_major     INTEGER NOT NULL,
            version_minor     INTEGER NOT NULL,
            version_patch     INTEGER NOT NULL,
            manifest_hash     TEXT NOT NULL,
            status            TEXT NOT NULL,
            installed_at      INTEGER,
            quarantine_reason TEXT
        );

        CREATE TABLE IF NOT EXISTS resume_state (
            profile_id                 TEXT NOT NULL,
            lesson_id                  TEXT NOT NULL,
            last_completed_step_order  INTEGER NOT NULL,
            updated_at                 INTEGER NOT NULL,
            PRIMARY KEY (profile_id, lesson_id)
        );

        CREATE TABLE IF NOT EXISTS lesson_progress (
            profile_id       TEXT NOT NULL,
            lesson_id        TEXT NOT NULL,
            status           TEXT NOT NULL,
            steps_completed  INTEGER NOT NULL DEFAULT 0,
            steps_total      INTEGER NOT NULL DEFAULT 0,
            started_at       INTEGER NOT NULL,
            completed_at     INTEGER,
            PRIMARY KEY (profile_id, lesson_id)
        );

        -- Short-retention event log: no module names, only generic IDs.
        CREATE TABLE IF NOT EXISTS event_log (
            event_id    TEXT PRIMARY KEY NOT NULL,
            event_tag   TEXT NOT NULL,
            ts          INTEGER NOT NULL,
            detail_json TEXT
        );

        CREATE INDEX IF NOT EXISTS event_log_ts ON event_log (ts);
        "#,
    )
    .execute(pool)
    .await
    .map_err(StorageError::Database)?;

    Ok(())
}
