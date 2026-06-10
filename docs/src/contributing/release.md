# Release Process

## Pre-release checklist

### Code quality
- [ ] `cargo test` passes with 0 failures.
- [ ] `cargo check` produces 0 warnings (`-D warnings` is enforced).
- [ ] `cargo fmt --check --all` passes.
- [ ] `cargo xtask docs-check` passes.
- [ ] `CHANGELOG.md` is up to date.

### Security review
- [ ] `cargo test -p taktakk-security -- audit` passes (17 checks).
- [ ] `cargo test -p taktakk-core -- field_check` passes.
- [ ] `cargo test -p taktakk-a11y -- audit` passes.
- [ ] No suspicious terms in facade schema (manual review).
- [ ] No HTTP client in dependency tree (`cargo tree | grep reqwest`).

### Integration
- [ ] `cargo test -p taktakk-integration` passes (16 tests).
- [ ] `cargo run -p taktakk-linux` produces expected demo output.

### Documentation
- [ ] `cargo xtask docs-check` confirms all required pages exist.
- [ ] SUMMARY.md links are all valid.

## Building the release

```bash
# Build with locked dependencies
cargo build --release --locked

# Record SHA-256 of output binary
sha256sum target/release/taktakk > CHECKSUMS.txt

# Generate release manifest (placeholder — full tooling TBD)
cargo xtask release-candidate
```

## Assembling a seed kit

```bash
# Minimal kit: app + emergency Shield + one locale (~5 MB)
cargo xtask seed-kit minimal

# Standard kit: app + core modules + common locales (~25 MB)
cargo xtask seed-kit standard

# Full kit: all approved modules + all locale packs (~50 MB)
cargo xtask seed-kit full
```

## Publishing

1. Tag the release in git: `git tag -s v0.9.0 -m "v0.9.0"`
2. Push the tag: `git push origin v0.9.0`
3. Publish `CHECKSUMS.txt`, `release-manifest.json`, and `NOTICE`.
4. Update the field operator distribution documentation if the
   install workflow has changed.

## After release

- [ ] Announce to field operators via the offline channel (no public web post).
- [ ] Update trust anchor sets if any signing keys were rotated.
- [ ] File any quarantined packages from the previous release cycle as issues.
