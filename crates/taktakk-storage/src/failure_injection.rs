//! Failure injection test harness (RFC 031).
//!
//! Simulates the failures that occur in taktakk's target environments:
//! power loss mid-write, corrupt packages, low storage, interrupted sync.
//!
//! Used by `xtask field-failure-tests`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use taktakk_core::error::{CoreError, CoreResult};
use taktakk_core::ports::package_store::ObjectStore;

// ── Write-limited object store ────────────────────────────────────────────────

/// An `ObjectStore` wrapper that fails after `fail_after` successful writes.
///
/// Simulates power loss or storage-full conditions during package installation.
pub struct FaultInjectingStore<S: ObjectStore> {
    inner: S,
    write_count: Arc<AtomicU32>,
    fail_after: u32,
}

impl<S: ObjectStore> FaultInjectingStore<S> {
    pub fn new(inner: S, fail_after: u32) -> Self {
        Self {
            inner,
            write_count: Arc::new(AtomicU32::new(0)),
            fail_after,
        }
    }

    pub fn writes_completed(&self) -> u32 {
        self.write_count.load(Ordering::SeqCst)
    }
}

impl<S: ObjectStore + Send + Sync> ObjectStore for FaultInjectingStore<S> {
    fn put(&self, data: &[u8]) -> CoreResult<String> {
        let count = self.write_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.fail_after {
            return Err(CoreError::Storage(format!(
                "simulated write failure after {} writes", self.fail_after
            )));
        }
        self.inner.put(data)
    }

    fn get(&self, hash: &str) -> CoreResult<Vec<u8>> {
        self.inner.get(hash)
    }

    fn exists(&self, hash: &str) -> CoreResult<bool> {
        self.inner.exists(hash)
    }

    fn quarantine(&self, hash: &str, reason: &str) -> CoreResult<()> {
        self.inner.quarantine(hash, reason)
    }

    fn delete(&self, hash: &str) -> CoreResult<()> {
        self.inner.delete(hash)
    }

    fn stage(&self, install_id: &str, data: &[u8]) -> CoreResult<String> {
        let count = self.write_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.fail_after {
            return Err(CoreError::Storage(format!(
                "simulated write failure after {} writes", self.fail_after
            )));
        }
        self.inner.stage(install_id, data)
    }

    fn commit_staging(&self, install_id: &str) -> CoreResult<()> {
        self.inner.commit_staging(install_id)
    }

    fn abort_staging(&self, install_id: &str) -> CoreResult<()> {
        self.inner.abort_staging(install_id)
    }

    fn list_staging_ids(&self) -> CoreResult<Vec<String>> {
        self.inner.list_staging_ids()
    }
}

// ── Corrupt package generator ─────────────────────────────────────────────────

/// Failure class enumeration for test parameterisation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FailureClass {
    /// Package magic bytes corrupted.
    CorruptMagic,
    /// Manifest bytes truncated.
    TruncatedManifest,
    /// Signature bytes zeroed.
    ZeroedSignature,
    /// Object data tampered after signing.
    TamperedObject,
    /// Empty file.
    EmptyFile,
    /// Random noise.
    RandomNoise,
}

/// Generate a corrupt `.nmp`-like byte sequence for testing failure handling.
pub fn generate_corrupt_package(class: &FailureClass) -> Vec<u8> {
    use taktakk_core::domain::package::NMP_MAGIC;
    match class {
        FailureClass::CorruptMagic => b"JPEG\xff\xd8\xff\xe0".to_vec(),
        FailureClass::TruncatedManifest => {
            let mut v = NMP_MAGIC.to_vec();
            v.push(1); // version
            v.extend_from_slice(&5u32.to_be_bytes()); // manifest length = 5
            v.extend_from_slice(b"TRUNC"); // only 5 bytes, no signature follows
            v
        }
        FailureClass::ZeroedSignature => {
            let mut v = NMP_MAGIC.to_vec();
            v.push(1);
            let manifest = b"{}";
            v.extend_from_slice(&(manifest.len() as u32).to_be_bytes());
            v.extend_from_slice(manifest);
            v.extend_from_slice(&64u32.to_be_bytes());
            v.extend_from_slice(&[0u8; 64]); // zeroed signature
            v.extend_from_slice(&0u32.to_be_bytes()); // 0 objects
            v
        }
        FailureClass::TamperedObject => {
            // Valid structure but last bytes flipped.
            let mut v = NMP_MAGIC.to_vec();
            v.push(1);
            let manifest = b"{\"module_id\":\"x\"}";
            v.extend_from_slice(&(manifest.len() as u32).to_be_bytes());
            v.extend_from_slice(manifest);
            v.extend_from_slice(&64u32.to_be_bytes());
            v.extend_from_slice(&[0xABu8; 64]);
            v.extend_from_slice(&1u32.to_be_bytes());
            let obj_data = b"tampered_data";
            v.extend_from_slice(&(obj_data.len() as u32).to_be_bytes());
            v.extend_from_slice(obj_data);
            // Flip last 2 bytes to simulate tampering.
            let len = v.len();
            v[len - 1] ^= 0xFF;
            v[len - 2] ^= 0xFF;
            v
        }
        FailureClass::EmptyFile => vec![],
        FailureClass::RandomNoise => (0u8..=127u8).cycle().take(256).collect(),
    }
}

// ── Power-loss simulation ─────────────────────────────────────────────────────

/// Simulate a mid-write power loss by writing only `partial_bytes` of `data`.
///
/// Returns the path to the partially written file and its hash (of partial data).
pub fn write_partial(dir: &Path, data: &[u8], partial_bytes: usize) -> std::io::Result<PathBuf> {
    use sha2::{Digest, Sha256};
    let hash = hex::encode(Sha256::digest(data));
    let path = dir.join(format!("{hash}.partial"));
    let truncated = &data[..partial_bytes.min(data.len())];
    std::fs::write(&path, truncated)?;
    Ok(path)
}

/// Check whether a directory contains any leftover `.partial` staging files.
pub fn has_partial_files(dir: &Path) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries.flatten().any(|e| {
                e.path().extension().and_then(|x| x.to_str()) == Some("partial")
            })
        })
        .unwrap_or(false)
}

/// Clean up `.partial` files left behind by interrupted operations.
pub fn cleanup_partial_files(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|x| x.to_str()) == Some("partial") {
                if std::fs::remove_file(&path).is_ok() {
                    count += 1;
                }
            }
        }
    }
    count
}
