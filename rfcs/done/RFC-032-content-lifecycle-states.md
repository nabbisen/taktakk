# RFC-032: Content lifecycle states

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M9 |
| **Priority** | — |

## Summary

ContentLifecycleState: Active/Deprecated{replaced_by}/Disabled{reason_key}/Quarantined. is_runnable() gate.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
