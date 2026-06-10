# Module Authoring

## Authoring constraints

These constraints are non-negotiable for any module submitted to the
taktakk content ecosystem.

1. **Every critical step must have a pictogram or SVG.** Text is secondary.
2. **Provide audio for every step** in every locale you claim to support.
3. **Keep lessons short**: 5–8 steps per lesson maximum.
4. **No video**. Video requires too much storage and battery.
5. **No remote URLs**. Lessons must work with zero network access.
6. **No sensitive user answers stored**. Exercise results record only
   correct/incorrect, never the actual answer text.

## RTL/LTR layout rules

Your SVGs must work in both LTR and RTL layouts. Use symmetrical
compositions where possible. Avoid:

- Text embedded in SVG (use `text_key` references and locale packs instead).
- Arrows or directional elements that carry sequential meaning.
- Left/right spatial positioning to indicate order of steps.

Icons for universal concepts (water, first aid cross, emergency exit) must
**never** be mirrored in RTL. Only navigation arrows and list-direction
indicators should mirror.

## Step content types

| `content_type` | Description |
|---|---|
| `text` | Display a localised text block with optional audio |
| `svg_placeholder` | Display an SVG from the object store |
| `acknowledge` | User taps to confirm they have understood |
| `multiple_choice` | Choose one correct answer from 2–4 options |
| `ordering` | Drag items into the correct sequence |
| `wasm` | Sandboxed interactive exercise |

## Signing a package

Use the taktakk CLI (once available) or the `NmpWriter` API in
`taktakk-content::nmp::writer`:

```rust
let mut writer = NmpWriter::new(manifest);
writer.add_object("steps/step-00.json", data, ObjectType::Json);
let nmp_bytes = writer.build(|manifest_bytes| {
    signing_key.sign(manifest_bytes).to_bytes()
})?;
```

The signer must correspond to a `TrustAnchor` installed on target devices.

## Content review checklist

Before submitting a module:

- [ ] All steps have a pictogram or SVG.
- [ ] Audio provided for all steps in all claimed locales.
- [ ] Manifest SHA-256 hashes are correct for all objects.
- [ ] Tested on a device with 1 GB RAM and slow storage.
- [ ] No text embedded in SVGs.
- [ ] No URLs, phone numbers, or organisation names in content.
- [ ] RTL layout tested with Arabic locale.
- [ ] Package signed with the correct trust anchor key.
