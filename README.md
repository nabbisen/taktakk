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

## Why taktakk — and when to use it

Use taktakk when learners need access to knowledge but:

- Internet infrastructure is absent, unreliable, or surveilled.
- Power is intermittent (solar, generator, none).
- Devices are old (1 GB RAM, 5–10-year-old Android).
- Carrying an openly educational app could attract unwanted attention.
- Content must travel physically between communities.

Typical deployments: humanitarian field operations, refugee education
programmes, community health worker training, digital literacy in
low-connectivity regions.

---

## Quick Start

```bash
# Requires Rust 1.91+ and cargo-1.91
cargo build
cargo test

# Run the end-to-end demo (unlock → install → lesson → wipe)
cargo run -p taktakk-linux
```

Import a sample content package from Rust:

```rust
use taktakk_content::{
    install::install_package,
    samples::build_shield_water_package,
    fixtures::test_trust_anchor,
};

let nmp     = build_shield_water_package()?;
let outcome = install_package(&nmp, "pkg-001", &[test_trust_anchor()], &store, now);
```

---

## Design Notes

**Clock facade over a dedicated lock screen.**
A clock is consulted briefly in any context. No product name, icon hint,
or permission request appears until the user performs the unlock gesture.
A separate duress code silently wipes everything and returns to the clock.

**Crypto erasure over slow deletion.**
The wipe path overwrites cryptographic key slots with seven passes of random
bytes before touching any file. Without the key, ciphertext is computationally
unrecoverable — even if deletion is interrupted by a power cut.

**Port-based architecture.**
`taktakk-core` contains only domain types, use cases, and trait ports. It
has no database, network, or crypto I/O of its own. This keeps all business
logic independently testable and straightforward to port to new platforms.

**Zero telemetry by design.**
The only log is a 24-hour rolling event log using ten approved anonymous
tag buckets. It contains no module names, user identifiers, or content
details. No analytics, crash reporters, or update pings exist.

---

## For more detail

See the [full documentation](docs/src/SUMMARY.md):

- [What is taktakk?](docs/src/getting-started/features.md)
- [Tutorial](docs/src/getting-started/tutorial.md)
- [API Reference](docs/src/guide/api-reference.md)
- [Design Philosophy](docs/src/contributing/philosophy.md)
- [Architecture](docs/src/contributing/architecture.md)
- [Local Development](docs/src/contributing/local-development.md)
