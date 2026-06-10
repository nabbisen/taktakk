//! Sync inventory: local catalogue of installed packages for peer comparison.
//!
//! During a sync session, both devices exchange their inventory so that
//! each can compute what the other is missing. No package content is
//! exchanged at this stage — only hashes and version strings.

use serde::{Deserialize, Serialize};
use taktakk_core::domain::sync::SyncInventoryItem;

/// A local inventory snapshot ready to share with a peer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalInventory {
    /// Packages available on this device.
    pub items: Vec<SyncInventoryItem>,
    /// Compact JSON snapshot (for wire transmission or QR bootstrap).
    pub snapshot_hash: String,
}

impl LocalInventory {
    /// Build from a list of installed (package_id, version, manifest_hash) triples.
    pub fn build(packages: Vec<(String, String, String)>) -> Self {
        let items: Vec<SyncInventoryItem> = packages
            .into_iter()
            .map(|(package_id, version_string, manifest_hash)| SyncInventoryItem {
                package_id,
                version_string,
                manifest_hash,
                locally_available: true,
            })
            .collect();

        let snapshot_hash = compute_inventory_hash(&items);
        Self { items, snapshot_hash }
    }

    /// Compute which items in `remote` are absent from this inventory.
    pub fn missing_from_remote(&self, remote: &LocalInventory) -> Vec<SyncInventoryItem> {
        let local_ids: std::collections::HashSet<&str> =
            self.items.iter().map(|i| i.package_id.as_str()).collect();

        remote
            .items
            .iter()
            .filter(|item| !local_ids.contains(item.package_id.as_str()))
            .cloned()
            .collect()
    }

    /// Items this device has that the remote is missing.
    pub fn missing_from_local(&self, remote: &LocalInventory) -> Vec<SyncInventoryItem> {
        remote.missing_from_remote(self)
    }

    /// Returns `true` if both inventories contain the same set of package IDs.
    pub fn is_in_sync_with(&self, remote: &LocalInventory) -> bool {
        self.snapshot_hash == remote.snapshot_hash
    }
}

/// Compute a deterministic SHA-256 hash of the sorted inventory for quick
/// equality checks without full comparison.
fn compute_inventory_hash(items: &[SyncInventoryItem]) -> String {
    use sha2::{Digest, Sha256};
    let mut sorted: Vec<&str> = items.iter().map(|i| i.manifest_hash.as_str()).collect();
    sorted.sort_unstable();
    let joined = sorted.join("|");
    hex::encode(Sha256::digest(joined.as_bytes()))
}
