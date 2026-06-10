# RFC-042: Supply Chain Integrity

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Review finding** | Non-functional §13–14, §17 |

## Problem

Three supply chain gaps prevent reproducible, auditable builds:

1. **`Cargo.lock` is absent from the tarball.** `cargo build --locked`
   cannot be run. The dependency tree cannot be reproduced. Offline builds
   are unreliable.

2. **`rust-toolchain.toml` is absent.** The `rustc`/`cargo` version is
   documented in `docs/src/contributing/reproducible-builds.md` but not
   machine-enforced. Different operators may build with different Rust
   versions, producing non-identical binaries.

3. **Version inconsistency.** The tarball is named `taktakk-v0_9_1.tar.gz`
   but `Cargo.toml` workspace version is `0.9.0`. `CHANGELOG.md` and
   `release tag` do not match the artifact name.

## Design

### `Cargo.lock`

As an application workspace (not a library-only crate), `Cargo.lock`
**must** be committed to version control and included in every release
tarball.

```bash
# Add to .gitignore exclusion:
!Cargo.lock
```

Release tarball creation (`xtask release-candidate`) must verify:
```
sha256sum Cargo.lock >> release-manifest.json
```

### `rust-toolchain.toml`

```toml
[toolchain]
channel    = "1.91"
components = ["rustfmt", "clippy"]
targets    = [
    "x86_64-unknown-linux-gnu",
    "aarch64-linux-android",
    "wasm32-unknown-unknown",
]
```

Placed at the workspace root. Cargo and rustup both honour this file.
It supersedes the documentation-only version pin in
`reproducible-builds.md`.

### Version consistency check in xtask

`xtask release-candidate` must reject a build if any of these disagree:

- `Cargo.toml` workspace `version`
- `CHANGELOG.md` most-recent `## [x.y.z]` heading
- `git describe --exact-match --tags HEAD` (if on a tagged commit)
- tarball name passed as argument

```rust
fn check_version_consistency(expected: &str) -> Result<(), String> {
    let cargo_ver = read_cargo_version()?;
    if cargo_ver != expected {
        return Err(format!(
            "Cargo.toml version {cargo_ver} != expected {expected}"
        ));
    }
    let changelog_ver = read_changelog_version()?;
    if changelog_ver != expected {
        return Err(format!(
            "CHANGELOG.md version {changelog_ver} != expected {expected}"
        ));
    }
    Ok(())
}
```

### CI pipeline (`.github/workflows/ci.yml`)

Minimum CI job:

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-24.04
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt --all -- --check
      - run: cargo clippy --workspace --all-targets -- -D warnings
      - run: cargo test --workspace --all-targets --locked
      - run: cargo xtask docs-check
```

The `--locked` flag on `cargo test` ensures `Cargo.lock` is present and
up to date.

## Acceptance criteria

1. `Cargo.lock` exists at workspace root and is included in the release tarball.
2. `rust-toolchain.toml` exists at workspace root; `rustup show` reports
   the pinned channel.
3. `cargo build --locked` succeeds from a clean checkout with no network
   access (using `cargo vendor` or offline cache).
4. `xtask release-candidate --version 0.9.1` fails if `Cargo.toml` says
   `0.9.0`.
5. CI job definition exists at `.github/workflows/ci.yml` and runs
   `--locked` test.
6. `Cargo.toml` workspace version, `CHANGELOG.md`, and tarball name are
   all `0.9.1` after this RFC is applied.
