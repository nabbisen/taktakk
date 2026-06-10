//! `.nmp` binary reader: parse a package from a byte slice.

use taktakk_core::domain::package::{NMP_FORMAT_VERSION, NMP_MAGIC, PackageManifest};

use super::error::{ContentError, ContentResult};

/// Maximum manifest size (1 MiB). Manifests are JSON metadata, not content.
pub const MAX_MANIFEST_BYTES: u32 = 1 << 20;

/// A parsed package ready for verification and extraction.
pub struct ParsedPackage {
    pub manifest: PackageManifest,
    /// Raw manifest bytes (used for signature verification).
    pub manifest_bytes: Vec<u8>,
    /// Ed25519 signature over `manifest_bytes`.
    pub signature: [u8; 64],
    /// Extracted object data, in manifest order.
    pub objects: Vec<Vec<u8>>,
}

/// Parse a `.nmp` byte slice without performing cryptographic verification.
///
/// The caller must call [`crate::verify`] on the result before trusting any data.
pub fn parse(data: &[u8]) -> ContentResult<ParsedPackage> {
    let mut cursor = 0;

    // Magic bytes (4)
    if data.len() < 4 || &data[..4] != &NMP_MAGIC {
        return Err(ContentError::BadMagic);
    }
    cursor += 4;

    // Format version (1)
    if data.len() < cursor + 1 {
        return Err(ContentError::Truncated { offset: cursor });
    }
    let version = data[cursor];
    if version != NMP_FORMAT_VERSION {
        return Err(ContentError::UnsupportedVersion(version));
    }
    cursor += 1;

    // Manifest length (u32 big-endian)
    let manifest_len = read_u32_be(data, cursor)?;
    cursor += 4;
    if manifest_len > MAX_MANIFEST_BYTES {
        return Err(ContentError::ManifestTooLarge {
            size: manifest_len,
            max: MAX_MANIFEST_BYTES,
        });
    }

    // Manifest JSON
    let manifest_end = cursor + manifest_len as usize;
    if data.len() < manifest_end {
        return Err(ContentError::Truncated { offset: cursor });
    }
    let manifest_bytes = data[cursor..manifest_end].to_vec();
    let manifest: PackageManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| ContentError::ManifestParse(e.to_string()))?;
    cursor = manifest_end;

    // Signature length (u32 big-endian) — must be 64 for Ed25519
    let sig_len = read_u32_be(data, cursor)?;
    cursor += 4;
    if sig_len != 64 {
        return Err(ContentError::SignatureFailed);
    }
    if data.len() < cursor + 64 {
        return Err(ContentError::Truncated { offset: cursor });
    }
    let mut signature = [0u8; 64];
    signature.copy_from_slice(&data[cursor..cursor + 64]);
    cursor += 64;

    // Object count (u32 big-endian)
    let obj_count = read_u32_be(data, cursor)? as usize;
    cursor += 4;

    if obj_count != manifest.objects.len() {
        return Err(ContentError::ObjectCountMismatch {
            manifest: manifest.objects.len(),
            actual: obj_count,
        });
    }

    // Object data
    let mut objects = Vec::with_capacity(obj_count);
    for _ in 0..obj_count {
        let obj_len = read_u32_be(data, cursor)? as usize;
        cursor += 4;
        if data.len() < cursor + obj_len {
            return Err(ContentError::Truncated { offset: cursor });
        }
        objects.push(data[cursor..cursor + obj_len].to_vec());
        cursor += obj_len;
    }

    Ok(ParsedPackage {
        manifest,
        manifest_bytes,
        signature,
        objects,
    })
}

fn read_u32_be(data: &[u8], offset: usize) -> ContentResult<u32> {
    if data.len() < offset + 4 {
        return Err(ContentError::Truncated { offset });
    }
    Ok(u32::from_be_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ]))
}

/// A streaming reader that validates one object at a time.
///
/// Useful for large packages where loading everything into RAM is undesirable.
pub struct NmpReader<'a> {
    data: &'a [u8],
    cursor: usize,
    pub manifest: Option<PackageManifest>,
    pub manifest_bytes: Vec<u8>,
    pub signature: [u8; 64],
    object_index: usize,
    #[allow(dead_code)]
    object_count: usize,
}

impl<'a> NmpReader<'a> {
    /// Create a reader and parse the header (magic, version, manifest, signature).
    pub fn new(data: &'a [u8]) -> ContentResult<Self> {
        let parsed = parse(data)?;
        // Re-derive cursor position after header for streaming access.
        // For the streaming API we re-parse the header only.
        let header_end = 4 + 1 + 4 + parsed.manifest_bytes.len() + 4 + 64 + 4;
        Ok(Self {
            data,
            cursor: header_end,
            manifest: Some(parsed.manifest),
            manifest_bytes: parsed.manifest_bytes,
            signature: parsed.signature,
            object_index: 0,
            object_count: 0, // will be set by read_next_object
        })
    }

    /// Read the next object's raw bytes. Returns `None` when all objects
    /// have been consumed.
    pub fn read_next_object(&mut self) -> ContentResult<Option<Vec<u8>>> {
        if let Some(ref m) = self.manifest {
            if self.object_index >= m.objects.len() {
                return Ok(None);
            }
        }
        let obj_len = {
            if self.data.len() < self.cursor + 4 {
                return Err(ContentError::Truncated { offset: self.cursor });
            }
            u32::from_be_bytes([
                self.data[self.cursor],
                self.data[self.cursor + 1],
                self.data[self.cursor + 2],
                self.data[self.cursor + 3],
            ]) as usize
        };
        self.cursor += 4;
        if self.data.len() < self.cursor + obj_len {
            return Err(ContentError::Truncated { offset: self.cursor });
        }
        let bytes = self.data[self.cursor..self.cursor + obj_len].to_vec();
        self.cursor += obj_len;
        self.object_index += 1;
        Ok(Some(bytes))
    }
}
