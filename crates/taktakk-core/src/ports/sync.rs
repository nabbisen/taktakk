//! Sync transport port.

use crate::domain::sync::{SyncInventoryItem, TransportKind};
use crate::error::CoreResult;

/// A discovered remote peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Ephemeral ID (not stored permanently).
    pub ephemeral_id: String,
    pub transport: TransportKind,
}

/// Abstract transport adapter for offline content sharing.
pub trait SyncTransport: Send + Sync {
    fn transport_kind(&self) -> TransportKind;

    /// Scan for reachable peers.
    fn discover_peers(&self) -> CoreResult<Vec<PeerInfo>>;

    /// Request the inventory from a peer.
    fn request_inventory(&self, peer: &PeerInfo) -> CoreResult<Vec<SyncInventoryItem>>;

    /// Send a single content object to a peer.
    fn send_object(&self, peer: &PeerInfo, sha256_hex: &str, data: &[u8]) -> CoreResult<()>;

    /// Receive a single content object from a peer.
    fn receive_object(&self, peer: &PeerInfo, sha256_hex: &str) -> CoreResult<Vec<u8>>;
}
