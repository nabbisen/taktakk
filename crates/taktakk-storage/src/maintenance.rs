//! Storage maintenance: garbage collection, quarantine expiry, staging cleanup.
//!
//! Maintenance tasks are designed to be:
//! - **Cancellable**: each step is independently atomic.
//! - **Battery-friendly**: prefer short work units with explicit yield points.
//! - **Safe**: never delete objects that are still referenced by installed packages.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use sqlx::SqlitePool;

use crate::error::{StorageError, StorageResult};

// ── Object store GC ───────────────────────────────────────────────────────────

/// Collect the SHA-256 hashes of all objects referenced by installed packages.
pub async fn referenced_object_hashes(core: &SqlitePool) -> StorageResult<HashSet<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT object_hash FROM package_objects
         JOIN content_packages USING (package_id)
         WHERE content_packages.status IN ('installed', 'pending', 'incomplete')",
    )
    .fetch_all(core)
    .await
    .map_err(StorageError::Database)?;

    Ok(rows.into_iter().map(|(h,)| h).collect())
}

/// Delete orphaned objects from the filesystem object store.
///
/// An orphan is an object on disk whose hash does not appear in `referenced`.
/// Returns the number of files deleted.
pub fn gc_object_store(store_dir: &Path, referenced: &HashSet<String>) -> usize {
    let mut deleted = 0;
    let Ok(prefixes) = std::fs::read_dir(store_dir) else { return 0 };

    for prefix_entry in prefixes.flatten() {
        let prefix_path = prefix_entry.path();
        if !prefix_path.is_dir() { continue; }
        // Skip special directories.
        if prefix_path.file_name().and_then(|n| n.to_str()) == Some("quarantine") {
            continue;
        }
        let Ok(objects) = std::fs::read_dir(&prefix_path) else { continue };
        for obj_entry in objects.flatten() {
            let obj_path = obj_entry.path();
            if let Some(rest) = obj_path.file_name().and_then(|n| n.to_str()) {
                // Reconstruct full hash = prefix (2 chars) + rest.
                if let Some(prefix) = prefix_path.file_name().and_then(|n| n.to_str()) {
                    let full_hash = format!("{prefix}{rest}");
                    if !referenced.contains(&full_hash) {
                        if std::fs::remove_file(&obj_path).is_ok() {
                            deleted += 1;
                        }
                    }
                }
            }
        }
    }
    deleted
}

// ── Quarantine expiry ─────────────────────────────────────────────────────────

/// Maximum age for quarantined objects before they are eligible for deletion
/// (30 days in seconds).
pub const QUARANTINE_EXPIRY_SECONDS: i64 = 30 * 24 * 3600;

/// Delete quarantine directory objects older than `max_age_seconds`.
///
/// Quarantined objects are kept for the retention window to allow operators
/// to investigate verification failures before they are purged.
pub fn expire_quarantine(quarantine_dir: &Path, now: i64, max_age_seconds: i64) -> usize {
    let cutoff = std::time::UNIX_EPOCH
        .checked_add(std::time::Duration::from_secs(
            (now - max_age_seconds).max(0) as u64,
        ))
        .unwrap_or(std::time::UNIX_EPOCH);

    let mut deleted = 0;
    let Ok(entries) = std::fs::read_dir(quarantine_dir) else { return 0 };

    for entry in entries.flatten() {
        let path = entry.path();
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if modified < cutoff && std::fs::remove_file(&path).is_ok() {
                    deleted += 1;
                }
            }
        }
    }
    deleted
}

// ── Integrity scan ────────────────────────────────────────────────────────────

/// Re-verify a random sample of installed objects against their stored hashes.
///
/// Returns `(verified, corrupt)` counts. Corrupt objects are moved to quarantine.
pub fn spot_check_objects(
    store_dir: &Path,
    hashes: &[String],
    sample_size: usize,
) -> (usize, usize) {
    use sha2::{Digest, Sha256};
    let sample: Vec<&String> = if hashes.len() <= sample_size {
        hashes.iter().collect()
    } else {
        // Non-random deterministic sample for reproducibility in tests.
        hashes.iter().step_by(hashes.len() / sample_size + 1).collect()
    };

    let mut verified = 0;
    let mut corrupt  = 0;

    for hash in sample {
        let prefix = &hash[..2];
        let rest   = &hash[2..];
        let path   = store_dir.join(prefix).join(rest);

        if let Ok(data) = std::fs::read(&path) {
            let actual = hex::encode(Sha256::digest(&data));
            if actual == *hash {
                verified += 1;
            } else {
                corrupt += 1;
                // Move to quarantine.
                let q_dir = store_dir.join("quarantine");
                let _ = std::fs::create_dir_all(&q_dir);
                let _ = std::fs::rename(&path, q_dir.join(hash));
            }
        }
    }
    (verified, corrupt)
}

// ── Maintenance report ────────────────────────────────────────────────────────

/// Summary of a maintenance run.
#[derive(Debug, Clone, Default)]
pub struct MaintenanceReport {
    pub objects_gc_deleted: usize,
    pub quarantine_expired: usize,
    pub partial_files_cleaned: usize,
    pub objects_spot_checked: usize,
    pub objects_corrupt_found: usize,
    pub log_rows_purged: u64,
}

impl MaintenanceReport {
    pub fn is_clean(&self) -> bool {
        self.objects_corrupt_found == 0
    }
}
