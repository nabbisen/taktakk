# Design Philosophy

## Purpose

taktakk exists because **access to knowledge should not depend on network
infrastructure, stable power, or freedom from authoritarian surveillance**.

Its target users are people in conflict zones, refugee camps, and under
occupation — people for whom learning how to purify water or calculate a
fair trade price can change the outcome of their day.

## Four invariants

These constraints are non-negotiable. Every design decision must satisfy
all four simultaneously:

**1. Offline first**
No feature requires an internet connection. Features that could in principle
use the network (sync, update checks) must degrade gracefully to a local-only
mode and must work without any network access at all.

**2. Facade safety**
Nothing in the locked state reveals that an educational platform exists.
Not a logo, not a string, not a file path, not a permission, not a network
request. An inspector holding a locked device must find a convincing clock.

**3. Key-first wipe**
Cryptographic keys are destroyed before any slow deletion operation begins.
"Slow deletion" includes database row deletion and filesystem `unlink()` calls.
This ensures that a power loss during wipe leaves data unreadable, not
recoverable. See [Panic Wipe](../guide/panic-wipe.md).

**4. Zero telemetry**
No analytics calls. No crash reporters. No version-check pings. No usage
statistics. The only persistent log is a 24-hour rolling event log using
approved anonymous tag buckets. Nothing leaves the device unless the user
explicitly initiates a sync.

## Threat model

The primary threats this software is designed to resist:

| Threat actor | Capability | Defence |
|---|---|---|
| Border inspector | Physical access to locked device | Clock facade; no suspicious permissions |
| Checkpoint officer | Coerced unlock | Duress code → instant crypto erasure |
| Malicious content distributor | Create `.nmp` files | Ed25519 signatures; quarantine on failure |
| Passive network observer | See traffic | Zero outbound connections |
| Post-seizure forensics | Image device after wipe | 7-pass key overwrite; unrecoverable ciphertext |

**Out of scope** for early versions: compromised bootloader, RAM side-channels,
social engineering of the unlock gesture.

## Design notes on specific decisions

**Why a clock and not a calculator?**
A clock is consulted briefly, repeatedly, in any context. A calculator is
used with deliberate focus. The clock cover story withstands casual inspection
without requiring the user to act naturally under stress.

**Why Argon2id for the unlock KDF?**
The unlock sequence is short (a few digits of time + a gesture duration). A
KDF with memory-hard cost makes brute-force enumeration of the unlock sequence
impractically slow even on dedicated hardware.

**Why Ed25519 over RSA for content signing?**
Smaller keys and signatures, faster verification on low-end ARM processors,
and no padding oracle attacks. Signature bytes are 64 bytes vs 256+ for RSA.

**Why content-addressed object storage?**
SHA-256 addressing allows deduplication (the same SVG used in two modules is
stored once), integrity verification on every read, and atomic installation
(a partially-received package cannot be mistaken for a complete one).

**Why per-statement SQLite DDL instead of multi-statement migrations?**
`sqlx::query()` accepts exactly one SQL statement. This constraint forces
each schema change to be expressed as an atomic, independently retryable
unit, which makes migration failure recovery straightforward.
