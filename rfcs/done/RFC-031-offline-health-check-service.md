# RFC-031: Offline health check service

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M9 |
| **Priority** | — |

## Summary

run_static_health_checks(packages, anchors, locales, free_bytes) → HealthReport. HealthSeverity: Info/Warning/Error.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
