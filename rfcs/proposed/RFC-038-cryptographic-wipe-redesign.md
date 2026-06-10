# RFC-038: Cryptographic Wipe Redesign

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Depends on** | RFC-037 (encryption must be in place first) |
| **Review finding** | Functional §3, §7; Non-functional §2–3 |

## Problem

The current `hard_wipe()` implementation:
1. Calls `destroy_key_slots()` to 7-pass overwrite `wrapped_blob`.
2. Issues `DELETE FROM <table>` for every core table.

This is sound *only if* `core.sqlite` and `object_store` are encrypted
(RFC-037). Without encryption, step 1 overwrites a key that was never
used to protect anything — the `DELETE` statements become the sole
protection, and SQLite `DELETE` does not guarantee physical erasure
(WAL residue, deleted-but-not-vacuumed pages, journal files).

Additional gaps:
- `object_store` files are not included in `hard_wipe()`.
- `core.sqlite-wal`, `core.sqlite-shm`, staging/ and quarantine/ temp
  files are not wiped.
- `PRAGMA secure_delete` is not set.
- No measurable time budget for wipe completion.

## Design

### Wipe scope enum (extend existing)

```rust
pub enum WipeScope {
    /// Overwrite key slots only. With encryption, this is sufficient for
    /// immediate security. Subsequent DeleteAll is belt-and-suspenders.
    KeysOnly,
    /// Keys + delete core DB rows (progress, profiles, etc.).
    StateOnly,
    /// KeysOnly + full core DB row deletion + object_store file deletion.
    Full,
    /// Full + reset facade to factory state (plain unconfigured clock).
    FactoryReset,
}
```

### Wipe execution order

```
1. destroy_key_slots()          — 7-pass overwrite in facade.sqlite
2. (if Full or FactoryReset)
   a. PRAGMA secure_delete = ON in core connection
   b. DELETE all rows from sensitive tables
   c. VACUUM INTO ":memory:"    — compacts and drops WAL/SHM
   d. Delete object_store/** files
   e. Delete staging/**, quarantine/** temp files
3. (if FactoryReset)
   a. DELETE alarm_entries, slot_config, key_registry from facade
   b. Re-open facade as plain unconfigured clock
```

### Time budget

`KeysOnly` must complete in < 500 ms on target hardware (1 GB RAM, eMMC).
`FactoryReset` should complete in < 3 s. Both are measured in acceptance
tests using a `FakeClock` wall-clock mock.

### WAL and journal handling

```sql
-- Before wipe:
PRAGMA wal_checkpoint(TRUNCATE);
-- After deletion:
PRAGMA journal_mode = DELETE;   -- disable WAL
PRAGMA secure_delete = ON;
VACUUM;
```

On SQLCipher builds, all of the above operates on ciphertext; plain OS
forensics cannot recover row content even before `VACUUM`.

## Acceptance criteria

1. With RFC-037 in place: after `KeysOnly` wipe, any attempt to open
   `core.sqlite` with the original connection string returns
   `SqliteError { code: 26, message: "file is not a database" }`.
2. After `FactoryReset`, `object_store/` directory is empty.
3. After `FactoryReset`, `staging/` and `quarantine/` directories are empty.
4. `core.sqlite-wal` and `core.sqlite-shm` do not exist after wipe.
5. `KeysOnly` completes in < 500 ms (measured with FakeClock in integration test).
6. All existing wipe idempotency tests continue to pass.
7. `cargo test -p taktakk-integration -- wipe_key_erasure_makes_db_unreadable`
   passes end-to-end proof.
