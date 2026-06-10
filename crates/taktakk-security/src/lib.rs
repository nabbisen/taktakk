//! taktakk-security: cryptography, key slots, trust anchors, and panic wipe.
//!
//! This crate implements the [`taktakk_core::ports::crypto`] traits and
//! provides the concrete cryptographic primitives used by the application.
//!
//! Raw secret key material is never exposed through public APIs.

pub mod hash;
pub mod key_slot;
pub mod trust_anchor;
pub mod verifier;
pub mod wipe;

#[cfg(test)]
mod tests;
