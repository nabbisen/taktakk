//! Offline sync and import domain model.
//!
//! taktakk propagates content peer-to-peer, never through a central server.
//! A sync "session" is a local negotiation of what content to transfer.

use serde::{Deserialize, Serialize};

/// A record of a content-sharing session with another device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSession {
    pub session_id: String,
    /// Ephemeral identifier for the remote peer (not a permanent ID).
    pub peer_ephemeral_id: String,
    pub transport: TransportKind,
    pub started_at: i64,
    pub completed_at: Option<i64>,
    pub status: SyncStatus,
    pub objects_received: u32,
    pub objects_sent: u32,
}

/// How the sync connection was established.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportKind {
    Bluetooth,
    WifiDirect,
    LocalNetwork,
    QrBootstrap,
    SdCard,
    UsbOtg,
    LocalFile,
}

/// Current state of a sync session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncStatus {
    Negotiating,
    Transferring,
    Verifying,
    Completed,
    Failed,
    Aborted,
}

/// An inventory item: a package the local device has or wants.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncInventoryItem {
    pub package_id: String,
    pub version_string: String,
    pub manifest_hash: String,
    pub locally_available: bool,
}
