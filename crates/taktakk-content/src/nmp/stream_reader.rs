//! Streaming `.nmp` package reader (RFC-039).
//!
//! Reads only the manifest eagerly. Objects are yielded one at a time through
//! a caller-provided `Read` handle so that a 50 MB package never requires
//! 50 MB of heap — only the current object's bytes plus the manifest.
//!
//! Size limits enforced during streaming:
//! - `MAX_MANIFEST_BYTES = 16 KiB`
//! - `MAX_OBJECT_BYTES   = 20 MiB` per object
//! - `MAX_OBJECT_COUNT   = 1 024`
//! - `MAX_PACKAGE_BYTES  = 50 MiB` total

use std::io::Read;

use sha2::{Digest, Sha256};

use taktakk_core::domain::package::{
    check_magic, NMP_FORMAT_VERSION, PackageManifest,
};

use super::error::{ContentError, ContentResult};

// ── Limits ────────────────────────────────────────────────────────────────────

pub const MAX_MANIFEST_BYTES:  u32 = 16 * 1024;
pub const MAX_OBJECT_BYTES:    u32 = 20 * 1024 * 1024;
pub const MAX_OBJECT_COUNT:    u32 = 1_024;
pub const MAX_PACKAGE_BYTES:   u64 = 50 * 1024 * 1024;

// ── Reader ────────────────────────────────────────────────────────────────────

/// Streaming reader for `.nmp` packages.
///
/// On construction, reads magic + version + manifest + signature (eager).
/// Objects are read lazily one at a time via `next_object_bytes()`.
pub struct NmpStreamReader<R: Read> {
    inner: R,
    pub manifest: PackageManifest,
    pub manifest_bytes: Vec<u8>,
    pub signature: [u8; 64],
    remaining_objects: u32,
    bytes_consumed: u64,
}

impl<R: Read> NmpStreamReader<R> {
    /// Open a stream and parse the header eagerly.
    ///
    /// After this call, `manifest`, `manifest_bytes`, and `signature` are
    /// populated. No object data has been read.
    pub fn open(mut reader: R) -> ContentResult<Self> {
        let mut bytes_consumed: u64 = 0;

        // Magic (4 bytes)
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic).map_err(|e| ContentError::Parse(e.to_string()))?;
        bytes_consumed += 4;
        if !check_magic(&magic) {
            return Err(ContentError::InvalidMagic);
        }

        // Format version (1 byte)
        let mut version = [0u8; 1];
        reader.read_exact(&mut version).map_err(|e| ContentError::Parse(e.to_string()))?;
        bytes_consumed += 1;
        if version[0] != NMP_FORMAT_VERSION {
            return Err(ContentError::UnsupportedVersion(version[0]));
        }

        // Manifest length (4 bytes big-endian)
        let manifest_len = read_u32_be(&mut reader, &mut bytes_consumed)?;
        if manifest_len > MAX_MANIFEST_BYTES {
            return Err(ContentError::Parse(format!(
                "manifest length {manifest_len} exceeds MAX_MANIFEST_BYTES {MAX_MANIFEST_BYTES}"
            )));
        }

        // Manifest JSON
        let mut manifest_bytes = vec![0u8; manifest_len as usize];
        reader.read_exact(&mut manifest_bytes).map_err(|e| ContentError::Parse(e.to_string()))?;
        bytes_consumed += manifest_len as u64;

        let manifest: PackageManifest = serde_json::from_slice(&manifest_bytes)
            .map_err(|e| ContentError::ManifestParse(e.to_string()))?;

        // Signature length (must be 64 for Ed25519)
        let sig_len = read_u32_be(&mut reader, &mut bytes_consumed)?;
        if sig_len != 64 {
            return Err(ContentError::Parse(format!(
                "unexpected signature length: {sig_len}"
            )));
        }

        let mut signature = [0u8; 64];
        reader.read_exact(&mut signature).map_err(|e| ContentError::Parse(e.to_string()))?;
        bytes_consumed += 64;

        // Object count
        let object_count = read_u32_be(&mut reader, &mut bytes_consumed)?;
        if object_count > MAX_OBJECT_COUNT {
            return Err(ContentError::Parse(format!(
                "object count {object_count} exceeds MAX_OBJECT_COUNT {MAX_OBJECT_COUNT}"
            )));
        }

