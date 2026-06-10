# Reproducible Builds

## Why reproducibility matters

taktakk operates in high-trust environments. Field operators and
support organisations must be able to verify that the binary they are
distributing matches the published source code.

## Build environment

| Component | Version | Notes |
|---|---|---|
| Rust toolchain | 1.91 | Pinned via `rust-toolchain.toml` |
| Cargo edition | 2024 | Set in workspace `Cargo.toml` |
| Target (Android) | `aarch64-linux-android` | API level 26+ |
| Target (Linux) | `x86_64-unknown-linux-gnu` | For CI and development |

## Dependency pinning

All dependencies are pinned via `Cargo.lock`. Never use `cargo update`
without a full audit of the changed dependency tree.

```bash
# Verify the lock file is committed and up to date
cargo verify-project
```

## Build flags

Release builds must use:
```toml
[profile.release]
opt-level = "z"    # Optimize for size (target: < 50 MB core)
lto = true
strip = true
panic = "abort"    # No unwinding runtime overhead
```

## Verifying a binary

```bash
# Build from source and compare SHA-256
cargo build --release --target aarch64-linux-android
sha256sum target/aarch64-linux-android/release/taktakk

# Compare against the published hash in CHECKSUMS.txt
```

## Audit trail

Every release must include:
- `CHECKSUMS.txt` — SHA-256 of the APK and PWA bundle.
- `NOTICE` — dependency attribution.
- Git tag signed with the project signing key.
