//! taktakk-content: signed content package format and validation.
//!
//! # Package lifecycle
//!
//! 1. **Parse** — read magic bytes, manifest length, manifest JSON, signature
//! 2. **Verify** — check Ed25519 signature against a known trust anchor
//! 3. **Extract** — for each object entry, read bytes and verify SHA-256 hash
//! 4. **Install** — store objects in the object store, record package in DB
//! 5. **Quarantine** — on any verification failure, isolate all extracted data

pub mod fixtures;
pub mod install;
pub mod nmp;
pub mod samples;
pub mod verify;

pub use nmp::error::{ContentError, ContentResult};

#[cfg(test)]
mod tests;
