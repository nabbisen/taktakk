//! Content package domain model.
//!
//! A `.nmp` (taktakk Module Package) is the unit of offline distribution.
//! Every package is cryptographically signed and content-addressed.

use serde::{Deserialize, Serialize};

use super::curriculum::ModuleVersion;

/// Magic bytes that identify a valid `.nmp` file: "TAKT".
pub const NMP_MAGIC: [u8; 4] = [0x54, 0x41, 0x4B, 0x54];

/// Current format version for `.nmp` packages.
pub const NMP_FORMAT_VERSION: u8 = 1;

/// Parsed manifest extracted from a `.nmp` package header.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Unique package identifier (e.g., `shield-water-purification`).
    pub module_id: String,
    /// Package version.
    pub version: ModuleVersion,
    /// Minimum taktakk runtime version required to execute this package.
    pub min_core_version: ModuleVersion,
    /// Signer identifier; used to look up the trust anchor.
    pub signer_id: String,
    /// Map of relative object paths to their SHA-256 hashes (hex-encoded).
    pub objects: Vec<ObjectEntry>,
    /// Locale tags this package provides (BCP 47).
    pub locales: Vec<String>,
}

/// A single content object referenced in a manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectEntry {
    /// Relative path within the package (e.g., `lesson-01.svg`).
    pub path: String,
    /// SHA-256 hash of the raw (uncompressed) object, hex-encoded.
    pub sha256: String,
    /// MIME-like type hint.
    pub object_type: ObjectType,
    /// Whether this object is required for basic functionality.
    pub required: bool,
}

/// Kind of content object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    Manifest,
    Wasm,
    Svg,
    Audio,
    Json,
    Patch,
}

/// A package record stored in the local database after installation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPackage {
    pub package_id: String,
    pub module_id: String,
    pub version: ModuleVersion,
    pub manifest_hash: String,
    pub status: PackageStatus,
    pub installed_at: Option<i64>,
    pub quarantine_reason: Option<String>,
}

/// Current installation state of a package.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageStatus {
    /// Pending verification or installation.
    Pending,
    /// Signature and hash checks passed; content is installed.
    Installed,
    /// At least one required object is missing or corrupt.
    Incomplete,
    /// Signature or hash check failed; content is isolated.
    Quarantined,
    /// Disabled by the user or operator.
    Disabled,
}

/// Verifies that the magic bytes of raw package data are valid.
pub fn check_magic(data: &[u8]) -> bool {
    data.len() >= 4 && data[..4] == NMP_MAGIC
}
