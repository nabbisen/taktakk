# RFC-030: Storage maintenance and GC

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M8 |
| **Priority** | — |

## Summary

gc_object_store() orphan deletion. expire_quarantine() 30-day expiry. spot_check_objects() integrity sampling. MaintenanceReport.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