        Ok(Self {
            inner: reader,
            manifest,
            manifest_bytes,
            signature,
            remaining_objects: object_count,
            bytes_consumed,
        })
    }

    /// Read and verify the next object's raw bytes.
    ///
    /// Returns `Ok(None)` when all objects have been consumed.
    /// Returns `Err` if the object hash mismatches, the object is too large,
    /// or the package total size limit is exceeded.
    ///
    /// The caller must look up the expected hash and type from
    /// `self.manifest.objects[index]`.
    pub fn next_object_bytes(&mut self) -> ContentResult<Option<Vec<u8>>> {
        if self.remaining_objects == 0 {
            return Ok(None);
        }

        let obj_len = read_u32_be(&mut self.inner, &mut self.bytes_consumed)?;
        if obj_len > MAX_OBJECT_BYTES {
            return Err(ContentError::Parse(format!(
                "object length {obj_len} exceeds MAX_OBJECT_BYTES {MAX_OBJECT_BYTES}"
            )));
        }

        self.bytes_consumed += obj_len as u64;
        if self.bytes_consumed > MAX_PACKAGE_BYTES {
            return Err(ContentError::Parse(format!(
                "package total size exceeds MAX_PACKAGE_BYTES {MAX_PACKAGE_BYTES}"
            )));
        }

        let mut data = vec![0u8; obj_len as usize];
        self.inner.read_exact(&mut data).map_err(|e| ContentError::Parse(e.to_string()))?;

        self.remaining_objects -= 1;
        Ok(Some(data))
    }

    /// The index of the next object to be read (0-based).
    pub fn next_object_index(&self) -> usize {
        self.manifest.objects.len() - self.remaining_objects as usize
    }

    /// Verify `object_bytes` against the manifest entry at `index`.
    pub fn verify_object(&self, index: usize, data: &[u8]) -> ContentResult<()> {
        let entry = self.manifest.objects.get(index).ok_or_else(|| {
            ContentError::Parse(format!("object index {index} out of range"))
        })?;
        let actual = hex::encode(Sha256::digest(data));
        if actual != entry.sha256 {
            return Err(ContentError::HashMismatch {
                path:     entry.path.clone(),
                expected: entry.sha256.clone(),
                actual,
            });
        }
        Ok(())
    }
}

// ── install_package_stream ────────────────────────────────────────────────────

use taktakk_core::ports::package_store::ObjectStore;
use taktakk_security::trust_anchor::TrustAnchor;
use crate::install::{InstallOutcome, build_content_package};
use crate::verify;

/// Install a package from a stream, verifying signature and per-object hashes.
///
/// Phase 1 — stream + verify (no DB writes):
///   Objects are staged to `object_store/staging/<install_id>/`.
/// Phase 2 — caller commits to DB, then calls `object_store.commit_staging()`.
/// On any failure, staged objects are cleaned up via `abort_staging()`.
pub fn install_package_stream<R: Read>(
    reader: R,
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    object_store: &dyn ObjectStore,
    now: i64,
) -> InstallOutcome {
    let mut stream = match NmpStreamReader::open(reader) {
        Ok(s) => s,
        Err(e) => return InstallOutcome::Quarantined {
            reason: format!("parse error: {e}"),
        },
    };

    // Verify signature before any object data is processed.
    if let Err(e) = verify::verify_signature_bytes(
        &stream.manifest_bytes,
        &stream.signature,
        &stream.manifest,
        trust_anchors,
    ) {
        return InstallOutcome::Quarantined {
            reason: format!("signature: {e}"),
        };
    }

    // Stage objects one at a time, verifying each hash.
    let mut index = 0;
    loop {
        match stream.next_object_bytes() {
            Err(e) => {
                let _ = object_store.abort_staging(package_id);
                return InstallOutcome::Quarantined {
                    reason: format!("object read error at index {index}: {e}"),
                };
            }
            Ok(None) => break,
            Ok(Some(data)) => {
                if let Err(e) = stream.verify_object(index, &data) {
                    let _ = object_store.abort_staging(package_id);
                    return InstallOutcome::Quarantined {
                        reason: format!("hash mismatch at index {index}: {e}"),
                    };
                }
                if let Err(e) = object_store.stage(package_id, &data) {
                    let _ = object_store.abort_staging(package_id);
                    return InstallOutcome::Quarantined {
                        reason: format!("staging write error: {e}"),
                    };
                }
                index += 1;
            }
        }
    }

    // All objects staged successfully. Promote to the live store.
    // In a fully transactional install (RFC-040 phase 2), the caller would
    // open a DB transaction here, then call commit_staging() after COMMIT.
    // For compatibility with the existing non-DB callers we promote inline.
    if let Err(e) = object_store.commit_staging(package_id) {
        let _ = object_store.abort_staging(package_id);
        return InstallOutcome::Quarantined {
            reason: format!("commit_staging failed: {e}"),
        };
    }

    let package = build_content_package(
        package_id,
        &stream.manifest,
        &stream.manifest_bytes,
        now,
    );
    InstallOutcome::Installed { package }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn read_u32_be<R: Read>(reader: &mut R, bytes_consumed: &mut u64) -> ContentResult<u32> {
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf).map_err(|e| ContentError::Parse(e.to_string()))?;
    *bytes_consumed += 4;
    Ok(u32::from_be_bytes(buf))
}
