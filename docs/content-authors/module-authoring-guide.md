# Module Authoring Guide

## Overview

A taktakk module is a signed `.nmp` package containing:
- A JSON manifest (module ID, version, object hashes, signer ID).
- SVG illustrations (one per lesson step, optional).
- Opus audio files (one per step, optional but strongly recommended).
- A Wasm binary for interactive exercises (optional).

## Authoring constraints (non-negotiable)

1. **Every critical step must have a pictogram or SVG.** Text is secondary.
2. **Provide audio for every lesson step** in every locale you support.
3. **Keep individual lessons short** (5–8 steps maximum).
4. **Use only approved object types:** JSON, SVG, Opus audio, Wasm.
5. **No video.** Video requires too much storage and battery.
6. **No remote URLs.** Lessons must work with zero network access.
7. **No sensitive user answers in state.** Exercise results store only
   correct/incorrect, never the actual answer text.

## RTL/LTR layout

SVGs must be designed to work in both left-to-right and right-to-left
layouts. Use symmetrical compositions where possible. Avoid:
- Text embedded in SVG (use `text_key` references instead).
- Directional arrows that carry meaning (use pictograms instead).
- Left/right spatial positioning to encode sequence order.

## Manifest structure

```json
{
  "module_id": "shield-water-purification",
  "version": { "major": 1, "minor": 0, "patch": 0 },
  "min_core_version": { "major": 0, "minor": 7, "patch": 0 },
  "signer_id": "your-signing-key-id",
  "locales": ["en", "ar", "sw"],
  "objects": [
    {
      "path": "lesson-01-step-01.svg",
      "sha256": "<64-char hex>",
      "object_type": "Svg",
      "required": true
    }
  ]
}
```

## Signing packages

Use the taktakk CLI (when available) to sign your package:

```bash
cargo xtask sign-package --key-id your-key-id --input module.nmp
```

The signing key must correspond to a trust anchor installed on target devices.

## Content review checklist

Before submitting a module for distribution:

- [ ] All steps have a pictogram or SVG illustration.
- [ ] Audio files provided for all steps in all claimed locales.
- [ ] Manifest SHA-256 hashes are correct for all objects.
- [ ] Module tested on a low-end device (1 GB RAM, slow CPU).
- [ ] No text embedded in SVGs.
- [ ] No URLs, phone numbers, or organization names in content.
- [ ] RTL layout tested with Arabic locale.
- [ ] Package signed with the correct trust anchor key.
