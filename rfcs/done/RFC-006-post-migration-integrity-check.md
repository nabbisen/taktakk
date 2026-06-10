# RFC-006: Post-migration integrity check

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M2 |
| **Priority** | — |

## Summary

PRAGMA integrity_check after every migration batch. Fails hard if result ≠ 'ok'.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
