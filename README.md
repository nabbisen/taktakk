# taktakk

[![crates.io](https://img.shields.io/crates/v/taktakk?label=rust)](https://crates.io/crates/taktakk)
[![Rust Documentation](https://docs.rs/taktakk/badge.svg?version=latest)](https://docs.rs/taktakk)
[![Dependency Status](https://deps.rs/crate/taktakk/latest/status.svg)](https://deps.rs/crate/taktakk)
[![License](https://img.shields.io/github/license/nabbisen/taktakk)](https://github.com/nabbisen/taktakk/blob/main/LICENSE)

Offline-first secure self-learning distribution.

---

## Overview

taktakk is an offline-first educational platform for communities
with no reliable network or power.
It delivers empowerment knowledge (Spear) on low-spec Android phones
without touching the internet.

---

## Why taktakk?

- **No internet required.** Learn fully offline; sync peer-to-peer via
  Bluetooth, Wi-Fi Direct, or SD card.
- **Plausible deniability.** Looks like a clock. No visible product name
  before unlock. Panic wipe destroys all data instantly.
- **Runs on old hardware.** Targets 5–10-year-old Android phones with
  1 GB RAM and slow flash storage.
- **Accessible.** High contrast, large touch targets, audio narration,
  RTL layout for Arabic/Farsi/Urdu, pictogram-first navigation.

---

## Quick Start (developers)

```bash
# Requires Rust 1.91+
cargo build
cargo test

# Run the Linux demo binary (M1–M6 integration)
cargo run -p taktakk-linux
```

See [docs/developers/architecture.md](docs/developers/architecture.md)
for workspace layout and crate responsibilities.

---

## Features

| Feature | Status |
|---|---|
| Clock facade + stealth unlock | ✅ M1 |
| Alarm, stopwatch, timer | ✅ M1 |
| Duress trigger → instant wipe | ✅ M1/M5 |
| SQLite 3-layer storage | ✅ M2 |
| `.nmp` Ed25519-signed packages | ✅ M3 |
| Shield/Spear dashboard | ✅ M4 |
| Lesson runner + exercise drills | ✅ M4 |
| i18n (en/ar/sw) + RTL | ✅ M4 |
| State/hard/factory wipe | ✅ M5 |
| Zero-telemetry log policy | ✅ M5 |
| SD card / local import | ✅ M6 |
| Chunk-based sync inventory | ✅ M6 |
| Permission delay model | ✅ M6 |
| Performance budget model | ✅ M7 |
| Accessibility audit (ABDD) | ✅ M7 |
| Security review checklist | ✅ M7 |

> **⚠ Security-sensitive software.** Review the
> [threat model](docs/security-reviewers/threat-model.md) before deployment.

---

## For more detail

See the [full documentation](docs/README.md) and the following key chapters:

- [Architecture](docs/developers/architecture.md)
- [Threat model](docs/security-reviewers/threat-model.md)
- [Field operator guide](docs/field-operators/seed-distribution.md)
- [Content authoring guide](docs/content-authors/module-authoring-guide.md)
