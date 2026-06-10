//! taktakk-core: domain types, use cases, and port trait definitions.
//!
//! This crate contains no platform-specific code, no SQL, no Leptos components,
//! and no direct cryptographic primitives. All external dependencies are
//! expressed as trait ports in [`ports`].

pub mod domain;
pub mod error;
pub mod ports;
pub mod use_cases;

#[cfg(test)]
mod tests;
