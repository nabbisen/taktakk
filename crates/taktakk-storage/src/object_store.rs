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

    // ── Staging support (RFC-040) ─────────────────────────────────────────

    fn stage(&self, install_id: &str, data: &[u8]) -> CoreResult<String> {
        use sha2::{Digest, Sha256};
        let hash = hex::encode(Sha256::digest(data));
        let dir = self.base.join("staging").join(install_id);
        std::fs::create_dir_all(&dir)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        let path = dir.join(&hash);
        let tmp  = dir.join(format!("{hash}.tmp"));
        std::fs::write(&tmp, data)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        std::fs::rename(&tmp, &path)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(hash)
    }

    fn commit_staging(&self, install_id: &str) -> CoreResult<()> {
        let staging_dir = self.base.join("staging").join(install_id);
        if !staging_dir.exists() {
            return Ok(());
        }
        let entries = std::fs::read_dir(&staging_dir)
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        for entry in entries.flatten() {
            let hash = entry.file_name().to_string_lossy().into_owned();
            if hash.ends_with(".tmp") { continue; }
            let dst = self.object_path(&hash);
            if let Some(p) = dst.parent() {
                std::fs::create_dir_all(p)
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
            }
            if !dst.exists() {
                std::fs::rename(entry.path(), &dst)
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
            }
        }
        let _ = std::fs::remove_dir_all(&staging_dir);
        Ok(())
    }

    fn abort_staging(&self, install_id: &str) -> CoreResult<()> {
        let staging_dir = self.base.join("staging").join(install_id);
        if staging_dir.exists() {
            std::fs::remove_dir_all(&staging_dir)
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    fn list_staging_ids(&self) -> CoreResult<Vec<String>> {
        let staging_root = self.base.join("staging");
        if !staging_root.exists() {
            return Ok(vec![]);
        }
        let ids = std::fs::read_dir(&staging_root)
            .map_err(|e| CoreError::Storage(e.to_string()))?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        Ok(ids)
    }
}
