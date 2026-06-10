//! Package installation use case.

use crate::domain::package::{ContentPackage, PackageStatus};
use crate::error::CoreResult;
use crate::ports::storage::PackageRepository;

/// Install a verified package record into the local database.
pub fn record_installed_package(
    pkg_repo: &dyn PackageRepository,
    package: ContentPackage,
) -> CoreResult<()> {
    pkg_repo.save_package(&package)
}

/// Quarantine a package that failed verification.
pub fn quarantine_package(
    pkg_repo: &dyn PackageRepository,
    package_id: &str,
) -> CoreResult<()> {
    pkg_repo.update_status(package_id, PackageStatus::Quarantined)
}
