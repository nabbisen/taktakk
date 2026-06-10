# Reproducible Builds

taktakk operates in high-trust environments. Field operators and support
organisations must be able to verify that the binary they are distributing
matches the published source code.

## Build environment

| Component | Version | Notes |
|---|---|---|
| Rust toolchain | 1.91 | Pinned in workspace `Cargo.toml` |
| Cargo edition | 2024 | Set in `[workspace.package]` |
| Target (Android) | `aarch64-linux-android` | API level 26+ |
| Target (Linux CI) | `x86_64-unknown-linux-gnu` | Development and testing |

## Dependency pinning

All dependencies are pinned via `Cargo.lock`. Never run `cargo update`
without a full audit of the changed dependency tree.

```bash
# Verify the lock file is consistent with Cargo.toml
cargo check --locked

# Check for available updates without applying them
cargo update --dry-run --verbose
```

## Release build flags

```toml
[profile.release]
opt-level     = "z"    # Optimize for binary size
lto           = true   # Link-time optimisation
codegen-units = 1      # Single codegen unit for best LTO
strip         = true   # Strip debug symbols
panic         = "abort" # No unwinding runtime
```

These settings target the ≤ 50 MB binary size requirement.

## Verifying a release binary

```bash
# Build from the exact locked dependency tree
cargo build --release --locked --target aarch64-linux-android

# Record the SHA-256
sha256sum target/aarch64-linux-android/release/taktakk

# Compare against CHECKSUMS.txt published with the release
sha256sum -c CHECKSUMS.txt
```

## Audit trail

Every release must include:

- `CHECKSUMS.txt` — SHA-256 of the APK and PWA bundle.
- `release-manifest.json` — version, git commit SHA (40 hex chars),
  Rust toolchain version, artifact list.
- `NOTICE` — complete dependency attribution.
- A git tag signed with the project signing key.

The `release-manifest.json` must itself be verifiable:

```bash
cargo xtask verify-release
```
