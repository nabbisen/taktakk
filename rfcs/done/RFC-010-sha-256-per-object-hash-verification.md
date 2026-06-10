# RFC-010: SHA-256 per-object hash verification

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M3 |
| **Priority** | — |

## Summary

Each object hash verified against manifest entry after extraction. Mismatch → quarantine.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
