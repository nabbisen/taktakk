# RFC-007: Content-addressed object store

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M2 |
| **Priority** | — |

## Summary

FsObjectStore: SHA-256 addressed, 2-char prefix directory layout. put/get/exists/quarantine/delete.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
