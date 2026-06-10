# Sample Content Packages

This directory contains sample `.nmp` content packages for development and testing.

All packages are signed with the **test fixture keypair** defined in
`taktakk_content::fixtures`. **Do not use in production.**

## Packages

| File | Module ID | Axis | Locale | Steps |
|---|---|---|---|---|
| `shield-water-purification-1.0.0.nmp` | `shield-water-purification` | Shield | en/ar/sw | 5 |
| `spear-basic-math-1.0.0.nmp` | `spear-basic-math` | Spear | en | 4 |

## Generating packages

```bash
cargo run -p taktakk-linux -- generate-samples
```

Or run the sample content integration tests:

```bash
cargo test -p taktakk-linux
```
