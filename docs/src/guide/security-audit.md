# Security Audit

Run automated checks with:

```bash
cargo test -p taktakk-security -- audit
cargo test -p taktakk-core    -- field_check
cargo test -p taktakk-a11y    -- audit
```

## Automated checklist

The `run_security_audit()` function in `taktakk-security::audit` verifies
17 properties at compile/runtime. All must pass before distribution.

### Privacy

- No HTTP client crate in workspace dependencies.
- Sync sessions store peer ephemeral hash, not raw peer ID.
- Import source stored as hash of path, not raw path.
- Event log tags restricted to approved buckets (no domain words).
- Log retention enforced: 24-hour maximum.

### Cryptography

- Packages signed with Ed25519 (`ed25519-dalek`).
- Unlock slots use Argon2id KDF with per-slot salt.
- Content objects addressed by SHA-256.
- Key slots never store plaintext keys.
- `zeroize` applied to in-memory key material on drop.

### Facade safety

- No "taktakk", "learn", "module", or "curriculum" strings in
  `facade.sqlite` schema or setting keys.
- App name shown to OS is generic.
- Gesture config stored under innocuous key name.
- No suspicious permissions requested at cold start.

### Wipe reliability

- Key slot destruction precedes file deletion in `hard_wipe()`.
- Key slots overwritten with 7 passes of random bytes.
- All wipe functions are idempotent.
- After factory reset, facade shows unconfigured clock.

### Package integrity

- Ed25519 signature verified before object hash verification.
- Object hash verified before writing to object store.
- Failed packages quarantined without executing any content.
- Magic bytes (`TAKT`) and format version checked first.

### Permission timing

- No permissions requested at cold start.
- All permission requests marked `unlocked_only = true`.

## Threat model summary

| Threat | Mitigation |
|---|---|
| Physical inspection of locked device | Clock facade; no visible product name or learning content |
| Coerced unlock / checkpoint | Duress gesture → silent crypto erasure |
| Tampered content distribution | Ed25519 signature + SHA-256 hash; quarantine on failure |
| Network traffic analysis | Zero outbound connections; P2P with ephemeral IDs |
| Post-wipe forensics | 7-pass key overwrite; computationally unrecoverable |
| Log metadata leakage | Approved tag buckets only; 24-hour retention |

For the full threat model see [Design Philosophy](../contributing/philosophy.md).
