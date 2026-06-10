//! End-to-end integration tests for the taktakk platform.
//!
//! These tests exercise the complete pipeline across all crates:
//! facade-clock → unlock gesture → database open → package install
//! → lesson run → wipe → sync inventory exchange.
//!
//! Every test uses an ephemeral temp directory as its data store.

pub mod harness;

#[cfg(test)]
mod tests;