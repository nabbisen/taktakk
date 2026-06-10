//! Start sync use case.

use crate::domain::sync::SyncInventoryItem;

/// Build a local inventory from installed packages.
pub fn build_local_inventory(package_ids: Vec<(String, String, String)>) -> Vec<SyncInventoryItem> {
    package_ids
        .into_iter()
        .map(|(package_id, version_string, manifest_hash)| SyncInventoryItem {
            package_id,
            version_string,
            manifest_hash,
            locally_available: true,
        })
        .collect()
}

/// Compute which objects in the remote inventory we are missing locally.
pub fn plan_download(
    remote: &[SyncInventoryItem],
    local: &[SyncInventoryItem],
) -> Vec<SyncInventoryItem> {
    let local_ids: std::collections::HashSet<&str> =
        local.iter().map(|i| i.package_id.as_str()).collect();
    remote
        .iter()
        .filter(|item| !local_ids.contains(item.package_id.as_str()))
        .cloned()
        .collect()
}
