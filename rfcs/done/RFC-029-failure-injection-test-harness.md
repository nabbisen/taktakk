# RFC-029: Failure injection test harness

| Field | Value |
|---|---|
| **Status** | Done |
| **Milestone** | M8 |
| **Priority** | — |

## Summary

FaultInjectingStore<S>: fail after N writes. FailureClass: CorruptMagic/TruncatedManifest/ZeroedSignature/TamperedObject/EmptyFile/RandomNoise. write_partial() power-loss sim.

## Acceptance criteria

Covered by the milestone's test suite (`cargo test`). See `CHANGELOG.md` for implementation notes.
