# Local Development

## Environment setup

### Rust toolchain

taktakk requires Rust 1.91 and Cargo 1.91 (2024 edition).

On Ubuntu 24.04 LTS, install via the versioned apt packages:

```bash
sudo apt-get install -y rustc-1.91 cargo-1.91

# Switch the default toolchain
sudo update-alternatives --install /usr/local/bin/rustc  rustc  /usr/bin/rustc-1.91  200
sudo update-alternatives --install /usr/local/bin/cargo  cargo  /usr/bin/cargo-1.91  200
sudo update-alternatives --install /usr/local/bin/rustdoc rustdoc /usr/bin/rustdoc-1.91 200

rustc --version   # → rustc 1.91.x
cargo --version   # → cargo 1.91.x
```

### System libraries

```bash
sudo apt-get install -y libssl-dev libsqlite3-dev pkg-config
```

## Running the test suite

```bash
# Run all tests
cargo test

# Run a specific crate
cargo test -p taktakk-core
cargo test -p taktakk-storage
cargo test -p taktakk-integration

# Run a specific test by name pattern
cargo test -p taktakk-security -- audit
cargo test -p taktakk-storage  -- wipe
```

The full suite should report **0 failures** and **0 warnings**
(warnings are treated as errors via `.cargo/config.toml`).

## Running the demo binary

```bash
cargo run -p taktakk-linux
```

This exercises the complete pipeline end-to-end:
- Clock facade + stealth unlock gesture
- Sample package install and catalog display
- Lesson runner with exercise drill
- P2P sync inventory diff
- i18n RTL/LTR navigation
- Accessibility audit

## Development tasks (xtask)

```bash
cargo xtask help             # list all available tasks
cargo xtask lint             # cargo fmt + clippy check
cargo xtask field-check      # static field-readiness health checks
cargo xtask security-review  # security audit instructions
cargo xtask docs-check       # verify all required doc files exist
```

## Project layout conventions

- **Tests** live in `src/tests.rs` (a separate module, not inline `#[cfg(test)]`
  blocks), per the project convention.
- **Port traits** are defined in `taktakk-core::ports` and implemented in
  crates that have the required dependencies.
- **Facade-visible storage** must only use clock-context-neutral names for
  tables and columns.
- All new public items must be documented with at least a one-line `///` comment.

## Checking for outdated dependencies

```bash
# Using cargo's built-in resolver (no extra tools needed)
cargo update --dry-run --verbose
```

Known deferred upgrades:
- `rand 0.8 → 0.10`: requires updating `thread_rng()` → `rng()` at two sites.
- `sha2 0.10 → 0.11`: requires coordinating with `argon2` version bump.

## Continuous integration

The CI pipeline runs on every pull request:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo xtask docs-check
```
