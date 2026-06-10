//! taktakk-module-engine: learning experience logic.
//!
//! This crate owns the domain model and state machine for the learning
//! experience: catalog view, lesson runner, exercise evaluation, and
//! crash-safe progress state serialization.
//!
//! It has no SQLite or Leptos dependencies; all I/O goes through port traits.

pub mod catalog;
pub mod error;
pub mod exercise;
pub mod runner;
pub mod state;
pub mod step;

pub use error::{EngineError, EngineResult};

#[cfg(test)]
mod tests;
