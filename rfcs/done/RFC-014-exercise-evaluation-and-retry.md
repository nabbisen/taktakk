# RFC-014: Exercise evaluation and retry

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M4 |
| **Priority** | — |

## Summary

evaluate() for MultipleChoice/Ordering/Acknowledge. max_attempts() per kind (3 for MC, 1 for Acknowledge).

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
