# RFC-018: Panic wipe model (key-first)

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M5 |
| **Priority** | — |

## Summary

Key destruction precedes file deletion. overwrite_all_keys() 7-pass random fill. state_wipe/hard_wipe/factory_reset idempotent.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
