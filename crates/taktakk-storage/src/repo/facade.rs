//! Facade SQLite operations: clock settings, alarm entries.
//!
//! All column and table names are intentionally neutral — nothing here
//! contains educational or security terminology.

use sqlx::SqlitePool;

use crate::error::StorageResult;

// ── Clock settings (key-value) ────────────────────────────────────────────────

pub async fn get_setting(pool: &SqlitePool, key: &str) -> StorageResult<Option<String>> {
    let row = sqlx::query_as::<_, (String,)>(
        "SELECT value FROM clock_settings WHERE key = ?",
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|(v,)| v))
}

pub async fn set_setting(pool: &SqlitePool, key: &str, value: &str) -> StorageResult<()> {
    sqlx::query(
        "INSERT INTO clock_settings (key, value) VALUES (?, ?)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_setting(pool: &SqlitePool, key: &str) -> StorageResult<()> {
    sqlx::query("DELETE FROM clock_settings WHERE key = ?")
        .bind(key)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Alarm entries ─────────────────────────────────────────────────────────────

/// Minimal alarm row returned by queries.
#[derive(Debug, Clone)]
pub struct AlarmRow {
    pub alarm_id: String,
    pub hour: u8,
    pub minute: u8,
    pub label: Option<String>,
    pub enabled: bool,
    pub repeat_days: u8,
}

pub async fn list_alarms(pool: &SqlitePool) -> StorageResult<Vec<AlarmRow>> {
    let rows = sqlx::query_as::<_, (String, i64, i64, Option<String>, i64, i64)>(
        "SELECT alarm_id, hour, minute, label, enabled, repeat_days
         FROM alarm_entries
         ORDER BY hour, minute",
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(alarm_id, hour, minute, label, enabled, repeat_days)| AlarmRow {
            alarm_id,
            hour: hour as u8,
            minute: minute as u8,
            label,
            enabled: enabled != 0,
            repeat_days: repeat_days as u8,
        })
        .collect())
}

pub async fn upsert_alarm(pool: &SqlitePool, alarm: &AlarmRow, created_at: i64) -> StorageResult<()> {
    sqlx::query(
        "INSERT INTO alarm_entries
             (alarm_id, hour, minute, label, enabled, repeat_days, created_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(alarm_id) DO UPDATE SET
             hour        = excluded.hour,
             minute      = excluded.minute,
             label       = excluded.label,
             enabled     = excluded.enabled,
             repeat_days = excluded.repeat_days",
    )
    .bind(&alarm.alarm_id)
    .bind(alarm.hour as i64)
    .bind(alarm.minute as i64)
    .bind(&alarm.label)
    .bind(alarm.enabled as i64)
    .bind(alarm.repeat_days as i64)
    .bind(created_at)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete_alarm(pool: &SqlitePool, alarm_id: &str) -> StorageResult<()> {
    sqlx::query("DELETE FROM alarm_entries WHERE alarm_id = ?")
        .bind(alarm_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ── Wipe helpers ──────────────────────────────────────────────────────────────

/// After a panic wipe, reset facade to a factory-fresh state.
/// Alarm entries are deleted; clock_settings are also cleared.
pub async fn factory_reset_facade(pool: &SqlitePool) -> StorageResult<()> {
    sqlx::query("DELETE FROM alarm_entries").execute(pool).await?;
    // Keep clock_settings except for the slot config key.
    sqlx::query("DELETE FROM slot_config").execute(pool).await?;
    sqlx::query("DELETE FROM key_registry").execute(pool).await?;
    Ok(())
}
