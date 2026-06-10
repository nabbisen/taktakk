//! SQLite-backed repository implementations.
//!
//! Each repository implements the corresponding trait from
//! `taktakk_core::ports::storage`. All queries are async and
//! run inside the provided `SqlitePool`.

pub mod facade;
pub mod package;
pub mod profile;
pub mod progress;
