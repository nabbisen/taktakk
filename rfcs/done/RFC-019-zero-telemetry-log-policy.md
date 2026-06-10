# RFC-019: Zero-telemetry log policy

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M5 |
| **Priority** | — |

## Summary

10 approved EventBucket tags only. 24-hour retention. validate_event_tag() rejects domain words.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
