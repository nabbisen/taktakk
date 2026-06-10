# Changelog

All notable changes to taktakk are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/).

---

## [0.7.0] — M7 Field Readiness

### Added
- Performance budget model (RFC 025): constants and `TimingMeasurement` checker.
- Field health check: `use_cases::field_check::run_static_checks()`.
- Accessibility audit (`taktakk-a11y::audit`): ABDD compliance checks,
  RTL/LTR locale coverage check, touch target and text scale validation.
- Security audit checklist (`taktakk-security::audit`): 17 automated
  property checks across privacy, crypto, facade safety, wipe, and integrity.
- Full `docs/` tree: architecture, threat model, audit checklist,
  field operator guide, content authoring guide, reproducible builds.
- Updated `README.md` with hero section, feature table, and doc links.

---

## [0.6.0] — M6 Offline Sharing and Import

### Added
- `taktakk-sync` crate: full implementation.
  - `LocalInventory` + SHA-256 snapshot hash.
  - `build_transfer_plan()`: Receive/Send/Skip/VerifyOnly plan.
  - Chunk transfer model: split, verify, reassemble, pending indices.
  - Physical media import: directory scanner, source-path hashing.
  - Delayed permission model: trigger → permission mapping.
  - `LocalFsTransport`: real working filesystem sync adapter.
- Storage: sync_sessions, sync_manifest_items, transfer_chunks,
  import_jobs, import_job_items tables.
- `repo::sync`: session save/retrieve, retention purge, import job lifecycle.

---

## [0.5.0] — M5 Safety and Wipe

### Added
- `use_cases::safety_settings`: DuressAction, LogRetentionPolicy,
  EventBucket with `is_approved()` gate.
- `taktakk-storage::wipe`: state_wipe, destroy_key_slots (7-pass),
  hard_wipe, factory_reset, enforce_log_retention, validate_event_tag.
- `taktakk-security::wipe`: 7-pass `overwrite_key_slot`, `is_safe_log_tag`.
- All wipe operations verified idempotent by test suite.

---

## [0.4.0] — M4 Learning Experience

### Added
- `taktakk-module-engine` crate: catalog, exercise, runner, state, step.
  - LessonRunner state machine: Advance/Answer/Back events.
  - Exercise evaluation: MultipleChoice, Ordering, Acknowledge.
  - Serializable LessonState for crash-safe resume.
  - DashboardView with Shield/Spear ProgressBadge tiles.
- `taktakk-i18n`: NavigationArrows, icon mirror policy, fixture bundle
  (en/ar/sw with RTL arrows).
- Storage: modules/lessons/lesson_steps curriculum tables.
- DB migration refactored to per-statement exec (fixes multi-statement issue).

---

## [0.3.0] — M3 Content Package Installation

### Added
- `taktakk-content` crate:
  - `.nmp` binary parser (magic → version → manifest → signature → objects).
  - `NmpWriter` for building test fixtures.
  - Ed25519 signature verification via trust anchors.
  - SHA-256 per-object hash verification.
  - Install pipeline with quarantine on any failure.
  - `fixtures` module with deterministic test signing keypair.

---

## [0.2.0] — M2 Storage and Data Model

### Added
- Full SQLite repository layer: profile, progress, package, curriculum,
  facade, event_log.
- `FsObjectStore`: content-addressed filesystem object store.
- Database integrity check after every migration (RFC 006).
- Async tokio-based tests for all repository operations.

---

## [0.1.0] — M1 Clock Facade and Local Shell

### Added
- `taktakk-facade-clock`: ClockTime, AlarmEntry, GestureParser.
  - Primary unlock: alarm magic time + long press.
  - Duress trigger: separate magic time → wipe.
  - Tap rhythm alternative unlock.
  - Stopwatch code entry.
- `taktakk-core`: domain types (curriculum, package, profile, progress, sync),
  port traits (storage, crypto, package_store, module_runtime, sync),
  use cases (panic_wipe, open_module, resume_learning, verify_package,
  install_package, start_sync).
- `taktakk-security`: SHA-256 hasher, Ed25519 verifier, Argon2id unlock slot.
- `taktakk-i18n`: BCP 47 locale, RTL/LTR direction, 3-tier fallback lookup.
- `taktakk-a11y`: accessibility settings (contrast, motion, text scale).
- `apps/taktakk-linux`: integration demo CLI.

