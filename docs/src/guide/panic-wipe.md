# Panic Wipe

## Overview

taktakk supports three levels of data erasure:

| Operation | What it removes | Speed |
|---|---|---|
| `state_wipe` | Learning progress only; keys and packages intact | Fast |
| `hard_wipe` | Keys + all core data; facade intact | Fast + slow |
| `factory_reset` | Hard wipe + reset facade to unconfigured clock | Fast + slow |

## How cryptographic erasure works

taktakk does **not** rely on slow file deletion for security. Instead:

1. All data in `core.sqlite` and the object store is encrypted with a
   master key stored in `facade.sqlite` (in `key_registry`).
2. Wipe overwrites the master key slots with **7 passes of random bytes**.
3. Without the master key, recovering any plaintext from the remaining
   ciphertext requires breaking 256-bit AES — computationally infeasible.
4. Slow file deletion then removes the ciphertext (best-effort; data is
   already unreadable before this step completes).

**Power loss between step 2 and step 3 is acceptable.** The data is already
unreadable once the keys are overwritten.

## Duress trigger

The duress code is set from inside the unlocked shell. It is stored as an
Argon2id KDF verifier hash — never as plaintext.

When the duress gesture is entered via the facade clock:

- No confirmation dialog is shown.
- `factory_reset()` is called immediately.
- The app transitions to a plain, unconfigured clock.

## Idempotency

All wipe functions are idempotent by design. Calling them on an already-wiped
database produces no error and no effect. This ensures that a second invocation
during an interrupted wipe is always safe.

Tests:

```bash
cargo test -p taktakk-storage -- wipe_idempotent
cargo test -p taktakk-storage -- factory_reset_idempotent
```

## What survives a factory reset

| Survives | Destroyed |
|---|---|
| Clock display settings | Unlock and duress codes |
| App binary | All learning progress |
| — | All installed packages |
| — | All trust anchors |
| — | Event log |

## Testing the wipe path

```bash
# Unit tests — in-memory and in-DB
cargo test -p taktakk-security -- wipe
cargo test -p taktakk-storage  -- wipe

# End-to-end integration tests
cargo test -p taktakk-integration -- wipe
```

Before distributing any device, perform a manual wipe test on a non-critical
unit and confirm the device appears as an unconfigured clock with no content.
