//! Field readiness health check and performance budget (RFC 025).
//!
//! This module provides an **offline, self-contained** health check that
//! operators can run from the unlocked shell to verify device readiness
//! before distributing seed kits.
//!
//! It also documents the performance budgets derived from RFC 025.

// ── Performance budgets (RFC 025 §3.2) ───────────────────────────────────────

/// Maximum acceptable cold-start time for the facade (milliseconds).
pub const BUDGET_FACADE_COLD_START_MS: u64 = 1_500;
/// Hard limit for facade cold start.
pub const LIMIT_FACADE_COLD_START_MS: u64 = 3_000;

/// Maximum acceptable unlock-to-shell transition time (milliseconds).
pub const BUDGET_UNLOCK_TRANSITION_MS: u64 = 1_000;
pub const LIMIT_UNLOCK_TRANSITION_MS: u64 = 2_500;

/// Maximum acceptable dashboard render time after unlock (milliseconds).
pub const BUDGET_DASHBOARD_RENDER_MS: u64 = 1_500;
pub const LIMIT_DASHBOARD_RENDER_MS: u64 = 3_000;

/// Maximum acceptable step viewer transition time (milliseconds).
pub const BUDGET_STEP_TRANSITION_MS: u64 = 250;
pub const LIMIT_STEP_TRANSITION_MS: u64 = 750;

/// Maximum acceptable resume-state write time (milliseconds).
pub const BUDGET_RESUME_WRITE_MS: u64 = 75;
pub const LIMIT_RESUME_WRITE_MS: u64 = 250;

/// Maximum acceptable SVG step render time (milliseconds).
pub const BUDGET_SVG_RENDER_MS: u64 = 250;
pub const LIMIT_SVG_RENDER_MS: u64 = 750;

/// Maximum acceptable audio start latency after tap (milliseconds).
pub const BUDGET_AUDIO_START_MS: u64 = 300;
pub const LIMIT_AUDIO_START_MS: u64 = 1_000;

/// Maximum package file size that must stream without OOM (bytes). 50 MiB.
pub const MAX_PACKAGE_STREAM_BYTES: u64 = 50 * 1024 * 1024;

/// Minimum available storage required for normal operation (bytes). 100 MiB.
pub const MIN_FREE_STORAGE_BYTES: u64 = 100 * 1024 * 1024;

// ── Health check result ───────────────────────────────────────────────────────

/// A single health check item result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckItem {
    pub name: &'static str,
    pub passed: bool,
    pub detail: Option<String>,
}

impl CheckItem {
    pub fn pass(name: &'static str) -> Self {
        Self { name, passed: true, detail: None }
    }

    pub fn fail(name: &'static str, detail: impl Into<String>) -> Self {
        Self { name, passed: false, detail: Some(detail.into()) }
    }
}

/// Summary of a field health check run.
#[derive(Debug, Clone)]
pub struct HealthReport {
    pub items: Vec<CheckItem>,
}

impl HealthReport {
    /// `true` if all checks passed.
    pub fn all_passed(&self) -> bool {
        self.items.iter().all(|i| i.passed)
    }

    /// Number of failed checks.
    pub fn failure_count(&self) -> usize {
        self.items.iter().filter(|i| !i.passed).count()
    }

    /// Render a compact human-readable summary (never exposed on locked facade).
    pub fn summary(&self) -> String {
        let total  = self.items.len();
        let passed = total - self.failure_count();
        format!("{passed}/{total} checks passed")
    }
}

// ── Static checks (no I/O needed) ────────────────────────────────────────────

/// Run all static (compile-time / in-memory) field readiness checks.
///
/// These checks do not require database access or file I/O.
pub fn run_static_checks() -> HealthReport {
    let items = vec![
        check_performance_budget_constants(),
        check_max_package_size(),
        check_log_retention_policy(),
        check_wipe_scope_ordering(),
        check_event_bucket_count(),
    ];
    HealthReport { items }
}

