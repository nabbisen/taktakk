//! Content-addressed object store backed by the filesystem.
//!
//! Objects are stored at `object_store/<first2>/<remaining>` (same layout
//! as Git's loose objects). A separate `quarantine/` sub-directory holds
//! objects that failed integrity checks.

use std::path::PathBuf;

use taktakk_core::error::{CoreError, CoreResult};
use taktakk_core::ports::package_store::ObjectStore;

/// Filesystem-backed content-addressed object store.
pub struct FsObjectStore {
    base: PathBuf,
}

impl FsObjectStore {
    pub fn new(base: PathBuf) -> Self {
        Self { base }
    }

    fn object_path(&self, sha256_hex: &str) -> PathBuf {
        let (prefix, rest) = sha256_hex.split_at(2);
        self.base.join(prefix).join(rest)
    }

    fn quarantine_path(&self, sha256_hex: &str) -> PathBuf {
        self.base.join("quarantine").join(sha256_hex)
    }
}

impl ObjectStore for FsObjectStore {
    fn put(&self, data: &[u8]) -> CoreResult<String> {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(data));
        let path = self.object_path(&hash);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        // Atomic write: write to a temp file, then rename.
        let tmp = path.with_extension("tmp");
        std::fs::write(&tmp, data)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        Ok(hash)
    }

    fn get(&self, sha256_hex: &str) -> CoreResult<Vec<u8>> {
        let path = self.object_path(sha256_hex);
        let data = std::fs::read(&path)
            .map_err(|_| CoreError::Storage(format!("object not found: {sha256_hex}")))?;

        // Verify hash on read.
        use sha2::{Digest, Sha256};
        let actual = hex::encode(Sha256::digest(&data));
        if actual != sha256_hex {
            return Err(CoreError::HashMismatch { hash: sha256_hex.to_string() });
        }
        Ok(data)
    }

    fn exists(&self, sha256_hex: &str) -> CoreResult<bool> {
        Ok(self.object_path(sha256_hex).exists())
    }

    fn quarantine(&self, sha256_hex: &str, _reason: &str) -> CoreResult<()> {
        let src = self.object_path(sha256_hex);
        let dst = self.quarantine_path(sha256_hex);
        if let Some(p) = dst.parent() {
            std::fs::create_dir_all(p)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        if src.exists() {
            std::fs::rename(&src, &dst)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    fn delete(&self, sha256_hex: &str) -> CoreResult<()> {
        let path = self.object_path(sha256_hex);
        if path.exists() {
            std::fs::remove_file(&path)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        Ok(())
    }
}
