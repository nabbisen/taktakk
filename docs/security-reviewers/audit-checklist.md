# Security Audit Checklist

Run `cargo test -p taktakk-security -- audit` to execute automated checks.

## Privacy

- [ ] No HTTP client crate in workspace dependencies.
- [ ] No analytics service calls in any crate.
- [ ] Sync sessions store peer ephemeral hash, not raw peer ID.
- [ ] Import source stored as hash of path, not raw path.
- [ ] Event log tags are restricted to approved buckets only.
- [ ] Log retention enforced at session start (24-hour max).

## Cryptography

- [ ] Packages signed with Ed25519 (ed25519-dalek).
- [ ] Unlock slots use Argon2id KDF with salt.
- [ ] Content objects addressed by SHA-256.
- [ ] Key slots never store plaintext keys.
- [ ] `zeroize` applied to in-memory key material on drop.

## Facade safety

- [ ] No "taktakk", "learn", "module", or "curriculum" strings in
  `facade.sqlite` schema or key names.
- [ ] App name shown to OS is generic (e.g. "Clock").
- [ ] Gesture config stored under innocuous key name (`alarm_offset_drift`).
- [ ] No suspicious permissions requested on cold start.

## Wipe reliability

- [ ] `destroy_key_slots()` called before `DELETE FROM` in `hard_wipe()`.
- [ ] Key slots overwritten with 7 passes of random bytes.
- [ ] All wipe functions are idempotent (tested).
- [ ] After factory reset, facade shows unconfigured clock.
- [ ] No "Wipe complete" dialog shown to user.

## Package integrity

- [ ] Ed25519 signature verified before object hash verification.
- [ ] Object hash verified before writing to object store.
- [ ] Failed packages quarantined without executing any content.
- [ ] Magic bytes (`TAKT`) and format version checked first.

## Permission timing

- [ ] No permissions requested on cold start.
- [ ] Bluetooth/network requested only when Share menu opened.
- [ ] Storage requested only when Import from storage opened.
- [ ] All permission requests marked `unlocked_only = true`.
