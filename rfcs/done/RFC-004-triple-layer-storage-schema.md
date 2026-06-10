# RFC-004: Triple-layer storage schema

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M2 |
| **Priority** | — |

## Summary

facade.sqlite (clock, keys), core.sqlite (curriculum, progress, sync), object_store/ (CAS filesystem). Facade column names are clock-context-neutral.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
