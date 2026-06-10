//! Port trait definitions.
//!
//! All I/O, storage, cryptography, and transport dependencies are expressed
//! as traits here. Concrete implementations live in platform-specific crates.

pub mod crypto;
pub mod module_runtime;
pub mod package_store;
pub mod storage;
pub mod sync;
