# RFC-044: Leptos UI Implementation Roadmap

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M11 (UI sprint) |
| **Priority** | P1 |
| **Review finding** | Functional §1, §7 |

## Problem

`taktakk-ui`, `apps/taktakk-pwa`, and `apps/taktakk-android-wrapper` are
stubs. The project is a Rust crate collection and a console demo, not an
app. As a result, **no feature can be field-tested by a real user**:

- No clock facade screen.
- No Leptos dashboard or module catalog.
- No lesson step viewer (SVG, audio, exercise UI).
- No RTL-mirrored layout in a real DOM.
- No 48 dp touch target enforcement in real rendering.
- No permission request flow visible to users.
- No PWA service worker or Android APK.

## Design

### Screen inventory (per external design v1)

| Screen | Crate | Priority |
|---|---|---|
| Clock facade (digital/analogue + alarm) | taktakk-ui | P0 — required for stealth |
| Setup wizard (3 questions) | taktakk-ui | P0 |
| Home / Shield–Spear dashboard | taktakk-ui | P0 |
| Module list | taktakk-ui | P0 |
| Step viewer (text/SVG/exercise) | taktakk-ui | P0 |
| Share / sync menu | taktakk-ui | P1 |
| Import screen | taktakk-ui | P1 |
| Settings screen | taktakk-ui | P1 |

### `taktakk-ui` architecture

Each screen is a Leptos component:

```
taktakk-ui/src/
  screens/
    clock/          mod.rs, clock.rs, alarm.rs
    home/           mod.rs, dashboard.rs
    catalog/        mod.rs, module_tile.rs
    viewer/         mod.rs, step.rs, exercise.rs, progress_bar.rs
    share/          mod.rs, inventory.rs, transfer.rs
    import/         mod.rs, scan.rs
    settings/       mod.rs, language.rs, a11y.rs
  shell/
    router.rs       memory router; no URL leakage
    unlock.rs       gesture → open shell
  i18n_ctx.rs       Leptos context for locale + direction
  a11y_ctx.rs       Leptos context for a11y settings
```

### Screen contract (per design spec)

Each screen defines:
- **Input state** (props + reactive signals)
- **Empty state** (no modules installed, no progress)
- **Error state** (package quarantined, permission denied)
- **Permission-denied state** (where applicable)

### RTL DOM

The root `<body>` element receives `dir="rtl"` or `dir="ltr"` from the
active locale context. All layout uses CSS logical properties
(`margin-inline-start`, `padding-inline-end`, etc.) rather than
`margin-left` / `padding-right`.

### `apps/taktakk-pwa`

```
apps/taktakk-pwa/
  index.html          minimal shell; no taktakk branding
  src/
    main.rs           Leptos hydrate entry
    sw.js             service worker: offline cache, no network fallback
  Trunk.toml          build config
```

### `apps/taktakk-android-wrapper`

Minimum viable Android shell: a WebView that loads the PWA from local
assets. The app name shown to the OS is configurable at build time
(default: "Clock"). No taktakk branding in the manifest.

## Acceptance criteria

1. `trunk build --release` in `apps/taktakk-pwa` produces a working PWA
   bundle (HTML + WASM + CSS) without network requests.
2. The Android wrapper APK passes `aapt dump badging` with
   `application-label:'Clock'` and no "taktakk" string.
3. Clock facade launches with no educational terminology visible.
4. Unlock gesture opens the dashboard; duress gesture returns to clock.
5. A lesson can be started, stepped through, and completed via the UI.
6. RTL locale switches the `dir` attribute and mirrors navigation arrows.
7. Touch targets are ≥ 48 dp in Chromium DevTools device mode.
8. `cargo test -p taktakk-integration -- ui_smoke` passes a headless
   DOM-based smoke test for each screen.
