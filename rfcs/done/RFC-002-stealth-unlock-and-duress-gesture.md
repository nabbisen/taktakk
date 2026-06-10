# RFC-002: Stealth unlock and duress gesture

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M1 |
| **Priority** | — |

## Summary

GestureParser state machine. Primary unlock: alarm magic time + long-press ≥2800 ms. Duress: separate magic time + long-press → Duress outcome. Tap-rhythm alternative unlock.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
