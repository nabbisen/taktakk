# Panic Wipe Documentation

## Overview

taktakk supports two types of data destruction:

1. **State wipe** — removes all learning progress; profiles and packages remain.
2. **Hard wipe / Factory reset** — destroys all data and returns the app to a
   factory-fresh clock state.

## Cryptographic erasure (the "instant kill")

taktakk uses **cryptographic erasure** rather than slow file deletion. All data
in `core.sqlite` and the object store is encrypted with a master key stored in
`facade.sqlite`. Wipe works by:

1. Overwriting the master key slot with 7 passes of random bytes.
2. Deleting core database tables (best-effort; data is already unreadable).
3. Resetting the facade to an unconfigured clock state.

Without the master key, recovering any plaintext from the remaining ciphertext
requires breaking 256-bit AES — computationally infeasible.

## Duress triggers

The duress code is configured from inside the unlocked shell. It is stored as
a separate KDF verifier hash (Argon2id), never as plaintext.

When the duress code is entered in the facade clock:
- No confirmation dialog is shown.
- The app transitions to factory-reset state silently.
- If an inspector opens the app, they see an unconfigured clock.

## Wipe idempotency

All wipe functions are tested for idempotency: calling them on an already-wiped
database must not fail. This ensures that a second invocation during an
interrupted wipe does not cause errors.

## What survives a wipe

After factory reset, the following survive:
- The clock face and display settings.
- The app binary itself.
- Nothing else.

The following are destroyed:
- Unlock sequences and duress codes.
- All learning progress.
- All installed packages and curriculum data.
- All trust anchors.
- The event log.

## Testing

Run `cargo test -p taktakk-storage -- wipe` to execute the full wipe test suite,
including idempotency, key slot destruction verification, and factory-reset
facade state checks.
