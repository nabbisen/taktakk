//! Local filesystem transport adapter.
//!
//! This is a **real, working** transport that uses the local filesystem as
//! the "channel" between two simulated devices. It serves two purposes:
//!
//! 1. **Development / CI**: Run full sync integration tests without any
//!    Bluetooth or Wi-Fi hardware.
//! 2. **Prototype**: Demonstrates the full protocol pipeline end-to-end,
//!    making it straightforward to replace the backend with BLE/WiFi-Direct.
//!
//! The adapter uses a shared directory as the "wire": Device A writes its
//! inventory to `<dir>/a_inventory.json`; Device B reads it. Objects are
//! staged in `<dir>/objects/<hash>`.

use std::path::{Path, PathBuf};

use taktakk_core::error::{CoreError, CoreResult};

use super::SyncTransportAdapter;

/// Local filesystem sync transport.
pub struct LocalFsTransport {
    /// Shared staging directory used as the "channel".
    staging_dir: PathBuf,
    /// This device's side tag (`"a"` or `"b"`).
    side: &'static str,
    /// The peer's side tag.
    peer: &'static str,
}

impl LocalFsTransport {
    /// Create a new local transport for side `"a"` or `"b"`.
    pub fn new(staging_dir: PathBuf, side: &'static str) -> Self {
        let peer = if side == "a" { "b" } else { "a" };
        std::fs::create_dir_all(staging_dir.join("objects")).ok();
        Self { staging_dir, side, peer }
    }

    fn inventory_path(&self, who: &str) -> PathBuf {
        self.staging_dir.join(format!("{who}_inventory.json"))
    }

    fn object_path(&self, hash: &str) -> PathBuf {
        self.staging_dir.join("objects").join(hash)
    }
}

impl SyncTransportAdapter for LocalFsTransport {
    fn name(&self) -> &str { "local-fs" }

    fn exchange_inventory(&self, local_json: &str) -> CoreResult<String> {
        // Write ours.
        std::fs::write(self.inventory_path(self.side), local_json)
            .map_err(|e| CoreError::Sync(e.to_string()))?;

        // Read peer's (may not exist yet in sequential tests; caller retries).
        let peer_path = self.inventory_path(self.peer);
        if !peer_path.exists() {
            return Err(CoreError::Sync(
                "peer inventory not yet available".to_string(),
            ));
        }
        std::fs::read_to_string(&peer_path)
            .map_err(|e| CoreError::Sync(e.to_string()))
    }

    fn fetch_object(&self, object_hash: &str) -> CoreResult<Vec<u8>> {
        let path = self.object_path(object_hash);
        std::fs::read(&path)
            .map_err(|_| CoreError::Sync(format!("object not found: {object_hash}")))
    }

    fn push_object(&self, object_hash: &str, data: &[u8]) -> CoreResult<()> {
        std::fs::write(self.object_path(object_hash), data)
            .map_err(|e| CoreError::Sync(e.to_string()))
    }
}

/// Stage a set of objects into the shared directory so a peer can fetch them.
pub fn stage_objects(staging_dir: &Path, objects: &[(&str, &[u8])]) -> CoreResult<()> {
    let obj_dir = staging_dir.join("objects");
    std::fs::create_dir_all(&obj_dir)
        .map_err(|e| CoreError::Sync(e.to_string()))?;
    for (hash, data) in objects {
        let path = obj_dir.join(hash);
        std::fs::write(&path, data)
            .map_err(|e| CoreError::Sync(e.to_string()))?;
    }
    Ok(())
}
