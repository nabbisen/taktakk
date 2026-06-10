# RFC-015: RTL/LTR navigation mirroring

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M4 |
| **Priority** | — |

## Summary

NavigationArrows::for_direction(). Must Mirror: arrows, chevrons, progress bars. Must Not Mirror: emergency icons, water/medical pictograms.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
