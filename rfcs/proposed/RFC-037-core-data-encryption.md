# RFC-037: core.sqlite + object_store Encryption

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Review finding** | Functional §2; Non-functional §1 |

## Problem

`core.sqlite` is opened as plain SQLite. `object_store` objects are written
with `std::fs::write()`. On a seized device, all of the following are
immediately readable without any key knowledge:

- Learning progress and resume state
- Installed package metadata and trust anchors
- Lesson content objects (SVGs, audio hashes, JSON steps)
- Event log entries
- Profile identifiers

This means RFC-018's "key-first wipe" cannot fulfil its promise: destroying
the key slot does not make the data unreadable because the data is stored
in plaintext.

## Design

### Option A — SQLCipher (preferred for core.sqlite)

Replace `sqlx`'s default SQLite driver with a SQLCipher-linked build.
The master key (derived from the unlocked slot) is passed as a PRAGMA:

```sql
PRAGMA key = 'x''<32-byte-hex>''';
```

The application never has a `PRAGMA rekey` call in normal operation.
SQLCipher encrypts every 4 KiB page with AES-256-CBC + per-page HMAC.

**Pros:** transparent to the repository layer; WAL pages also encrypted.
**Cons:** requires native linking; cross-compilation to Android adds
complexity; SQLCipher licence (BSD) must be added to NOTICE.

### Option B — Application-level AES-GCM per row / per object (fallback)

For deployments where SQLCipher is unavailable:

- `core.sqlite` stores `ciphertext BLOB` + `nonce BLOB` instead of
  raw column values in sensitive tables.
- A per-table DEK (Data Encryption Key) is wrapped with the master key
  stored in `key_registry`.
- `object_store` objects are encrypted with a per-object AES-256-GCM key
  derived from `HKDF(master_key, "obj:" || object_hash)`.

`content_objects` gains three columns:

```sql
encryption_alg   TEXT NOT NULL DEFAULT 'aes256gcm',
key_id           TEXT NOT NULL,
nonce_hex        TEXT NOT NULL
```

### Object store encryption

Regardless of Option A or B, every object file written by `FsObjectStore`
must be encrypted. The canonical approach:

```
file_on_disk = AES-256-GCM(key=DEK, nonce=random_12_bytes, plaintext=object_bytes)
              || nonce (12 bytes) || tag (16 bytes)
```

The DEK is either the master key (simple) or a per-object derived key.

### WAL residue

With SQLCipher, WAL pages are encrypted automatically. With Option B,
add `PRAGMA secure_delete = ON` as a belt-and-suspenders measure and
checkpoint + truncate WAL before entering factory-reset state.

### Key lifecycle

The master key lives in memory only after unlock. It is derived via
Argon2id from the user's unlock sequence and stored as a wrapped blob
in `slot_config.verifier_blob` (facade.sqlite). After wipe, the
`wrapped_blob` is overwritten (7 passes); without it, the master key
cannot be reconstructed.

## Acceptance criteria

1. `cargo test -p taktakk-storage -- encryption` — a test opens `core.sqlite`
   with the wrong key and receives a failure (not silent success).
2. After `destroy_key_slots()`, any attempt to re-open `core.sqlite` with the
   original connection fails to decrypt.
3. After `factory_reset()`, a fresh open of `core.sqlite` finds empty tables
   (not errors): the wipe path re-creates an empty encrypted DB.
4. `object_store` test: store an object, wipe the key, attempt `get()` →
   returns `Err(StorageError::DecryptionFailed)`.
5. `cargo test -p taktakk-integration -- encryption` passes the end-to-end
   proof: key erasure → data unreadable.
6. No regression: existing 266 tests continue to pass.

## Implementation notes

- Add `sqlcipher` feature flag to `taktakk-storage/Cargo.toml` defaulting off;
  enabled in release builds.
- Migration path: first launch after upgrade detects unencrypted DB, encrypts
  in place (or wipes and re-seeds from seed kit).
- Object store: add `put_encrypted()` / `get_decrypted()` wrappers that
  keep the current `put()` / `get()` API for test fixtures.
