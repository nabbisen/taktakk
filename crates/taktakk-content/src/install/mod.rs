//! Package installation pipeline.
//!
//! The pipeline is:
//! 1. Parse the `.nmp` bytes.
//! 2. Verify the Ed25519 signature.
//! 3. Verify each object's SHA-256 hash.
//! 4. Write objects to the object store.
//! 5. Record the package in the database (status = Installed).
//!
//! Any failure before step 4 causes an immediate quarantine record
//! without touching the object store.

use taktakk_core::domain::curriculum::ModuleVersion;
use taktakk_core::domain::package::{ContentPackage, PackageStatus};
use taktakk_core::ports::package_store::ObjectStore;
use taktakk_security::trust_anchor::TrustAnchor;

use crate::nmp::error::{ContentError, ContentResult};
use crate::nmp::reader::parse;
use crate::verify;

/// Outcome of a package installation attempt.
#[derive(Debug)]
pub enum InstallOutcome {
    /// All checks passed; objects stored.
    Installed { package: ContentPackage },
    /// One or more checks failed; package is quarantined.
    Quarantined { reason: String },
}

/// Install a `.nmp` byte buffer using the supplied object store and trust anchors.
///
/// `package_id` should be a pre-generated UUID for the DB record.
/// `now` is the current Unix timestamp in seconds.
pub fn install_package(
    raw: &[u8],
    package_id: &str,
    trust_anchors: &[TrustAnchor],
    object_store: &dyn ObjectStore,
    now: i64,
) -> InstallOutcome {
    // Parse
    let pkg = match parse(raw) {
        Ok(p) => p,
        Err(e) => return InstallOutcome::Quarantined { reason: e.to_string() },
    };

    // Verify signature
    if let Err(e) = verify::verify_signature(&pkg, trust_anchors) {
        return InstallOutcome::Quarantined { reason: e.to_string() };
    }

    // Verify object hashes
    if let Err(e) = verify::verify_objects(&pkg) {
        return InstallOutcome::Quarantined { reason: e.to_string() };
    }

    // Write objects to store
    for (entry, data) in pkg.manifest.objects.iter().zip(pkg.objects.iter()) {
        // The store's `put` re-computes the hash and stores under that key.
        // We verify the returned hash matches the manifest entry.
        let stored_hash = match object_store.put(data) {
            Ok(h) => h,
            Err(e) => {
                return InstallOutcome::Quarantined {
                    reason: format!("object store write failed for '{}': {}", entry.path, e),
                }
            }
        };
        if stored_hash != entry.sha256 {
            // Shouldn't happen after verify_objects, but be defensive.
            let _ = object_store.quarantine(&stored_hash, "post-write hash mismatch");
            return InstallOutcome::Quarantined {
                reason: format!(
                    "post-write hash mismatch for '{}': {} != {}",
                    entry.path, entry.sha256, stored_hash
                ),
            };
        }
    }

    let version = pkg.manifest.version.clone();
    let module_id = pkg.manifest.module_id.clone();
    let manifest_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(&pkg.manifest_bytes);
        hex::encode(h.finalize())
    };

    InstallOutcome::Installed {
        package: ContentPackage {
            package_id: package_id.to_string(),
            module_id,
            version,
            manifest_hash,
            status: PackageStatus::Installed,
            installed_at: Some(now),
            quarantine_reason: None,
        },
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
