//! Event log — safe boundary API (RFC-041).
//!
//! The only persistent log in taktakk. Stores anonymous operational events;
//! never module names, user identifiers, file paths, or peer IDs.
//!
//! ## Public API
//!
//! - `log_event()` — the sole write entry point. Takes a typed `EventBucket`
//!   and an optional `SafeEventDetail`. Rejects at the type level any attempt
//!   to store free-form strings.
//! - `purge_old()`, `wipe_all()`, `recent()` — read/maintenance.
//!
//! `append()` is private and used only by `log_event()`. Test code may use
//! the `#[cfg(test)]` re-export `append_for_test()`.

use uuid::Uuid;
use sqlx::SqlitePool;

use taktakk_core::use_cases::safety_settings::EventBucket;

use crate::error::StorageResult;

// ── Safe event detail ─────────────────────────────────────────────────────────

/// A log detail payload with no free-text fields.
///
/// All fields are numeric. There is no `String` or `Value` field to prevent
/// module names, peer IDs, file paths, or user data from entering the log.
#[derive(Debug, Clone, Default)]
pub struct SafeEventDetail {
    /// Numeric error code — never an error message string.
    pub error_code: Option<u32>,
    /// Object count (e.g. packages transferred in a sync).
    pub object_count: Option<u32>,
    /// Elapsed time in milliseconds (performance logging only).
    pub elapsed_ms: Option<u32>,
}

impl SafeEventDetail {
    fn to_json(&self) -> String {
        let parts: Vec<String> = [
            self.error_code.map(|v| format!("\"err\":{v}")),
            self.object_count.map(|v| format!("\"n\":{v}")),
            self.elapsed_ms.map(|v| format!("\"ms\":{v}")),
        ]
        .into_iter()
        .flatten()
        .collect();

        if parts.is_empty() {
            "{}".to_string()
        } else {
            format!("{{{}}}", parts.join(","))
        }
    }
}

// ── Internal record (private) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct EventRecord {
    event_id:       String,
    event_tag:      String,
    ts:             i64,
    retention_until: i64,
    detail_json:    Option<String>,
}

/// Append one event. Private — callers must use `log_event()`.
async fn append(pool: &SqlitePool, record: &EventRecord) -> StorageResult<()> {
    sqlx::query(
        "INSERT INTO event_log (event_id, event_tag, ts, retention_until, detail_json)
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(&record.event_id)
    .bind(&record.event_tag)
    .bind(record.ts)
    .bind(record.retention_until)
    .bind(&record.detail_json)
    .execute(pool)
    .await?;
    Ok(())
}

// ── Public API ────────────────────────────────────────────────────────────────

/// The only public write path for the event log.
///
/// Accepts only approved `EventBucket` values and a `SafeEventDetail`
/// with no free-text fields. Automatically sets a `retention_until`
/// timestamp of `now + 86_400` (24 hours).
pub async fn log_event(
    pool: &SqlitePool,
    bucket: EventBucket,
    detail: Option<SafeEventDetail>,
    now: i64,
) -> StorageResult<()> {
    let record = EventRecord {
        event_id:       Uuid::new_v4().to_string(),
        event_tag:      bucket.tag().to_string(),
        ts:             now,
        retention_until: now + 86_400,
        detail_json:    detail.map(|d| d.to_json()),
    };
    append(pool, &record).await
}

/// Delete events whose `retention_until` has passed.
pub async fn purge_old(pool: &SqlitePool, now: i64) -> StorageResult<u64> {
    let result = sqlx::query("DELETE FROM event_log WHERE retention_until < ?")
        .bind(now)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Delete all events — used during panic wipe.
pub async fn wipe_all(pool: &SqlitePool) -> StorageResult<()> {
    sqlx::query("DELETE FROM event_log").execute(pool).await?;
    Ok(())
}

/// Retrieve recent events (newest first, up to `limit`).
/// Returns `(event_tag, ts, detail_json)` triples.
pub async fn recent(
    pool: &SqlitePool,
    limit: i64,
) -> StorageResult<Vec<(String, i64, Option<String>)>> {
    let rows = sqlx::query_as::<_, (String, i64, Option<String>)>(
        "SELECT event_tag, ts, detail_json
         FROM event_log
         ORDER BY ts DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

// ── Test-only escape hatch ────────────────────────────────────────────────────

#[cfg(test)]
pub async fn append_for_test(
    pool: &SqlitePool,
    event_id: &str,
    event_tag: &str,
    ts: i64,
) -> StorageResult<()> {
    append(pool, &EventRecord {
        event_id:        event_id.to_string(),
        event_tag:       event_tag.to_string(),
        ts,
        retention_until: ts + 86_400,
        detail_json:     None,
    }).await
}
