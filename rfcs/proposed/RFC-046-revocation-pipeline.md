# RFC-046: Revocation Pipeline (Real Apply)

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M11 |
| **Priority** | P1 |
| **Review finding** | Non-functional §6 |

## Problem

`taktakk-security::revocation` has `RevocationPackage` and `plan_revocation()`
as a pure decision model. What is missing:

- Processing a `.nmp` package with `package_type = trust-revocation`.
- Verifying the revocation package's own signature (must be master-level anchor).
- Applying `keys_to_revoke` → `UPDATE trust_anchors SET status='revoked'`.
- Applying `packages_to_quarantine` → `UPDATE content_packages SET status='quarantined'`.
- Rejecting any future import signed by a revoked key.
- Persisting the revocation event to the event log.
- A superseding-revocation check (newer revocation for the same key wins).

## Design

### `.nmp` package_type extension

`PackageManifest` gains an optional field:

```rust
pub enum PackageKind {
    Content,           // normal learning module
    LocalePack,        // locale strings
    TrustRevocation,   // revokes keys / quarantines packages
    TrustUpdate,       // adds a new trust anchor
}
```

The install pipeline branches on `PackageKind` before Phase 1 (streaming).

### `apply_revocation_package()`

```rust
pub async fn apply_revocation_package(
    db: &Database,
    rev: &RevocationPackage,
    anchors: &[TrustAnchor],
    now: i64,
) -> CoreResult<RevocationResult>;

pub struct RevocationResult {
    pub keys_revoked: Vec<String>,
    pub packages_quarantined: Vec<String>,
    pub severity: RevocationSeverity,
}
```

Steps:
1. Verify revocation package signature against a **master-level** trust anchor
   (a separate anchor category, not a content anchor).
2. Open DB transaction.
3. For each `key_id` in `keys_to_revoke`:
   - `UPDATE trust_anchors SET status='revoked', revoked_at=? WHERE signing_key_id=?`
4. For each `hash` in `packages_to_quarantine`:
   - `UPDATE content_packages SET status='quarantined', quarantine_reason='revoked by RFC-046' WHERE manifest_hash=?`
5. Log `EventBucket::WipeOk` (closest approved bucket for a security event).
6. Commit.

### Import gate

In `verify_signature()` (taktakk-content):

```rust
let anchor = trust_anchors
    .iter()
    .find(|a| a.signing_key_id == manifest.signer_id)
    .ok_or(ContentError::UnknownSigner)?;

if anchor.status != TrustAnchorStatus::Active {
    return Err(ContentError::RevokedSigner {
        key_id: manifest.signer_id.clone(),
    });
}
```

This gate already uses `TrustAnchorStatus` — this RFC simply ensures
the DB is updated (step 3 above) before any new import is accepted.

### Superseding revocation

A revocation package with a later `issued_at` for the same `signing_key_id`
wins. Older revocations are silently ignored (idempotent).

## Acceptance criteria

1. `cargo test -p taktakk-integration -- revocation_applies_to_db` —
   after `apply_revocation_package()`, the revoked key's status in
   `trust_anchors` is `'revoked'`.
2. `cargo test -p taktakk-integration -- revoked_key_rejects_import` —
   attempting to install a package signed with the revoked key returns
   `InstallOutcome::Quarantined { reason: "RevokedSigner" }`.
3. A revocation package signed by a non-master-level anchor is rejected.
4. Calling `apply_revocation_package()` twice with the same payload is
   idempotent (no error, no duplicate DB rows).
5. A quarantined package's `content_objects` remain in the object store
   but are inaccessible via the install pipeline.
