//! Release candidate model (RFC 029/030).
//!
//! Defines the release manifest, seed kit profiles, and the build metadata
//! required for reproducible, verifiable offline distribution.

use serde::{Deserialize, Serialize};

// ── Release manifest ──────────────────────────────────────────────────────────

/// Machine-readable release manifest written during `xtask release-candidate`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseManifest {
    /// Semantic version string (e.g. `"0.8.0"`).
    pub version: String,
    /// Git commit SHA (40 hex chars).
    pub git_commit: String,
    /// Rust toolchain version (e.g. `"1.91.1"`).
    pub rust_toolchain: String,
    /// Build profile (`"release"` or `"debug"`).
    pub build_profile: String,
    /// List of artifact checksums.
    pub artifacts: Vec<ArtifactRecord>,
    /// ISO 8601 build timestamp.
    pub built_at: String,
    /// Feature flags active at build time.
    pub features: Vec<String>,
}

impl ReleaseManifest {
    /// Validate that all required fields are present and non-empty.
    pub fn validate(&self) -> Result<(), String> {
        if self.version.is_empty() {
            return Err("version is empty".to_string());
        }
        if self.git_commit.len() != 40 {
            return Err(format!(
                "git_commit must be 40 hex chars, got {}",
                self.git_commit.len()
            ));
        }
        if !self.git_commit.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err("git_commit contains non-hex characters".to_string());
        }
        if self.rust_toolchain.is_empty() {
            return Err("rust_toolchain is empty".to_string());
        }
        Ok(())
    }

    /// Check that all artifact hashes have the correct length (SHA-256 = 64 hex).
    pub fn validate_checksums(&self) -> Result<(), String> {
        for art in &self.artifacts {
            if art.sha256.len() != 64 {
                return Err(format!(
                    "artifact '{}' has invalid sha256 length {}",
                    art.name,
                    art.sha256.len()
                ));
            }
        }
        Ok(())
    }
}

/// One artifact in the release manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRecord {
    /// Descriptive name (e.g. `"taktakk-linux-x86_64"`).
    pub name: String,
    /// SHA-256 of the artifact file, hex-encoded.
    pub sha256: String,
    /// File size in bytes.
    pub byte_size: u64,
    /// Artifact kind.
    pub kind: ArtifactKind,
}

/// Classification of a release artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    AndroidApk,
    PwaBundle,
    LinuxBinary,
    ContentPackage,
    LocalePack,
    SeedKit,
    TrustAnchorUpdate,
}

// ── Seed kit profiles (RFC 030) ───────────────────────────────────────────────

/// Seed kit assembly profile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeedKitProfile {
    /// App + one locale + emergency Shield content only. (~5 MB target)
    Minimal,
    /// App + core Shield/Spear starter modules + common locales. (~25 MB)
    Standard,
    /// All approved starter modules + all locale packs. (~50 MB)
    Full,
}

impl SeedKitProfile {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Minimal  => "minimal",
            Self::Standard => "standard",
            Self::Full     => "full",
        }
    }

    /// Target size in bytes (guidance; actual size depends on content).
    pub fn target_bytes(&self) -> u64 {
        match self {
            Self::Minimal  =>  5 * 1024 * 1024,
            Self::Standard => 25 * 1024 * 1024,
            Self::Full     => 50 * 1024 * 1024,
        }
    }
}

/// A seed kit manifest: the list of packages in an assembled kit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedKitManifest {
    pub profile: SeedKitProfile,
    pub version: String,
    /// Package IDs included in this kit.
    pub package_ids: Vec<String>,
    /// Total uncompressed size estimate in bytes.
    pub estimated_bytes: u64,
    /// SHA-256 of this manifest JSON, for verification.
    pub manifest_hash: String,
}

impl SeedKitManifest {
    /// Verify that the manifest is internally consistent.
    pub fn validate(&self) -> Result<(), String> {
        if self.package_ids.is_empty() {
            return Err("seed kit contains no packages".to_string());
        }
        if self.version.is_empty() {
            return Err("version is empty".to_string());
        }
        // Minimal kit must have at least one Shield package.
        // Full validation requires checking package metadata; this is a
        // structural check only.
        Ok(())
    }

    /// Check that estimated size is within the profile target.
    pub fn check_size_budget(&self) -> Result<(), String> {
        let budget = self.profile.target_bytes();
        if self.estimated_bytes > budget {
            Err(format!(
                "{} kit exceeds size budget: {} > {} bytes",
                self.profile.label(),
                self.estimated_bytes,
                budget
            ))
        } else {
            Ok(())
        }
    }
}
