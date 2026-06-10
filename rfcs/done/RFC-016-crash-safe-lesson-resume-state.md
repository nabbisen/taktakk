# RFC-016: Crash-safe lesson resume state

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M4 |
| **Priority** | — |

## Summary

LessonState serialized to JSON after each completed step. next_step_order() + complete_step() + progress_fraction().

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
