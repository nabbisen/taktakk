//! Package installation pipeline (RFC-039/040).
//!
//! `install_package()` is a thin `Cursor`-based wrapper for test fixtures.
//! `install_package_stream()` is the real implementation using the streaming
//! `NmpStreamReader`.

use std::io::Cursor;

use sha2::{Digest, Sha256};
use taktakk_core::domain::curriculum::ModuleVersion;
use taktakk_core::domain::package::{ContentPackage, PackageManifest, PackageStatus};
use taktakk_core::ports::package_store::ObjectStore;
use taktakk_security::trust_anchor::TrustAnchor;

use crate::nmp::stream_reader::install_package_stream;

/// Outcome of a package installation attempt.
#[derive(Debug)]
pub enum InstallOutcome {
    /// All checks passed; objects stored.
    Installed { package: ContentPackage },
    /// One or more checks failed; package is quarantined.
    Quarantined { reason: String },
}

/// Install a `.nmp` byte buffer.
///
/// Thin wrapper around `install_package_stream` for test fixtures and
/// backwards-compatible callers that already have the bytes in memory.
pub fn install_package(
    raw: &[u8],
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    object_store: &dyn ObjectStore,
    now: i64,
) -> InstallOutcome {
    install_package_stream(Cursor::new(raw), package_id, trust_anchors, object_store, now)
}

/// Build a `ContentPackage` record for a successful install.
///
/// Called by `install_package_stream` after all objects are verified and stored.
pub fn build_content_package(
    package_id: &str,
    manifest: &PackageManifest,
    manifest_bytes: &[u8],
    now: i64,
) -> ContentPackage {
    let manifest_hash = hex::encode(Sha256::digest(manifest_bytes));
    ContentPackage {
        package_id: package_id.to_string(),
        module_id: manifest.module_id.clone(),
        version: manifest.version.clone(),
        manifest_hash,
        status: PackageStatus::Installed,
        installed_at: Some(now),
        quarantine_reason: None,
    }
}

/// Build a quarantined `ContentPackage` record for a failed install.
pub fn quarantine_record(
    package_id: &str,
    module_id: &str,
    reason: &str,
) -> ContentPackage {
    ContentPackage {
        package_id: package_id.to_string(),
        module_id: module_id.to_string(),
        version: ModuleVersion::new(0, 0, 0),
        manifest_hash: String::new(),
        status: PackageStatus::Quarantined,
        installed_at: None,
        quarantine_reason: Some(reason.to_string()),
    }
}
