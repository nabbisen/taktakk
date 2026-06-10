//! Physical media import: scan directories for `.nmp` files.
//!
//! This module handles "Sneakernet" distribution — importing packages
//! from SD cards, USB drives, or the device's Downloads folder.
//!
//! **Security rule:** Files are never auto-parsed on discovery.
//! The user must explicitly confirm each import to avoid exploits from
//! maliciously crafted `.nmp` files placed on shared media.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// A `.nmp` file found during a media scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoundPackage {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// File size in bytes.
    pub byte_size: u64,
    /// Whether the file's magic bytes look like a valid `.nmp`.
    pub looks_valid: bool,
}

/// Result of a media scan.
#[derive(Debug, Default)]
pub struct ScanResult {
    pub found: Vec<FoundPackage>,
    pub scan_error_count: usize,
}

/// Recursively scan `root` for files with the `.nmp` extension.
///
/// Files are **not** parsed or verified here; that happens in the install
/// pipeline after the user confirms the selection.
///
/// `max_depth` limits recursion to avoid infinite symlink loops.
pub fn scan_directory(root: &Path, max_depth: u32) -> ScanResult {
    let mut result = ScanResult::default();
    scan_recursive(root, 0, max_depth, &mut result);
    result
}

fn scan_recursive(dir: &Path, depth: u32, max_depth: u32, result: &mut ScanResult) {
    if depth > max_depth {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => {
            result.scan_error_count += 1;
            return;
        }
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_recursive(&path, depth + 1, max_depth, result);
        } else if path.extension().and_then(|e| e.to_str()) == Some("nmp") {
            let byte_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            let looks_valid = check_magic(&path);
            result.found.push(FoundPackage { path, byte_size, looks_valid });
        }
    }
}

/// Peek at the first 4 bytes of a file to check magic bytes.
fn check_magic(path: &Path) -> bool {
    use taktakk_core::domain::package::NMP_MAGIC;
    let mut buf = [0u8; 4];
    std::fs::File::open(path)
        .ok()
        .and_then(|mut f| {
            use std::io::Read;
            f.read_exact(&mut buf).ok()
        })
        .is_some()
        && buf == NMP_MAGIC
}

/// Read a confirmed `.nmp` file from disk, buffered to avoid OOM on
/// devices with < 1 GiB RAM.
///
/// `max_bytes` is a safety cap; returns `Err` if the file exceeds it.
pub fn read_package_file(path: &Path, max_bytes: u64) -> std::io::Result<Vec<u8>> {
    let meta = std::fs::metadata(path)?;
    if meta.len() > max_bytes {
        return Err(std::io::Error::new(
            std::io::ErrorKind::FileTooLarge,
            format!("file exceeds limit: {} > {}", meta.len(), max_bytes),
        ));
    }
    std::fs::read(path)
}

/// Source label hash: store only a hash of the file path, never the raw path.
///
/// Protects the identity of the import source (which organisation's SD card
/// was used, which folder it came from).
pub fn source_label_hash(path: &Path) -> String {
    use sha2::{Digest, Sha256};
    let s = path.to_string_lossy();
    hex::encode(Sha256::digest(s.as_bytes()))
}

/// Status of an import job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImportStatus {
    Scanning,
    Verified,
    Installed,
    Failed,
}

/// Per-item result for one `.nmp` within an import job.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemVerifyResult {
    Valid,
    Invalid,
    Duplicate,
}

/// Per-item install outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemInstallResult {
    Installed,
    Skipped,
    Failed,
}
