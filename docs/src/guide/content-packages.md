# Content Packages

## The `.nmp` format

A `.nmp` (taktakk Module Package) is a binary bundle that carries an entire
learning module. Every package is cryptographically signed and content-addressed.

### Wire structure

```
[0..4]        Magic bytes: 0x54 0x41 0x4B 0x54  ("TAKT")
[4]           Format version: u8  (current: 1)
[5..9]        Manifest length: u32 big-endian
[9..9+M]      Manifest JSON
[9+M..9+M+4]  Signature length: u32 big-endian  (always 64 for Ed25519)
[9+M+4..9+M+68]  Ed25519 signature over manifest bytes
[9+M+68..9+M+72] Object count: u32 big-endian
For each object:
  [0..4]    Object data length: u32 big-endian
  [4..4+N]  Raw object bytes
```

The signature covers **only the manifest bytes**. This allows signature
verification before any object data is loaded into memory.

### Manifest structure

```json
{
  "module_id": "shield-water-purification",
  "version": { "major": 1, "minor": 0, "patch": 0 },
  "min_core_version": { "major": 0, "minor": 7, "patch": 0 },
  "signer_id": "your-signing-key-id",
  "locales": ["en", "ar", "sw"],
  "objects": [
    {
      "path": "steps/step-00.json",
      "sha256": "<64-char hex>",
      "object_type": "Json",
      "required": true
    }
  ]
}
```

## Verification pipeline

`install_package()` runs these steps in order, quarantining on any failure:

1. Check magic bytes and format version.
2. Verify Ed25519 signature against the trust anchor matching `signer_id`.
3. For each object, verify SHA-256 hash matches the manifest entry.
4. Write all objects to the content-addressed object store.
5. Return `InstallOutcome::Installed` with the package record.

## Trust anchors

Trust anchors are Ed25519 public keys of organisations authorised to publish
taktakk content. They are stored in `core.sqlite` and can be delivered as
special trust-update packages.

See [Security Audit](security-audit.md) for how to verify trust anchor
configuration.

## Object types

| Type | Usage |
|---|---|
| `Json` | Lesson step metadata and curriculum index |
| `Svg` | Pictogram illustrations |
| `Audio` | Opus-encoded audio narration |
| `Wasm` | Interactive exercise modules (sandboxed) |
| `Patch` | Delta update for an existing object |
