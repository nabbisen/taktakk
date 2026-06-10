# Architecture

## Core principles

taktakk is built around four invariants that must never be violated:

1. **Offline first.** No feature requires an internet connection.
2. **Facade safety.** Nothing in the locked state reveals the app's purpose.
3. **Key-first wipe.** Crypto keys are destroyed before any slow deletion.
4. **Zero telemetry.** No analytics, no crash reporters, no network pings.

## Workspace layout

```
crates/
  taktakk-core          Domain types, use cases, port traits
  taktakk-storage       SQLite repositories, object store, wipe ops
  taktakk-content       .nmp parser, Ed25519 verification, install pipeline
  taktakk-module-engine Lesson runner, exercise evaluation, catalog model
  taktakk-i18n          BCP 47 locale, RTL/LTR, 3-tier string lookup
  taktakk-a11y          Accessibility settings, ABDD audit
  taktakk-security      Key slots, Argon2id verifier, wipe coordinator, audit
  taktakk-sync          Inventory diff, chunk transfer, local-FS transport
  taktakk-facade-clock  Clock domain, stealth unlock/duress gesture parser
  taktakk-ui            Leptos component library (stub; M4+ UI)
apps/
  taktakk-linux         Development CLI and CI demo binary
  taktakk-pwa           PWA entry point (stub)
  taktakk-android-wrapper  Android wrapper (stub)
```

## Storage layers

| Layer | File | Contents |
|---|---|---|
| Facade | `facade.sqlite` | Alarm entries, clock settings, key slots (neutral names) |
| Core | `core.sqlite` | Curriculum, progress, sync history (encrypted at rest) |
| Objects | `object_store/` | Content-addressed binary assets (SHA-256 addressed) |

## Dependency rules

- `taktakk-core` has no I/O dependencies.
- `taktakk-ui` must not contain SQL, crypto, or transport logic.
- Facade-visible storage must not contain educational terminology.
- All wipe operations must destroy keys before deleting files.

## Security architecture

The security model is a layered defence:

1. **Facade layer:** The app looks like a clock. No suspicious UI.
2. **Unlock layer:** KDF-derived verifier (Argon2id). Never stores raw passcode.
3. **Content layer:** Ed25519-signed `.nmp` packages. Hash-verified objects.
4. **Wipe layer:** Crypto erasure via key slot overwrite. 7-pass random fill.
5. **Log layer:** Only approved event bucket tags. 24-hour retention. No PII.
