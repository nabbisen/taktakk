//! Offline health check service (RFC 036).
//!
//! Runs entirely locally without network access or telemetry.
//! The results are shown only in the unlocked shell; nothing leaks to
//! the facade or any external service.

use serde::{Deserialize, Serialize};

/// Category of a health check item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthCategory {
    Database,
    ObjectStore,
    Packages,
    TrustAnchors,
    StorageSpace,
    LocalePacks,
    PendingMaintenance,
}

/// Severity of a health check finding.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum HealthSeverity {
    Info,
    Warning,
    Error,
}

/// One item in a health check report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthItem {
    pub category: HealthCategory,
    pub severity: HealthSeverity,
    pub message: String,
    pub detail: Option<String>,
}

impl HealthItem {
    pub fn info(cat: HealthCategory, msg: impl Into<String>) -> Self {
        Self { category: cat, severity: HealthSeverity::Info,
               message: msg.into(), detail: None }
    }
    pub fn warning(cat: HealthCategory, msg: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { category: cat, severity: HealthSeverity::Warning,
               message: msg.into(), detail: Some(detail.into()) }
    }
    pub fn error(cat: HealthCategory, msg: impl Into<String>, detail: impl Into<String>) -> Self {
        Self { category: cat, severity: HealthSeverity::Error,
               message: msg.into(), detail: Some(detail.into()) }
    }
}

/// Full health report returned by the health check service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub items: Vec<HealthItem>,
    /// Unix timestamp when the check was run.
    pub checked_at: i64,
}

impl HealthReport {
    pub fn new(checked_at: i64) -> Self {
        Self { items: Vec::new(), checked_at }
    }

    pub fn push(&mut self, item: HealthItem) {
        self.items.push(item);
    }

    /// `true` if no errors were found (warnings are acceptable).
    pub fn is_healthy(&self) -> bool {
        !self.items.iter().any(|i| i.severity == HealthSeverity::Error)
    }

    /// Count items by severity.
    pub fn count(&self, sev: &HealthSeverity) -> usize {
        self.items.iter().filter(|i| &i.severity == sev).count()
    }

    pub fn summary(&self) -> String {
        let errors   = self.count(&HealthSeverity::Error);
        let warnings = self.count(&HealthSeverity::Warning);
        if errors > 0 {
            format!("{errors} error(s), {warnings} warning(s)")
        } else if warnings > 0 {
            format!("healthy with {warnings} warning(s)")
        } else {
            "healthy".to_string()
        }
    }
}

// ── Static health checks (no I/O) ────────────────────────────────────────────

/// Run static health checks (no database or filesystem access).
pub fn run_static_health_checks(
    installed_package_count: usize,
    trusted_anchor_count: usize,
    active_locale_count: usize,
    free_storage_bytes: Option<u64>,
    now: i64,
) -> HealthReport {
    let mut report = HealthReport::new(now);

    // Packages
    if installed_package_count == 0 {
        report.push(HealthItem::warning(
            HealthCategory::Packages,
            "no packages installed",
            "import at least one Shield module before distributing",
        ));
    } else {
        report.push(HealthItem::info(
            HealthCategory::Packages,
            format!("{installed_package_count} package(s) installed"),
        ));
    }

    // Trust anchors
    if trusted_anchor_count == 0 {
        report.push(HealthItem::error(
            HealthCategory::TrustAnchors,
            "no trusted signing keys",
            "no packages can be installed or verified without trust anchors",
        ));
    } else {
        report.push(HealthItem::info(
            HealthCategory::TrustAnchors,
            format!("{trusted_anchor_count} trust anchor(s) active"),
        ));
    }

    // Locale packs
    if active_locale_count == 0 {
        report.push(HealthItem::warning(
            HealthCategory::LocalePacks,
            "no locale packs installed",
            "install at least one locale pack",
        ));
    } else {
        report.push(HealthItem::info(
            HealthCategory::LocalePacks,
            format!("{active_locale_count} locale pack(s) active"),
        ));
    }

    // Storage space
    if let Some(free) = free_storage_bytes {
        use crate::use_cases::field_check::MIN_FREE_STORAGE_BYTES;
        if free < MIN_FREE_STORAGE_BYTES {
            report.push(HealthItem::warning(
                HealthCategory::StorageSpace,
                format!("low storage: {} MB free", free / (1024 * 1024)),
                format!("minimum recommended: {} MB", MIN_FREE_STORAGE_BYTES / (1024 * 1024)),
            ));
        } else {
            report.push(HealthItem::info(
                HealthCategory::StorageSpace,
                format!("{} MB free", free / (1024 * 1024)),
            ));
        }
    }

    report
}

/// Content lifecycle state of a module (RFC 035).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentLifecycleState {
    /// Active and available for learning.
    Active,
    /// Deprecated: replaced by a newer version, but still usable.
    Deprecated { replaced_by: String },
    /// Disabled by a revocation package; content must not run.
    Disabled { reason_key: String },
    /// Quarantined: failed integrity checks.
    Quarantined,
}

impl ContentLifecycleState {
    /// Whether the module can be presented to a learner.
    pub fn is_runnable(&self) -> bool {
        matches!(self, Self::Active | Self::Deprecated { .. })
    }
}
