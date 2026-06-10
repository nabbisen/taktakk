# RFC-023: Chunk-based transfer and resume

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M6 |
| **Priority** | — |

## Summary

split_into_chunks() 64 KiB. verify_chunk() SHA-256. reassemble_chunks() with per-chunk hash check. pending_chunk_indices() for resume.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
