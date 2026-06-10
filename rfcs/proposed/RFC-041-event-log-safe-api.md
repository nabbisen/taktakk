# RFC-041: event_log Safe Boundary API

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Review finding** | Functional §9; Non-functional §4 |

## Problem

`event_log::append(pool, record)` accepts an `EventRecord` with free-text
`event_tag: String` and `detail_json: Option<String>`. Nothing in the
storage layer enforces:

- Only approved `EventBucket` tags are written.
- `detail_json` does not contain module names, file paths, peer IDs,
  profile IDs, or organisation names.
- A `retention_until` timestamp is always set.

`validate_event_tag()` exists in `taktakk-storage::wipe` but is never
called by `append()`. The security check lives in `taktakk-security::audit`
as a pure function that checks a sample of dangerous tags — not as an
enforced gate on every write.

A caller mistake can silently persist a log entry that identifies a
learning module, a file path, or a sync peer — all violations of the
zero-telemetry policy (RFC-019).

## Design

### Make `append()` private; expose typed API only

```rust
// Public API — the only way to write a log entry:
pub async fn log_event(
    pool: &SqlitePool,
    bucket: EventBucket,
    detail: Option<SafeEventDetail>,
    now: i64,
) -> StorageResult<()>;

// Private; used only by log_event():
async fn append(pool: &SqlitePool, record: &EventRecord) -> StorageResult<()>;
```

### `SafeEventDetail`

```rust
/// A log detail payload that has been verified to contain no PII or
/// domain-sensitive information.
///
/// Fields are all fixed-type numeric or enum values — no raw strings.
#[derive(Debug, Clone)]
pub struct SafeEventDetail {
    /// Optional error code (numeric only — never a message string).
    pub error_code: Option<u32>,
    /// Object count (e.g. packages received/sent in a sync).
    pub object_count: Option<u32>,
    /// Elapsed milliseconds (for performance logging).
    pub elapsed_ms: Option<u32>,
}

impl SafeEventDetail {
    pub fn to_json(&self) -> String { /* compact fixed-schema JSON */ }
}
```

No `String` fields. No free text. No `serde_json::Value`.

### Retention column

`event_log` gains a `retention_until INTEGER NOT NULL` column. `log_event()`
sets it to `now + max_age_seconds` (from `LogRetentionPolicy`).
`enforce_log_retention()` deletes rows where `retention_until < now`.

### Schema migration

```sql
ALTER TABLE event_log ADD COLUMN retention_until INTEGER;
UPDATE event_log SET retention_until = ts + 86400 WHERE retention_until IS NULL;
-- Then make it NOT NULL via a new table + copy migration.
```

### Test-only escape hatch

For unit tests that need to write arbitrary log entries:

```rust
#[cfg(test)]
pub async fn append_for_test(pool: &SqlitePool, record: &EventRecord) -> StorageResult<()> {
    append(pool, record).await
}
```

This is excluded from non-test builds via `#[cfg(test)]`.

## Acceptance criteria

1. `log_event(pool, EventBucket::SessionOpen, None, now)` succeeds.
2. Any caller that previously used `append()` directly fails to compile
   (private after this change).
3. `SafeEventDetail` has no `String` fields; the compiler rejects any
   attempt to add a raw string detail.
4. A log entry inserted via `log_event()` has a non-null `retention_until`.
5. `enforce_log_retention()` removes entries where `retention_until < now`.
6. `cargo test -p taktakk-storage -- event_log_safe` includes:
   - banned-word test: attempting to log a module name → compile error.
   - retention: insert 3 entries with different timestamps, purge, verify count.
7. The audit check in `taktakk-security::audit::check_log_tags_approved_only()`
   is updated to test the new typed API instead of a free-text sample.
