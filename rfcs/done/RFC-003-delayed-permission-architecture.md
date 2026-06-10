# RFC-003: Delayed permission architecture

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M1 |
| **Priority** | — |

## Summary

Permissions are never requested at cold start. TriggerAction → required_permissions mapping. All requests are unlocked_only = true.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
