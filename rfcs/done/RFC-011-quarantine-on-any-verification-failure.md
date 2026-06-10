# RFC-011: Quarantine on any verification failure

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M3 |
| **Priority** | — |

## Summary

install_package() returns InstallOutcome::Quarantined{reason} on any failure. No partial install.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
