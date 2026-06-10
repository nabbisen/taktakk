# Threat Model

## Threat actors

| Actor | Capability | Goal |
|---|---|---|
| Border inspector | Physical access to unlocked device | Identify learning app, link user to aid org |
| Checkpoint officer | Coerced unlock | Extract user identity or content list |
| Malicious package distributor | Can create `.nmp` files | Inject false/harmful content |
| Passive network observer | Can see network traffic | Identify taktakk usage |
| Storage forensics | Post-wipe disk imaging | Recover deleted learning content |

## Mitigations

### Facade safety (border/checkpoint)
- App presents as a clock with no suspicious permissions on launch.
- No "taktakk" branding, no "learn" strings in locked state.
- Stealth unlock via normal clock interactions (set alarm → long press).
- Duress code triggers silent wipe → app returns to factory-fresh clock.

### Content integrity (malicious distributor)
- All `.nmp` packages require Ed25519 signature from a trusted anchor.
- Magic bytes + format version validated before manifest is parsed.
- SHA-256 hash of every object verified after extraction.
- Failed packages are quarantined and never executed.

### Network invisibility (observer)
- Zero outbound connections. No analytics, no update checks, no STUN.
- P2P sync uses truncated rotating hash for discovery (RFC 022).
- Permission requests delayed until user explicitly opens Share/Import.

### Post-wipe forensics (storage)
- Crypto erasure: key slots overwritten with 7 passes of random bytes.
- Without the master key, ciphertext is computationally unrecoverable.
- Facade resets to unconfigured clock state after wipe.
- Event log purged. Unlock slots zeroed.

## Out of scope (early versions)

- Compromised bootloader / ROM-level forensics.
- Side-channel attacks on the KDF (requires physical access to RAM).
- Social engineering of the user to reveal the unlock sequence.
