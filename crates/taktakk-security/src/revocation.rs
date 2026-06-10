//! Trust revocation model (RFC 034).
//!
//! A signed revocation package can disable compromised signing keys and
//! known-bad content packages, propagating through the same offline channels
//! as normal content (SD card, Bluetooth, local sync).

use serde::{Deserialize, Serialize};

use taktakk_core::domain::curriculum::ModuleVersion;

/// A signed revocation package (received as a special `.nmp` with
/// `package_type = "trust-revocation"`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevocationPackage {
    /// The signer must be a master-level trust anchor.
    pub signer_id: String,
    /// When this revocation was issued.
    pub issued_at: i64,
    /// Severity of the revocation event.
    pub severity: RevocationSeverity,
    /// Keys being revoked.
    pub revoked_keys: Vec<RevokedKey>,
    /// Package hashes being quarantined.
    pub revoked_package_hashes: Vec<String>,
    /// User-facing localizable message key (calm, non-revealing).
    pub user_message_key: String,
    /// Optional pointer to a replacement package.
    pub replacement_hint: Option<ReplacementHint>,
}

/// Severity level of the revocation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RevocationSeverity {
    /// Routine key rotation; no immediate user action needed.
    Informational,
    /// Known content error; users should update when possible.
    Advisory,
    /// Safety-critical issue; affected modules should be disabled.
    Critical,
}

/// A signing key being revoked.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedKey {
    pub signing_key_id: String,
    /// When the key is considered compromised from.
    pub compromised_from: i64,
}

/// Pointer to a package that replaces revoked content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplacementHint {
    pub module_id: String,
    pub min_version: ModuleVersion,
    /// SHA-256 of the replacement manifest, so it can be pre-verified.
    pub manifest_hash_hint: Option<String>,
}

/// Apply a revocation package to local trust anchors and package records.
///
/// This is a pure decision function — it returns the list of changes to
/// make; the caller applies them to the database.
pub fn plan_revocation(rev: &RevocationPackage) -> RevocationPlan {
    RevocationPlan {
        keys_to_revoke: rev.revoked_keys.iter()
            .map(|k| k.signing_key_id.clone())
            .collect(),
        packages_to_quarantine: rev.revoked_package_hashes.clone(),
        severity: rev.severity.clone(),
        user_message_key: rev.user_message_key.clone(),
    }
}

/// The set of changes to apply when processing a revocation package.
#[derive(Debug, Clone)]
pub struct RevocationPlan {
    pub keys_to_revoke: Vec<String>,
    pub packages_to_quarantine: Vec<String>,
    pub severity: RevocationSeverity,
    pub user_message_key: String,
}

impl RevocationPlan {
    /// `true` if this plan has any actual changes to apply.
    pub fn has_changes(&self) -> bool {
        !self.keys_to_revoke.is_empty() || !self.packages_to_quarantine.is_empty()
    }

    /// `true` if this is a critical revocation requiring immediate user notification.
    pub fn is_critical(&self) -> bool {
        self.severity == RevocationSeverity::Critical
    }
}
