//! Manifest exchange: compare two inventories and build a transfer plan.
//!
//! The manifest exchange is the first step of every sync session.
//! No object data is transferred until the plan is agreed upon.

use serde::{Deserialize, Serialize};
use taktakk_core::domain::sync::SyncInventoryItem;

use crate::inventory::LocalInventory;

/// The action to take for one package during a sync.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncAction {
    /// Send this package to the peer.
    Send,
    /// Receive this package from the peer.
    Receive,
    /// Both sides already have the same version; no transfer needed.
    Skip,
    /// Hash mismatch — verify integrity before proceeding.
    VerifyOnly,
}

/// One item in the negotiated sync plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestItem {
    pub package_id: String,
    pub action: SyncAction,
    /// Manifest hash of the local copy (if present).
    pub local_manifest_hash: Option<String>,
    /// Manifest hash of the remote copy (if present).
    pub remote_manifest_hash: Option<String>,
}

/// Build a transfer plan from two inventories.
///
/// Returns the list of actions that need to happen for the local device to
/// come into parity with the remote.
pub fn build_transfer_plan(
    local: &LocalInventory,
    remote: &LocalInventory,
) -> Vec<ManifestItem> {
    use std::collections::HashMap;

    let local_map: HashMap<&str, &SyncInventoryItem> =
        local.items.iter().map(|i| (i.package_id.as_str(), i)).collect();
    let remote_map: HashMap<&str, &SyncInventoryItem> =
        remote.items.iter().map(|i| (i.package_id.as_str(), i)).collect();

    let all_ids: std::collections::HashSet<&str> = local_map
        .keys()
        .chain(remote_map.keys())
        .copied()
        .collect();

    let mut plan: Vec<ManifestItem> = all_ids
        .into_iter()
        .map(|id| {
            let l = local_map.get(id);
            let r = remote_map.get(id);
            let action = match (l, r) {
                (Some(li), Some(ri)) => {
                    if li.manifest_hash == ri.manifest_hash {
                        SyncAction::Skip
                    } else {
                        SyncAction::VerifyOnly
                    }
                }
                (Some(_), None) => SyncAction::Send,
                (None, Some(_)) => SyncAction::Receive,
                (None, None) => SyncAction::Skip, // shouldn't happen
            };
            ManifestItem {
                package_id: id.to_string(),
                action,
                local_manifest_hash: l.map(|i| i.manifest_hash.clone()),
                remote_manifest_hash: r.map(|i| i.manifest_hash.clone()),
            }
        })
        .collect();

    // Stable sort: Receive first, then Send, then VerifyOnly, then Skip.
    plan.sort_by_key(|item| match item.action {
        SyncAction::Receive    => 0,
        SyncAction::Send       => 1,
        SyncAction::VerifyOnly => 2,
        SyncAction::Skip       => 3,
    });

    plan
}

/// Serialise a `LocalInventory` to a compact JSON string for wire transmission
/// or QR-code bootstrap.
pub fn serialise_inventory(inv: &LocalInventory) -> Result<String, serde_json::Error> {
    serde_json::to_string(inv)
}

/// Deserialise a `LocalInventory` received from a peer.
pub fn deserialise_inventory(json: &str) -> Result<LocalInventory, serde_json::Error> {
    serde_json::from_str(json)
}
