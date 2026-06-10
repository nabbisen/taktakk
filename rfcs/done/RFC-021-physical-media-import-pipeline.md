# RFC-021: Physical media import pipeline

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M6 |
| **Priority** | — |

## Summary

scan_directory() finds .nmp without parsing. read_package_file() with max_bytes cap. source_label_hash() — path never stored raw.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
