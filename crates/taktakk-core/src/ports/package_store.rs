//! Package store port: content-addressed object storage.

use crate::error::CoreResult;

/// Content-addressed binary object store.
///
/// Objects are identified by their SHA-256 hash (hex-encoded).
/// Paths within the store are determined by the implementation.
pub trait ObjectStore: Send + Sync {
    /// Store raw object data, addressed by its SHA-256 hash.
    /// Returns the hex-encoded hash that can be used to retrieve the object.
    fn put(&self, data: &[u8]) -> CoreResult<String>;

    /// Retrieve an object by its SHA-256 hash.
    fn get(&self, sha256_hex: &str) -> CoreResult<Vec<u8>>;

    /// Check whether an object exists without retrieving its data.
    fn exists(&self, sha256_hex: &str) -> CoreResult<bool>;

    /// Move an object to the quarantine area.
    fn quarantine(&self, sha256_hex: &str, reason: &str) -> CoreResult<()>;

    /// Delete an object permanently.
    fn delete(&self, sha256_hex: &str) -> CoreResult<()>;

    // ── Staged install support (RFC-040) ──────────────────────────────────

    /// Write object data to `staging/<install_id>/<hash>`.
    ///
    /// Staged objects are not yet part of the live store; they are promoted
    /// atomically by `commit_staging()` after the DB transaction succeeds.
    fn stage(&self, install_id: &str, data: &[u8]) -> CoreResult<String>;

    /// Promote all staged objects for `install_id` to the live store.
    ///
    /// Called after the DB transaction commits successfully.
    fn commit_staging(&self, install_id: &str) -> CoreResult<()>;

    /// Delete all staged objects for `install_id` without promoting.
    ///
    /// Called when the install fails or is aborted.
    fn abort_staging(&self, install_id: &str) -> CoreResult<()>;

    /// List all install IDs that have staged objects.
    ///
    /// Used by crash recovery to find orphaned staging directories.
    fn list_staging_ids(&self) -> CoreResult<Vec<String>>;
}
