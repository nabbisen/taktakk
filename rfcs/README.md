# taktakk RFCs

Design notes for taktakk features, policies, and remediations. Each RFC
scopes one piece of work in enough detail that an implementer can start
without a second design pass — but no more than that.

RFCs are not blanket commitments. The [ROADMAP](../ROADMAP.md) sets which
will actually ship and in what order. An RFC landing here means the design
is settled enough to write code from.

---

## Status key

| Status | Meaning |
|---|---|
| `Done` | Implemented and passing in test suite |
| `Proposed` | Design settled; implementation not yet started |
| `Draft` | Design still in flux |
| `Deferred` | Intentionally postponed; reason noted |
| `Withdrawn` | Will not implement; reason noted |

---

## Done (M1–M9)

| RFC | Title | Milestone |
|---|---|---|
| [RFC-001](done/RFC-001-clock-facade-domain.md) | Clock facade domain model | M1 |
| [RFC-002](done/RFC-002-stealth-unlock-gesture.md) | Stealth unlock and duress gesture | M1 |
| [RFC-003](done/RFC-003-delayed-permission-architecture.md) | Delayed permission architecture | M1 |
| [RFC-004](done/RFC-004-triple-layer-storage-schema.md) | Triple-layer storage schema | M2 |
| [RFC-005](done/RFC-005-async-repository-layer.md) | Async SQLite repository layer | M2 |
| [RFC-006](done/RFC-006-post-migration-integrity-check.md) | Post-migration integrity check | M2 |
| [RFC-007](done/RFC-007-content-addressed-object-store.md) | Content-addressed object store | M2 |
| [RFC-008](done/RFC-008-nmp-package-format.md) | `.nmp` binary package format | M3 |
| [RFC-009](done/RFC-009-ed25519-package-signing.md) | Ed25519 package signing | M3 |
| [RFC-010](done/RFC-010-sha256-object-verification.md) | SHA-256 per-object hash verification | M3 |
| [RFC-011](done/RFC-011-quarantine-on-failure.md) | Quarantine on any verification failure | M3 |
| [RFC-012](done/RFC-012-shield-spear-dashboard.md) | Shield / Spear dashboard model | M4 |
| [RFC-013](done/RFC-013-lesson-runner-state-machine.md) | Lesson runner state machine | M4 |
| [RFC-014](done/RFC-014-exercise-evaluation.md) | Exercise evaluation and retry | M4 |
| [RFC-015](done/RFC-015-rtl-ltr-navigation.md) | RTL/LTR navigation mirroring | M4 |
| [RFC-016](done/RFC-016-crash-safe-resume-state.md) | Crash-safe lesson resume state | M4 |
| [RFC-017](done/RFC-017-i18n-3tier-fallback.md) | i18n 3-tier locale fallback | M4 |
| [RFC-018](done/RFC-018-panic-wipe-model.md) | Panic wipe model (key-first) | M5 |
| [RFC-019](done/RFC-019-zero-telemetry-log-policy.md) | Zero-telemetry log policy | M5 |
| [RFC-020](done/RFC-020-safety-settings-domain.md) | Safety settings domain | M5 |
| [RFC-021](done/RFC-021-physical-media-import.md) | Physical media import pipeline | M6 |
| [RFC-022](done/RFC-022-sync-inventory-exchange.md) | Sync inventory exchange protocol | M6 |
| [RFC-023](done/RFC-023-chunk-transfer-model.md) | Chunk-based transfer and resume | M6 |
| [RFC-024](done/RFC-024-permission-timing.md) | Permission request timing rules | M6 |
| [RFC-025](done/RFC-025-performance-budget.md) | Performance budget constants | M7 |
| [RFC-026](done/RFC-026-abdd-accessibility-audit.md) | ABDD accessibility audit | M7 |
| [RFC-027](done/RFC-027-security-review-checklist.md) | Security review checklist | M7 |
| [RFC-028](done/RFC-028-release-manifest-seedkit.md) | Release manifest and seed kit profiles | M8 |
| [RFC-029](done/RFC-029-failure-injection-harness.md) | Failure injection test harness | M8 |
| [RFC-030](done/RFC-030-storage-maintenance-gc.md) | Storage maintenance and GC | M8 |
| [RFC-031](done/RFC-031-offline-health-check.md) | Offline health check service | M9 |
| [RFC-032](done/RFC-032-content-lifecycle.md) | Content lifecycle states | M9 |
| [RFC-033](done/RFC-033-trust-revocation-model.md) | Trust revocation model | M9 |
| [RFC-034](done/RFC-034-field-pilot-protocol.md) | Field pilot protocol | M9 |
| [RFC-035](done/RFC-035-sample-content-packages.md) | Sample content packages | M9 |
| [RFC-036](done/RFC-036-e2e-integration-tests.md) | End-to-end integration test suite | M9 |

---

## Proposed — remediation from v0.9.1 review

These RFCs address findings from the functional and non-functional reviews
of v0.9.1. All P0 items must ship before any field distribution.

### P0 — Release blockers

| RFC | Title | Review finding |
|---|---|---|
| [RFC-037](proposed/RFC-037-core-data-encryption.md) | core.sqlite + object_store encryption | Functional §2, Non-functional §1 |
| [RFC-038](proposed/RFC-038-cryptographic-wipe-redesign.md) | Cryptographic wipe redesign | Functional §3, Non-functional §2 |
| [RFC-039](proposed/RFC-039-streaming-nmp-parser.md) | Streaming `.nmp` parser and bounded import | Functional §4–5, Non-functional §7–9 |
| [RFC-040](proposed/RFC-040-transactional-package-install.md) | Transactional package installation | Functional §6, Non-functional §11 |
| [RFC-041](proposed/RFC-041-event-log-safe-api.md) | event_log safe boundary API | Functional §9, Non-functional §4 |
| [RFC-042](proposed/RFC-042-supply-chain-integrity.md) | Supply chain integrity (Cargo.lock, toolchain, CI) | Non-functional §13–14 |

### P1 — M9 completion criteria

| RFC | Title | Review finding |
|---|---|---|
| [RFC-043](proposed/RFC-043-xtask-real-execution.md) | xtask real execution | Non-functional §15 |
| [RFC-044](proposed/RFC-044-leptos-ui-roadmap.md) | Leptos UI implementation roadmap | Functional §1 |
| [RFC-045](proposed/RFC-045-real-transport-adapter.md) | Real transport adapter (BLE / Wi-Fi / QR) | Functional §8, Non-functional §5 |
| [RFC-046](proposed/RFC-046-revocation-pipeline.md) | Revocation pipeline (real apply) | Non-functional §6 |
| [RFC-047](proposed/RFC-047-trust-anchor-privacy.md) | Trust anchor label privacy | Non-functional §5 |
| [RFC-048](proposed/RFC-048-performance-benchmarks.md) | Performance benchmarks (real measurement) | Non-functional §10 |
| [RFC-049](proposed/RFC-049-housekeeping.md) | Housekeeping (stray files, version consistency, RFC index) | Both reviews |
