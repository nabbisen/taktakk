# RFC-027: Security review checklist

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M7 |
| **Priority** | — |

## Summary

run_security_audit() → SecurityAuditReport. 17 checks: privacy, crypto, facade safety, wipe reliability, package integrity, permission timing.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
