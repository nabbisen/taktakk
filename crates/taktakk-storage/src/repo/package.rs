//! Content package repository.

use sqlx::SqlitePool;

use taktakk_core::domain::curriculum::ModuleVersion;
use taktakk_core::domain::package::{ContentPackage, PackageStatus};
use taktakk_core::error::{CoreError, CoreResult};

pub async fn save(pool: &SqlitePool, p: &ContentPackage) -> CoreResult<()> {
    let status_str = package_status_str(&p.status);
    sqlx::query(
        "INSERT INTO content_packages
             (package_id, module_id, version_major, version_minor, version_patch,
              manifest_hash, status, installed_at, quarantine_reason)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(package_id) DO UPDATE SET
             status             = excluded.status,
             installed_at       = excluded.installed_at,
             quarantine_reason  = excluded.quarantine_reason",
    )
    .bind(&p.package_id)
    .bind(&p.module_id)
    .bind(p.version.major)
    .bind(p.version.minor)
    .bind(p.version.patch)
    .bind(&p.manifest_hash)
    .bind(status_str)
    .bind(p.installed_at)
    .bind(&p.quarantine_reason)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn get(pool: &SqlitePool, package_id: &str) -> CoreResult<Option<ContentPackage>> {
    type Row = (String, String, i64, i64, i64, String, String, Option<i64>, Option<String>);
    let row = sqlx::query_as::<_, Row>(
        "SELECT package_id, module_id, version_major, version_minor, version_patch,
                manifest_hash, status, installed_at, quarantine_reason
         FROM content_packages
         WHERE package_id = ?",
    )
    .bind(package_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(row_to_package))
}

pub async fn list(pool: &SqlitePool) -> CoreResult<Vec<ContentPackage>> {
    type Row = (String, String, i64, i64, i64, String, String, Option<i64>, Option<String>);
    let rows = sqlx::query_as::<_, Row>(
        "SELECT package_id, module_id, version_major, version_minor, version_patch,
                manifest_hash, status, installed_at, quarantine_reason
         FROM content_packages
         ORDER BY installed_at DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(rows.into_iter().map(row_to_package).collect())
}

pub async fn update_status(
    pool: &SqlitePool,
    package_id: &str,
    status: PackageStatus,
) -> CoreResult<()> {
    sqlx::query("UPDATE content_packages SET status = ? WHERE package_id = ?")
        .bind(package_status_str(&status))
        .bind(package_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

fn package_status_str(s: &PackageStatus) -> &'static str {
    match s {
        PackageStatus::Pending     => "pending",
        PackageStatus::Installed   => "installed",
        PackageStatus::Incomplete  => "incomplete",
        PackageStatus::Quarantined => "quarantined",
        PackageStatus::Disabled    => "disabled",
    }
}

fn parse_status(s: &str) -> PackageStatus {
    match s {
        "installed"   => PackageStatus::Installed,
        "incomplete"  => PackageStatus::Incomplete,
        "quarantined" => PackageStatus::Quarantined,
        "disabled"    => PackageStatus::Disabled,
        _             => PackageStatus::Pending,
    }
}

type Row = (String, String, i64, i64, i64, String, String, Option<i64>, Option<String>);

fn row_to_package(r: Row) -> ContentPackage {
    let (package_id, module_id, major, minor, patch, manifest_hash, status_str,
         installed_at, quarantine_reason) = r;
    ContentPackage {
        package_id,
        module_id,
        version: ModuleVersion::new(major as u16, minor as u16, patch as u16),
        manifest_hash,
        status: parse_status(&status_str),
        installed_at,
        quarantine_reason,
    }
}
