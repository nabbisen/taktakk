# Architecture

## Workspace layout

```
crates/
  taktakk-core            Domain types, use cases, port traits (no I/O)
  taktakk-storage         SQLite repositories, object store, wipe operations
  taktakk-content         .nmp parser, Ed25519 verification, install pipeline
  taktakk-module-engine   Lesson runner, exercise evaluation, catalog model
  taktakk-facade-clock    Clock domain, stealth unlock/duress gesture parser
  taktakk-i18n            BCP 47 locale, RTL/LTR, 3-tier string lookup
  taktakk-a11y            Accessibility settings, ABDD compliance audit
  taktakk-security        Key slots, Argon2id verifier, wipe coordinator, audit
  taktakk-sync            Inventory diff, chunk transfer, transport adapters
  taktakk-ui              Leptos component library (UI layer — M4+)
  taktakk-integration     End-to-end integration tests

apps/
  taktakk-linux           Development CLI, CI demo, and integration harness
  taktakk-pwa             PWA entry point (Leptos/WASM — stub)
  taktakk-android-wrapper Android JNI wrapper (stub)

docs/
  book.toml               mdBook configuration
  src/                    Documentation source (this directory)
```

## Crate dependency rules

1. `taktakk-core` has **no runtime I/O dependencies**. It depends only on
   `serde`, `thiserror`, `uuid`, and `chrono`. All external I/O is expressed
   as port traits.

2. `taktakk-ui` must **not** contain SQL queries, cryptographic primitives,
   or transport logic. It is a pure rendering layer.

3. **Facade-visible storage** (`facade.sqlite`) must **not** contain educational
   terminology. Table and column names must be clock-context-neutral.

4. All wipe operations must **destroy keys before deleting files**.

## Storage layers

| Layer | File | Purpose |
|---|---|---|
| Facade | `facade.sqlite` | Clock settings, alarm entries, key slots (neutral names) |
| Core | `core.sqlite` | Curriculum, progress, sync history, event log |
| Objects | `object_store/` | Content-addressed binary assets (SHA-256 keyed) |

The object store layout follows Git's loose object convention:
`object_store/<first-2-hex-chars>/<remaining-62-chars>`.

## Security layers

```
┌──────────────────────────────────────────────────────┐
│  Clock Facade      (no suspicious UI, no permissions) │
├──────────────────────────────────────────────────────┤
│  Unlock Layer      (Argon2id KDF, never stores code)  │
├──────────────────────────────────────────────────────┤
│  Content Layer     (Ed25519 + SHA-256, quarantine)    │
├──────────────────────────────────────────────────────┤
│  Wipe Layer        (7-pass key overwrite, idempotent) │
├──────────────────────────────────────────────────────┤
│  Log Layer         (24h retention, approved tags only)│
└──────────────────────────────────────────────────────┘
```

## Port trait pattern

`taktakk-core` defines traits for all I/O. Platform crates implement them:

```
taktakk-core::ports::storage::CurriculumRepository
    ↑ implemented by taktakk-storage::repo::curriculum

taktakk-core::ports::crypto::SignatureVerifier
    ↑ implemented by taktakk-security::verifier::Ed25519Verifier

taktakk-core::ports::package_store::ObjectStore
    ↑ implemented by taktakk-storage::object_store::FsObjectStore
```

This keeps all use-case logic in `taktakk-core` testable without any
database or filesystem access.
