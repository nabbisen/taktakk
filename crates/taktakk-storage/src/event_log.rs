//! Event log with 24-hour automatic retention.
//!
//! The log records generic operational events (opens, closes, install outcomes)
//! without storing module names, user identifiers, or content details.
//! It is the only persistent log in taktakk; all other diagnostic output
//! stays in a bounded in-memory ring buffer.

use sqlx::SqlitePool;

use crate::error::StorageResult;

/// A single event record.
#[derive(Debug, Clone)]
pub struct EventRecord {
    pub event_id: String,
    /// Generic tag (e.g. `"session.start"`, `"install.ok"`, `"wipe.keys"`).
    /// Must not contain module names, user aliases, or PII.
    pub event_tag: String,
    /// Unix timestamp (seconds).
    pub ts: i64,
    /// Optional opaque JSON detail. Must be pre-redacted before storage.
    pub detail_json: Option<String>,
}

/// Append one event to the log.
pub async fn append(pool: &SqlitePool, record: &EventRecord) -> StorageResult<()> {
    sqlx::query(
        "INSERT INTO event_log (event_id, event_tag, ts, detail_json)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&record.event_id)
    .bind(&record.event_tag)
    .bind(record.ts)
    .bind(&record.detail_json)
    .execute(pool)
    .await?;
    Ok(())
}

/// Delete all events older than `retention_seconds` from `now`.
///
/// Call periodically (e.g. on session start) to enforce the 24-hour policy.
pub async fn purge_old(pool: &SqlitePool, now: i64, retention_seconds: i64) -> StorageResult<u64> {
    let cutoff = now - retention_seconds;
    let result = sqlx::query("DELETE FROM event_log WHERE ts < ?")
        .bind(cutoff)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

/// Retrieve recent events (newest first, up to `limit`).
pub async fn recent(pool: &SqlitePool, limit: i64) -> StorageResult<Vec<EventRecord>> {
    let rows = sqlx::query_as::<_, (String, String, i64, Option<String>)>(
        "SELECT event_id, event_tag, ts, detail_json
         FROM event_log
         ORDER BY ts DESC
         LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|(event_id, event_tag, ts, detail_json)| EventRecord {
            event_id,
            event_tag,
            ts,
            detail_json,
        })
        .collect())
}

/// Delete all events — used during panic wipe.
pub async fn wipe_all(pool: &SqlitePool) -> StorageResult<()> {
    sqlx::query("DELETE FROM event_log").execute(pool).await?;
    Ok(())
}
