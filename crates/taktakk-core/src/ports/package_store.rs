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
}
