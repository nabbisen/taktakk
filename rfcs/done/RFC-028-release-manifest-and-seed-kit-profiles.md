# RFC-028: Release manifest and seed kit profiles

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M8 |
| **Priority** | — |

## Summary

ReleaseManifest with git commit (40 hex), toolchain, artifacts. SeedKitProfile: Minimal(5MB)/Standard(25MB)/Full(50MB) with size budget check.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
