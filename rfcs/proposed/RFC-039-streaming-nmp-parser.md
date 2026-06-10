# RFC-039: Streaming `.nmp` Parser and Buffer-Bounded Import

| Field | Value |
|---|---|
| **Status** | Proposed |
| **Created** | 2026-05-09 |
| **Milestone** | M10 (remediation sprint) |
| **Priority** | P0 — release blocker |
| **Review finding** | Functional §4–5; Non-functional §7–9 |

## Problem

### `.nmp` parser — full load

`parse(data: &[u8])` copies the entire package into memory as
`Vec<Vec<u8>>`. On a 1 GB RAM device with a 50 MB package, this can
allocate > 100 MB (input + parsed copy + pending objects).

`NmpReader::new(data)` calls `parse(data)?` internally, so the
"streaming" API is not actually streaming.

### `import.rs` — comment/code mismatch

`read_package_file()` has the comment "buffered to avoid OOM on devices
with < 1 GiB RAM" but is implemented as `std::fs::read(path)`, which
reads the entire file into a `Vec<u8>`. The comment is wrong.

### `object_store` — bulk get

`FsObjectStore::put(data: &[u8])` / `get() -> Vec<u8>` loads entire
objects into memory. For Wasm and audio objects this becomes problematic
once encryption (RFC-037) adds decryption buffers on top.

## Design

### `NmpStreamReader<R: Read>`

```rust
pub struct NmpStreamReader<R: Read> {
    reader: BufReader<R>,
    pub manifest: PackageManifest,   // eagerly parsed (typically ≤ 4 KB)
    signature: [u8; 64],
    remaining_objects: u32,
}

impl<R: Read> NmpStreamReader<R> {
    /// Parse magic + version + manifest + signature.
    /// Returns Err if manifest exceeds MAX_MANIFEST_BYTES (16 KiB).
    pub fn open(reader: R) -> ContentResult<Self>;

    /// Yield one object at a time: (path, hash, type, reader).
    /// The caller is responsible for reading and verifying the bytes.
    pub fn next_object(&mut self) -> ContentResult<Option<ObjectEntry>>;
}

pub struct ObjectEntry<'a, R: Read> {
    pub path: String,
    pub expected_hash: String,
    pub object_type: ObjectType,
    pub expected_size: u32,
    pub reader: &'a mut BufReader<R>,
}
```

Size limits enforced during streaming:
- `MAX_MANIFEST_BYTES = 16 * 1024`
- `MAX_OBJECT_BYTES = 20 * 1024 * 1024` (20 MiB per object)
- `MAX_OBJECT_COUNT = 1024`
- `MAX_PACKAGE_BYTES = 50 * 1024 * 1024` (50 MiB total)

### `install_package_stream`

The install API accepts a stream:

```rust
pub fn install_package_stream<R: Read>(
    reader: R,
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    object_store: &dyn ObjectStore,
    now: i64,
) -> InstallOutcome;
```

Internally:
1. `NmpStreamReader::open(reader)` — reads manifest only.
2. Verify signature against trust anchors (no object data loaded yet).
3. For each `ObjectEntry`, stream through SHA-256 hasher while writing
   to `object_store/staging/<install_id>/<hash>`.
4. If any hash mismatches, abort and delete all staging objects.
5. Return `InstallOutcome::Installed` or `::Quarantined`.

The existing `install_package(raw: &[u8], ...)` becomes a thin wrapper
calling `install_package_stream(Cursor::new(raw), ...)` for test fixtures.

### `open_package_stream` in import.rs

```rust
pub fn open_package_stream(
    path: &Path,
    max_bytes: u64,
) -> std::io::Result<impl Read> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::FileTooLarge,
            format!("package exceeds limit: {} > {}", meta.len(), max_bytes),
        ));
    }
    Ok(BufReader::new(std::fs::File::open(path)?))
}
```

`read_package_file()` is removed from the public API (or deprecated to
test-only).

### `put_stream` / `get_stream` on ObjectStore

```rust
pub trait ObjectStore: Send + Sync {
    fn put(&self, data: &[u8]) -> CoreResult<String>;       // keep for small objects
    fn put_stream(&self, reader: &mut dyn Read, expected_size: u64) -> CoreResult<String>;
    fn get(&self, hash: &str) -> CoreResult<Vec<u8>>;       // keep for small objects
    fn get_stream(&self, hash: &str) -> CoreResult<Box<dyn Read>>;
    fn exists(&self, hash: &str) -> CoreResult<bool>;
    fn quarantine(&self, hash: &str, reason: &str) -> CoreResult<()>;
    fn delete(&self, hash: &str) -> CoreResult<()>;
}
```

`audio` and `wasm` objects should always use the stream variants.

## Acceptance criteria

1. `NmpStreamReader::open()` on a 50 MB package heap-allocates < 100 KB
   (manifest only) before any object is processed.
2. An object that exceeds `MAX_OBJECT_BYTES` is rejected mid-stream with
   `ContentError::ObjectTooLarge`; no partial object is written to the
   object store.
3. A package with `MAX_OBJECT_COUNT + 1` objects is rejected with
   `ContentError::TooManyObjects`.
4. `open_package_stream()` rejects files > 50 MiB with `FileTooLarge`.
5. The existing 18 content tests continue to pass via the thin wrapper.
6. `cargo test -p taktakk-content -- streaming` includes a test that
   verifies peak RSS does not exceed 4 × manifest size for a 10 MB package.
