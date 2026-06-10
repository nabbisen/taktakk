# RFC-040: Transactional Package Installation

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Depends on** | RFC-039 (streaming parser) |
| **Review finding** | Functional §6; Non-functional §11–12 |

## Problem

The current `install_package()` writes objects to `object_store`, then
returns a `ContentPackage` struct. The caller is expected to call
`repo::package::save()` separately. This means:

- Power loss between object write and `save()` leaves orphaned objects.
- The quarantine record is never persisted to `content_packages`.
- `package_objects`, `content_objects`, `modules`, and `lessons` tables
  are not populated during install — the catalog only fills if the caller
  manually populates them.
- Crash recovery cannot distinguish "partially installed" from
  "cleanly installed" packages.

## Design

### Staging area

```
object_store/
  staging/
    <install_id>/          ← UUID per install attempt
      <object_hash>        ← streamed + verified but not yet promoted
  <prefix>/
    <hash>                 ← promoted (committed) objects
  quarantine/
    <hash>                 ← failed objects
```

### `install_package_transactional`

```rust
pub async fn install_package_transactional(
    db: &Database,
    object_store: &dyn ObjectStore,
    reader: impl Read,
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    now: i64,
) -> InstallOutcome;
```

Execution steps:

```
Phase 1 — Stream + verify (no DB writes yet)
  a. Open NmpStreamReader (manifest + signature only → in memory)
  b. Verify signature against trust anchors
  c. For each object: stream → SHA-256 → write to staging/<install_id>/<hash>
  d. If any error: delete staging/<install_id>/, return Quarantined

Phase 2 — Atomic DB transaction
  BEGIN TRANSACTION
    INSERT content_packages (status='installed')
    INSERT content_objects   (for each object)
    INSERT package_objects   (join table)
    INSERT modules           (from manifest metadata)
    INSERT lessons           (from manifest lesson_index)
    INSERT lesson_steps      (from manifest steps)
  COMMIT

Phase 3 — Promote objects
  For each staging/<install_id>/<hash>:
    rename → object_store/<prefix>/<hash>
  Delete staging/<install_id>/

On any Phase 2/3 failure:
  ROLLBACK DB transaction
  Delete staging/<install_id>/
  INSERT content_packages (status='quarantined', quarantine_reason=...)
```

### Crash recovery on startup

During `Database::open()`, after migrations:

```rust
async fn recover_incomplete_installs(db: &Database, store: &dyn ObjectStore) {
    // Any package with status='pending' that has been pending > 5 min → quarantine.
    // Clean up staging/ directories whose install_id has no matching pending package.
}
```

### `ObjectStore` staging extension

```rust
pub trait ObjectStore: Send + Sync {
    // ... existing methods ...
    fn stage(&self, install_id: &str, data: &mut dyn Read) -> CoreResult<String>;
    fn promote(&self, install_id: &str, hash: &str) -> CoreResult<()>;
    fn abort_staging(&self, install_id: &str) -> CoreResult<()>;
}
```

## Acceptance criteria

1. Power loss after Phase 1 (staging written) but before Phase 2: on next
   startup, `recover_incomplete_installs()` removes orphaned staging files
   and no package record appears in `content_packages`.
2. Power loss mid-Phase 2 (DB transaction): `ROLLBACK` ensures no partial
   package record; staging is cleaned on next startup.
3. A successful install populates `content_packages`, `modules`, `lessons`,
   `content_objects`, and `package_objects` in one transaction.
4. A quarantined install writes a `content_packages` record with
   `status='quarantined'` and a non-empty `quarantine_reason`.
5. `cargo test -p taktakk-integration -- install_power_loss_recovery` uses
   `FaultInjectingStore` to simulate failure at object N, then verifies
   DB integrity on re-open.
6. Object store contains no orphaned objects after a failed install.
