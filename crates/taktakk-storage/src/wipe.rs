//! Wipe orchestration for the storage layer.
//!
//! Three levels of erasure, in order of escalating thoroughness:
//!
//! 1. **State wipe** — remove learning progress only; keys and packages intact.
//! 2. **Hard wipe** — destroy crypto keys first, then delete all core data.
//! 3. **Factory reset** — hard wipe + reset facade to an unconfigured clock.
//!
//! All wipe operations are **idempotent**: calling them on an already-wiped
//! database must succeed without error.
//!
//! Key destruction (step 1 of hard wipe) is always attempted before any
//! slower delete operations, so power loss mid-wipe leaves data unreadable.

use rand::RngCore;
use sqlx::SqlitePool;

use crate::error::{StorageError, StorageResult};

// ── State wipe ────────────────────────────────────────────────────────────────

/// Remove all learning progress from `core.sqlite`.
///
/// Preserves: profiles, installed packages, curriculum metadata, trust anchors.
/// Removes:   resume_state, lesson_progress, learning_sessions,
///            exercise_attempts, event_log.
pub async fn state_wipe(core: &SqlitePool) -> StorageResult<()> {
    let mut tx = core.begin().await.map_err(StorageError::Database)?;

    for table in &["event_log", "exercise_attempts", "learning_sessions",
                   "lesson_progress", "resume_state"] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx).await.map_err(StorageError::Database)?;
    }

    tx.commit().await.map_err(StorageError::Database)?;
    Ok(())
}

// ── Key slot destruction ──────────────────────────────────────────────────────

/// Overwrite every active key slot in `facade.sqlite` with random bytes.
///
/// This is the "instant kill" step (RFC 018). Once the wrapped key bytes are
/// overwritten, all data in `core.sqlite` and the object store becomes
/// permanently unreadable — no slow file deletion needed for security.
///
/// Uses a 7-pass overwrite where feasible on flash storage (best-effort).
pub async fn destroy_key_slots(facade: &SqlitePool) -> StorageResult<()> {
    let mut rng = rand::thread_rng();

    // Retrieve all key IDs that are not already destroyed.
    let ids: Vec<(String,)> = sqlx::query_as(
        "SELECT key_id FROM key_registry WHERE state_tag != 'destroyed'",
    )
    .fetch_all(facade)
    .await
    .map_err(StorageError::Database)?;

    for (key_id,) in &ids {
        // 7-pass overwrite of the wrapped_blob.
        for _ in 0..7 {
            let noise: Vec<u8> = {
                let mut v = vec![0u8; 64];
                rng.fill_bytes(&mut v);
                v
            };
            sqlx::query(
                "UPDATE key_registry SET wrapped_blob = ?, state_tag = 'destroyed'
                 WHERE key_id = ?",
            )
            .bind(&noise)
            .bind(key_id)
            .execute(facade)
            .await
            .map_err(StorageError::Database)?;
        }
    }

    // Overwrite unlock slot verifier bytes.
    let slot_ids: Vec<(String,)> = sqlx::query_as("SELECT slot_id FROM slot_config")
        .fetch_all(facade).await.map_err(StorageError::Database)?;

    for (slot_id,) in &slot_ids {
        let noise: Vec<u8> = {
            let mut v = vec![0u8; 64];
            rng.fill_bytes(&mut v);
            v
        };
        sqlx::query(
            "UPDATE slot_config SET verifier_blob = ?, salt_blob = ?, alg_params = '{}' WHERE slot_id = ?",
        )
        .bind(&noise)
        .bind(&noise[..32])
        .bind(slot_id)
        .execute(facade).await.map_err(StorageError::Database)?;
    }

    Ok(())
}

// ── Hard wipe ─────────────────────────────────────────────────────────────────

/// Hard wipe: key destruction then full deletion of all core data.
///
/// Step order is critical (RFC 018):
/// 1. Overwrite key slots → data instantly unreadable.
/// 2. Delete all tables in core.sqlite.
///
/// Power loss between steps 1 and 2 is acceptable: without the keys,
/// the remaining ciphertext is computationally unrecoverable.
pub async fn hard_wipe(facade: &SqlitePool, core: &SqlitePool) -> StorageResult<()> {
    // Step 1: destroy keys (must succeed before anything else).
    destroy_key_slots(facade).await?;

    // Step 2: delete all core data.
    let mut tx = core.begin().await.map_err(StorageError::Database)?;

    for table in &[
        "event_log", "exercise_attempts", "learning_sessions",
        "lesson_progress", "resume_state",
        "integrity_checks", "package_objects", "content_objects",
        "lesson_steps", "lessons", "modules",
        "content_packages", "trust_anchors",
        "local_profiles",
    ] {
        sqlx::query(&format!("DELETE FROM {table}"))
            .execute(&mut *tx).await.map_err(StorageError::Database)?;
    }

    tx.commit().await.map_err(StorageError::Database)?;
    Ok(())
}

// ── Factory reset ─────────────────────────────────────────────────────────────

/// Factory reset: hard wipe + reset facade to an unconfigured clock state.
///
/// After this call the app presents as a brand-new, never-configured clock.
/// No confirmation dialog is shown by the caller; this is called directly
/// from the duress gesture handler.
pub async fn factory_reset(facade: &SqlitePool, core: &SqlitePool) -> StorageResult<()> {
    // Hard wipe first (keys → core data).
    hard_wipe(facade, core).await?;

    // Reset facade to factory state.
    let mut tx = facade.begin().await.map_err(StorageError::Database)?;

    sqlx::query("DELETE FROM alarm_entries")
        .execute(&mut *tx).await.map_err(StorageError::Database)?;
    sqlx::query("DELETE FROM slot_config")
        .execute(&mut *tx).await.map_err(StorageError::Database)?;
    sqlx::query("DELETE FROM key_registry")
        .execute(&mut *tx).await.map_err(StorageError::Database)?;
    sqlx::query("DELETE FROM clock_settings WHERE key != 'display_mode'")
        .execute(&mut *tx).await.map_err(StorageError::Database)?;

    tx.commit().await.map_err(StorageError::Database)?;
    Ok(())
}

// ── Log scrubbing ─────────────────────────────────────────────────────────────

/// Enforce the 24-hour log retention policy (RFC 019).
/// Returns the number of purged rows.
pub async fn enforce_log_retention(
    core: &SqlitePool,
    now: i64,
    max_age_seconds: i64,
) -> StorageResult<u64> {
    let cutoff = now - max_age_seconds;
    let result = sqlx::query("DELETE FROM event_log WHERE ts < ?")
        .bind(cutoff)
        .execute(core).await.map_err(StorageError::Database)?;
    Ok(result.rows_affected())
}

/// Validate that an event tag is an approved bucket (RFC 019).
///
/// Rejects any tag that could carry module names, profile IDs, or
/// aid-organisation identifiers.
pub fn validate_event_tag(tag: &str) -> bool {
    use taktakk_core::use_cases::safety_settings::EventBucket;
    EventBucket::is_approved(tag)
}
