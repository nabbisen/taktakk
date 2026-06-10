# Release Checklist

## Pre-release

### Code quality
- [ ] `cargo test` passes with 0 failures.
- [ ] `cargo check` produces 0 warnings.
- [ ] `cargo fmt --check --all` passes.
- [ ] No `TODO` or `FIXME` in security-critical code paths.
- [ ] CHANGELOG.md is up to date.

### Security review
- [ ] `cargo test -p taktakk-security -- audit` passes.
- [ ] `cargo test -p taktakk-core -- field_check` passes.
- [ ] Facade schema has no educational terminology.
- [ ] No HTTP client in dependency tree.
- [ ] Log tag policy enforced (approved buckets only).
- [ ] Wipe idempotency tests pass.

### Accessibility review
- [ ] `cargo test -p taktakk-a11y -- audit` passes.
- [ ] Default settings use Reduced motion, large touch targets, Dark contrast.
- [ ] At least one LTR and one RTL locale fixture passes.

### Documentation
- [ ] All required persona sections exist under `docs/`.
- [ ] `README.md` links to correct documentation.
- [ ] `docs/release/reproducible-builds.md` is current.

## Release build

```bash
# Build with locked dependencies
cargo build --release --locked

# Generate release manifest
cargo xtask release-candidate

# Verify checksums
cargo xtask verify-release
```

## Post-release

- [ ] Release tagged in git with project signing key.
- [ ] `CHECKSUMS.txt` published alongside release.
- [ ] Seed kit bundles assembled and tested.
- [ ] Field operator distribution guide updated if workflow changed.
- [ ] Any revoked packages or keys updated in trust anchor set.
