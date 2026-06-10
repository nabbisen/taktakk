# RFC-008: .nmp binary package format

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M3 |
| **Priority** | — |

## Summary

Wire: magic TAKT + version + manifest_len + manifest_JSON + sig_len + Ed25519_sig + obj_count + objects. Signature covers manifest bytes only.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
