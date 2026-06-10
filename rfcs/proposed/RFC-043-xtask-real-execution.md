# RFC-043: xtask Real Execution

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P1 |
| **Depends on** | RFC-042 (version consistency) |
| **Review finding** | Non-functional §15 |

## Problem

All `xtask` subcommands (`release-candidate`, `verify-release`, `seed-kit`,
`field-check`, `security-review`, `field-failure-tests`) print instructions
to stdout but perform no actual work. A developer running `cargo xtask
release-candidate` receives a checklist but no verification occurs.

This means:
- Release checklist is entirely manual and error-prone.
- Operator seed kits are not machine-verified.
- `CHECKSUMS.txt` and `release-manifest.json` are never generated.
- The security and field-check audits are decoupled from CI.

## Design

### `xtask check`

Replaces the existing `lint` task:

```
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets --locked
cargo xtask docs-check
```

Reports pass/fail with exit code. Used as the CI gate.

### `xtask release-candidate --version <semver>`

```
1. check_version_consistency(version)   ← RFC-042
2. cargo build --release --locked
3. sha256sum artifacts → CHECKSUMS.txt
4. generate release-manifest.json:
   {
     "version": "...",
     "git_commit": "$(git rev-parse HEAD)",
     "rust_toolchain": "$(rustc -Vv)",
     "cargo_lock_hash": "sha256(Cargo.lock)",
     "artifacts": [{ "name", "sha256", "byte_size", "kind" }, ...]
   }
5. Print summary and artifact paths.
```

### `xtask verify-release --manifest <path>`

```
1. Read release-manifest.json.
2. For each artifact: verify sha256sum matches.
3. Verify cargo_lock_hash matches current Cargo.lock.
4. Report any mismatches.
```

### `xtask seed-kit <profile>`

```
1. Verify all packages in the profile pass install_package() with test anchors.
2. Copy APK/PWA + .nmp packages to out/seed-kit-<profile>/.
3. Generate seed-kit-manifest.json with hashes.
4. Print total size and budget check.
```

### `xtask field-check`

```
1. cargo test -p taktakk-core    -- field_check  2>&1
2. cargo test -p taktakk-a11y   -- audit         2>&1
3. cargo test -p taktakk-security -- audit       2>&1
4. cargo test -p taktakk-integration             2>&1
5. Aggregate and report pass/fail with exit code.
```

### `xtask field-failure-tests`

```
1. cargo test -p taktakk-storage  -- failure_injection 2>&1
2. cargo test -p taktakk-storage  -- maintenance       2>&1
3. cargo test -p taktakk-integration -- install_power_loss_recovery 2>&1
4. Aggregate and report.
```

## Acceptance criteria

1. `cargo xtask check` exits 0 on a clean workspace; exits non-zero if
   any of fmt/clippy/test/docs fail.
2. `cargo xtask release-candidate --version 0.9.1` produces `CHECKSUMS.txt`
   and `release-manifest.json` with correct SHA-256 values.
3. `cargo xtask verify-release --manifest release-manifest.json` exits 0
   when checksums match; exits non-zero on any mismatch.
4. `cargo xtask field-check` exits 0 when all 266+ tests pass;
   exits non-zero and prints failing test names otherwise.
5. No xtask subcommand exits 0 while only printing instructions.
