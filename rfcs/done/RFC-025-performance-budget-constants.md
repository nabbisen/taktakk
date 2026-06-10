# RFC-025: Performance budget constants

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M7 |
| **Priority** | — |

## Summary

Budget/limit pairs for facade cold start, unlock, dashboard, step transition, resume write, SVG, audio. MAX_PACKAGE_STREAM_BYTES=50MiB.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
