//! Transport adapters for offline sync.
//!
//! The `SyncTransportAdapter` trait decouples the sync protocol from
//! transport mechanics. All implementations must work without internet.

pub mod local;

use taktakk_core::error::CoreResult;

/// An object found in the peer's inventory.
#[derive(Debug, Clone)]
pub struct RemoteObject {
    pub object_hash: String,
    pub byte_size: u64,
}

/// Abstract transport adapter for offline content sharing.
pub trait SyncTransportAdapter: Send + Sync {
    /// Human-readable transport name for logging (never exposed to locked UI).
    fn name(&self) -> &str;

    /// Send a serialised inventory JSON to the peer and receive theirs.
    fn exchange_inventory(&self, local_json: &str) -> CoreResult<String>;

    /// Fetch the raw bytes of one content object from the peer.
    fn fetch_object(&self, object_hash: &str) -> CoreResult<Vec<u8>>;

    /// Push a content object to the peer.
    fn push_object(&self, object_hash: &str, data: &[u8]) -> CoreResult<()>;
}
