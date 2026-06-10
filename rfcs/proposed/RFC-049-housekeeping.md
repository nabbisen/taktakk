# RFC-049: Housekeeping

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint, first PR) |
| **Priority** | P1 — should be done first (unblocks audit trail) |
| **Review finding** | Functional §10; Non-functional §17, §22–23 |

## Problem

Three unrelated housekeeping issues found in the v0.9.1 review:

### 1. `rfcs/README.md` contains `sui-id` content

The RFC index opens with `# sui-id RFCs` and lists WebAuthn, HIBP, and
email outbox topics. This is content from a different project. An auditor
or implementer reading the taktakk RFC index cannot find the correct
design rationale for any feature.

### 2. Stray `.rs` files at `crates/taktakk-core/` root

Root-level `domain.rs`, `ports.rs`, `use_cases.rs`, `tests.rs` may exist
at `crates/taktakk-core/` alongside the canonical `src/` tree. Cargo uses
`src/lib.rs` so these files are dead code — but they create maintenance
confusion and could diverge from the live implementation.

### 3. Version inconsistency

Tarball: `taktakk-v0_9_1.tar.gz`
`Cargo.toml`: `version = "0.9.0"`
`CHANGELOG.md`: `## [0.9.0]`

These three should always match. They do not match in v0.9.1.

## Design

### Fix 1 — RFC index

`rfcs/README.md` is rewritten as the taktakk RFC index (done in this
development session). Content:
- Status key table.
- Done/ table (RFC-001 through RFC-036).
- Proposed/ table (RFC-037 through RFC-049).
- No reference to any other project.

### Fix 2 — Stray files

```bash
# Find and remove stray .rs files at crate root level
find crates -maxdepth 2 -name '*.rs' \
  ! -path '*/src/*' \
  ! -name 'build.rs' | xargs rm -f
```

Add a CI check to `xtask check` (RFC-043):

```rust
fn check_no_stray_rs_files() -> Result<(), String> {
    let stray: Vec<_> = glob("crates/*/*.rs")?
        .filter_map(|p| p.ok())
        .filter(|p| !p.starts_with("crates/*/src/"))
        .collect();
    if !stray.is_empty() {
        return Err(format!("stray .rs files: {:?}", stray));
    }
    Ok(())
}
```

### Fix 3 — Version consistency

Bump `Cargo.toml` workspace version to `0.9.1`. Update `CHANGELOG.md`
to add `## [0.9.1]` entry. The tarball name and git tag must match.

This is a prerequisite for RFC-042's version consistency check.

## Acceptance criteria

1. `rfcs/README.md` opens with `# taktakk RFCs` and contains no
   reference to `sui-id`, WebAuthn, HIBP, or email outbox.
2. `find crates -maxdepth 2 -name '*.rs'` returns only files under
   `crates/*/src/` or `crates/*/build.rs`.
3. `Cargo.toml` version, `CHANGELOG.md` most-recent heading, tarball
   name, and git tag all read `0.9.1`.
4. `cargo check` (with `--locked`) succeeds after the version bump.
5. `rfcs/done/` contains stubs for RFC-001 through RFC-036.
6. `rfcs/proposed/` contains RFC-037 through RFC-049 (this sprint).