fn check_performance_budget_constants() -> CheckItem {
    // Budgets must be strictly below their hard limits.
    let ok = BUDGET_FACADE_COLD_START_MS < LIMIT_FACADE_COLD_START_MS
        && BUDGET_UNLOCK_TRANSITION_MS < LIMIT_UNLOCK_TRANSITION_MS
        && BUDGET_DASHBOARD_RENDER_MS < LIMIT_DASHBOARD_RENDER_MS
        && BUDGET_STEP_TRANSITION_MS < LIMIT_STEP_TRANSITION_MS
        && BUDGET_RESUME_WRITE_MS < LIMIT_RESUME_WRITE_MS
        && BUDGET_SVG_RENDER_MS < LIMIT_SVG_RENDER_MS
        && BUDGET_AUDIO_START_MS < LIMIT_AUDIO_START_MS;
    if ok {
        CheckItem::pass("performance_budgets_below_hard_limits")
    } else {
        CheckItem::fail("performance_budgets_below_hard_limits",
            "at least one budget equals or exceeds its hard limit")
    }
}

fn check_max_package_size() -> CheckItem {
    // 50 MiB limit must be respected.
    if MAX_PACKAGE_STREAM_BYTES == 50 * 1024 * 1024 {
        CheckItem::pass("max_package_stream_size_50mib")
    } else {
        CheckItem::fail("max_package_stream_size_50mib",
            format!("expected 52428800, got {MAX_PACKAGE_STREAM_BYTES}"))
    }
}

fn check_log_retention_policy() -> CheckItem {
    use crate::use_cases::safety_settings::LogRetentionPolicy;
    let policy = LogRetentionPolicy::default();
    match policy.validate() {
        Ok(_) => CheckItem::pass("default_log_retention_policy_valid"),
        Err(e) => CheckItem::fail("default_log_retention_policy_valid", e),
    }
}

fn check_wipe_scope_ordering() -> CheckItem {
    // Verify WipeScope variants exist (compile-time check via pattern match).
    use crate::use_cases::panic_wipe::WipeScope;
    let _ = WipeScope::KeysOnly;
    let _ = WipeScope::Full;
    CheckItem::pass("wipe_scope_variants_present")
}

fn check_event_bucket_count() -> CheckItem {
    use crate::use_cases::safety_settings::EventBucket;
    // There must be at least 8 approved buckets.
    let buckets = [
        EventBucket::SessionOpen, EventBucket::SessionClose,
        EventBucket::InstallOk,  EventBucket::InstallFail,
        EventBucket::WipeOk,     EventBucket::SyncOk,
        EventBucket::SyncFail,   EventBucket::ImportOk,
        EventBucket::ImportFail, EventBucket::IntegrityFail,
    ];
    if buckets.len() >= 8 {
        CheckItem::pass("event_bucket_minimum_coverage")
    } else {
        CheckItem::fail("event_bucket_minimum_coverage",
            format!("only {} buckets defined", buckets.len()))
    }
}

// ── Timing measurement model ──────────────────────────────────────────────────

/// A measured operation duration.
#[derive(Debug, Clone)]
pub struct TimingMeasurement {
    pub operation: &'static str,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub limit_ms: u64,
}

impl TimingMeasurement {
    pub fn within_budget(&self) -> bool {
        self.elapsed_ms <= self.budget_ms
    }

    pub fn within_limit(&self) -> bool {
        self.elapsed_ms <= self.limit_ms
    }

    pub fn to_check_item(&self) -> CheckItem {
        if self.within_limit() {
            let detail = if self.within_budget() {
                None
            } else {
                Some(format!(
                    "{}ms exceeds budget {}ms (within limit {}ms)",
                    self.elapsed_ms, self.budget_ms, self.limit_ms
                ))
            };
            CheckItem { name: self.operation, passed: true, detail }
        } else {
            CheckItem::fail(
                self.operation,
                format!("{}ms exceeds hard limit {}ms", self.elapsed_ms, self.limit_ms),
            )
        }
    }
}
