# RFC-047: Trust Anchor Label Privacy

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P1 |
| **Review finding** | Non-functional §5 |

## Problem

`trust_anchors.label TEXT NOT NULL` in `core.sqlite` and
`TrustAnchor { label: String }` in `taktakk-security` can contain an
organisation's real name (e.g. "UNHCR Kenya", "MSF Field Operations").

If the device is seized, an attacker can read `core.sqlite` (before RFC-037
is applied) or the object store and identify the aid organisation that
distributed the content. This endangers field staff.

## Design

### Remove `label` from the on-device DB

The on-device `trust_anchors` table stores only:

```sql
CREATE TABLE IF NOT EXISTS trust_anchors (
    signing_key_id   TEXT PRIMARY KEY NOT NULL,
    public_key_bytes BLOB NOT NULL,
    scope            TEXT NOT NULL,    -- 'content' | 'locale' | 'revocation'
    added_at         INTEGER NOT NULL,
    status           TEXT NOT NULL,
    revoked_at       INTEGER
);
```

No `label` column.

### Label lives in seed-kit manifest only

The seed-kit manifest (on the distributor's machine, never on the device)
contains the label:

```json
{
  "anchors": [
    {
      "signing_key_id": "abc123",
      "label": "Org Name (not stored on device)",
      "public_key_hex": "..."
    }
  ]
}
```

### `TrustAnchor` struct change

```rust
pub struct TrustAnchor {
    pub signing_key_id: String,
    pub public_key: VerifyingKey,
    pub scope: TrustAnchorScope,
    pub status: TrustAnchorStatus,
    pub added_at: i64,
    pub revoked_at: Option<i64>,
    // No `label` field.
}
```

### Documentation

`docs/src/guide/security-audit.md` gains a rule:
> "Never use an organisation's real name as a trust anchor label in any
> data stored on a device. Labels are operator-facing metadata only and
> must remain on the distributor's seed-kit machine."

## Acceptance criteria

1. `trust_anchors` schema has no `label` column (migration removes it).
2. `TrustAnchor` struct has no `label` field; any code that used it
   fails to compile (compiler-enforced removal).
3. `cargo test -p taktakk-security -- trust_anchor_no_label` passes.
4. The seed-kit manifest schema (RFC-028) retains a `label` field for
   operator use, documented as "not stored on device".
5. `docs/src/guide/security-audit.md` contains the label-policy rule.
