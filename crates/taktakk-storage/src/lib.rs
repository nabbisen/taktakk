//! taktakk-storage: SQLite repositories, migrations, and object-store.
//!
//! Storage layers:
//! - `facade.sqlite` — clock settings, unlock slot hashes, key slots
//! - `core.sqlite`  — encrypted curriculum, progress, and event state
//! - `object_store/` — content-addressed binary objects

pub mod db;
pub mod error;
pub mod event_log;
pub mod object_store;
pub mod repo;

pub use db::Database;
pub use error::{StorageError, StorageResult};

#[cfg(test)]
mod tests;
