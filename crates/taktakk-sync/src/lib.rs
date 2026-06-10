//! taktakk-sync: offline manifest exchange, delta planning, and transport adapters.
//!
//! All transport implementations must work without internet access.
//! The architecture separates the sync protocol (inventory diffing, chunk
//! verification) from transport mechanics (Bluetooth, local FS, SD card).

pub mod chunk;
pub mod import;
pub mod inventory;
pub mod manifest;
pub mod permission;
pub mod transport;

pub use transport::SyncTransportAdapter;

#[cfg(test)]
mod tests;
