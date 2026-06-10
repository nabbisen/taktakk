//! Safety settings domain model.
//!
//! These settings are stored in `facade.sqlite` under innocuous key names.
//! They control wipe behavior, duress configuration, and log policy.
//!
//! **Security rule:** This module must never surface the word "wipe",
//! "panic", "erase", or any suspicious term in user-visible string keys.
//! The UI uses pictogram-only representations for the safety functions.

use serde::{Deserialize, Serialize};

/// What action a duress trigger should perform.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DuressAction {
    /// Overwrite crypto keys only (instant; data remains but is unreadable).
    CryptoErase,
    /// Crypto erase followed by best-effort deletion of core database.
    FullErase,
    /// Transition to demo mode with no real content (plausible deniability).
    DemoMode,
}

impl Default for DuressAction {
    fn default() -> Self {
        Self::CryptoErase
    }
}

/// Voluntary state wipe scope (user-controlled "reset progress").
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateWipeScope {
    /// Remove all learning progress; keep profiles and installed packages.
    ProgressOnly,
    /// Remove progress and profiles; keep installed packages.
    ProfilesAndProgress,
    /// Remove everything except the facade clock settings.
    AllUserData,
}

/// Log retention policy settings.
///
/// All log values here are upper bounds; the implementation enforces
/// stricter limits where required by threat model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRetentionPolicy {
    /// Maximum age of event log entries in seconds (default: 86 400 = 24 h).
    pub max_age_seconds: i64,
    /// Whether to purge logs on every session start.
    pub purge_on_start: bool,
    /// Whether event_tag values are further anonymised to coarse buckets.
    pub anonymise_tags: bool,
}

impl Default for LogRetentionPolicy {
    fn default() -> Self {
        Self {
            max_age_seconds: 86_400, // 24 hours
            purge_on_start: true,
            anonymise_tags: true,
        }
    }
}

impl LogRetentionPolicy {
    /// Validate that the policy values are within safe bounds.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_age_seconds < 3600 {
            return Err("max_age_seconds must be at least 3600 (1 hour)".to_string());
        }
        if self.max_age_seconds > 604_800 {
            return Err("max_age_seconds must not exceed 604 800 (7 days)".to_string());
        }
        Ok(())
    }
}

/// Complete safety settings record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySettings {
    pub duress_action: DuressAction,
    pub log_policy: LogRetentionPolicy,
    /// Whether the "demo mode" fallback is configured (requires a separate
    /// demo profile to be seeded).
    pub demo_mode_available: bool,
}

impl Default for SafetySettings {
    fn default() -> Self {
        Self {
            duress_action: DuressAction::default(),
            log_policy: LogRetentionPolicy::default(),
            demo_mode_available: false,
        }
    }
}

/// Coarse event-tag bucket: the only information persisted to event_log.
///
/// All event_tag strings in the log must be drawn from this set.
/// Any other string is a violation of the zero-telemetry policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventBucket {
    SessionOpen,
    SessionClose,
    InstallOk,
    InstallFail,
    WipeOk,
    SyncOk,
    SyncFail,
    ImportOk,
    ImportFail,
    IntegrityFail,
}

impl EventBucket {
    /// The canonical tag string stored in the log.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::SessionOpen    => "s.open",
            Self::SessionClose   => "s.close",
            Self::InstallOk      => "pkg.ok",
            Self::InstallFail    => "pkg.fail",
            Self::WipeOk         => "wipe.ok",
            Self::SyncOk         => "sync.ok",
            Self::SyncFail       => "sync.fail",
            Self::ImportOk       => "imp.ok",
            Self::ImportFail     => "imp.fail",
            Self::IntegrityFail  => "integ.fail",
        }
    }

    /// Check that a raw tag string is an approved bucket.
    pub fn is_approved(tag: &str) -> bool {
        matches!(
            tag,
            "s.open" | "s.close" | "pkg.ok" | "pkg.fail"
            | "wipe.ok" | "sync.ok" | "sync.fail"
            | "imp.ok" | "imp.fail" | "integ.fail"
        )
    }
}