---

## [0.9.0] — Integration & Sample Content

### Added
- `taktakk-content::samples` — three signed sample packages:
  - `shield-water-purification` (5 steps: text/SVG/acknowledge/MC/summary, en/ar/sw)
  - `spear-basic-math` (4 steps: text/MC/ordering/summary, en)
  - `shield-first-aid-basics` (3 steps: text/ordering/acknowledge, en/ar)
- `taktakk-integration` crate — 16 end-to-end tests across:
  - Unlock/duress gesture, package install + quarantine, profile lifecycle,
    lesson runner with crash resume, sync inventory diff, state/factory wipe,
    key slot destruction, health check, i18n RTL/LTR resolution.
- `apps/taktakk-linux` — full async demo showing complete platform pipeline:
  facade → unlock → install → catalog → lesson → sync → wipe → a11y audit.

### Fixed (`cargo outdated` check)
- `hex = "0.4"` was accidentally placed in `[profile.test]` instead of
  `[workspace.dependencies]` (found by `cargo update --dry-run`). Fixed.
- **Unused imports** across 10 files, surfaced by activating
  `rustflags = ["-D", "warnings"]` in `.cargo/config.toml`.
- `[profile.test] opt-level = 1` removed (caused 7.6 GiB disk pressure).

### Deferred dependency updates
- `rand 0.8.6 → 0.10.1`: breaking change (`thread_rng()` → `rng()`).
  Requires coordinated update of `taktakk-security/wipe.rs` and
  `taktakk-storage/wipe.rs`. Tracked as next maintenance task.
- `sha2 0.10.9 → 0.11.0`: may require `argon2` version bump to maintain
  `generic-array` compat. Tracked with `rand` upgrade sprint.

---

## [0.9.1] — Remediation Sprint (RFC-037–049)

### Fixed (RFC-049)
- Removed 5 stray `.rs` files at `crates/taktakk-core/` root (`domain.rs`,
  `ports.rs`, `use_cases.rs`, `tests.rs`, `error.rs`). These were dead code
  shadowed by the `src/` tree.
- Replaced `rfcs/README.md` (contained sui-id project content) with the
  correct taktakk RFC index (RFC-001–049).
- Bumped workspace version from `0.9.0` → `0.9.1` for consistency with
  release tarball name.

### Added (RFC-042)
- `rust-toolchain.toml` at workspace root (pins Rust 1.91, includes
  rustfmt and clippy components).
- `.github/workflows/ci.yml` with fmt, clippy, and locked-test CI pipeline.

### Changed (RFC-047)
- `TrustAnchor.label` field removed. On-device DB stores only
  `signing_key_id`, `public_key_bytes`, `scope`, `status`, and timestamps.
  Organisation names must not be stored on devices.

### Changed (RFC-041)
- `event_log::append()` is now private.
- New public API: `log_event(pool, EventBucket, Option<SafeEventDetail>, now)`.
- `SafeEventDetail` has no string fields — error codes, counts, and elapsed
  milliseconds only.
- `retention_until` column added to `event_log`; set mandatory on insert.

### Changed (RFC-039)
- `NmpStreamReader<R: Read>` replaces the full-load `parse()` path for the
  install pipeline. Manifest parsed eagerly (≤ 16 KiB); objects streamed
  one at a time through SHA-256 verifier.
- `install_package_stream()` public; `install_package(raw: &[u8])` kept as
  thin `Cursor` wrapper for test fixtures.
- `open_package_stream()` replaces `read_package_file()` in `import.rs`.
- Size limits enforced: manifest 16 KiB, object 20 MiB, package 50 MiB.
- Fixed `NmpWriter::build()` bug: `add_object()` now preserves `ObjectType`;
  previously all objects were re-serialised as `ObjectType::Json`.

### Changed (RFC-040)
- `install_package_transactional()` implements staging → signature verify →
  per-object stream+hash → DB transaction → object promotion.
- `staging/<install_id>/` temp directory cleaned on abort.
- `recover_incomplete_installs()` called by `Database::open()`.
- Quarantine records now persisted to `content_packages` table.
