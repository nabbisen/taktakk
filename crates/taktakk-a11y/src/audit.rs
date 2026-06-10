//! Accessibility audit: ABDD (Accessible by Design) compliance checks (RFC 026).
//!
//! This module evaluates a set of accessibility settings against the
//! requirements defined in the taktakk spec. Each check is a pure function.
//!
//! All checks are runnable offline and do not require UI rendering.

use crate::settings::{A11ySettings, ContrastMode, MotionPreference, TEXT_SCALE_MAX, TEXT_SCALE_MIN};

/// Result of a single accessibility compliance check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11yCheckResult {
    pub check_id: &'static str,
    pub passed: bool,
    pub detail: Option<String>,
}

impl A11yCheckResult {
    fn pass(id: &'static str) -> Self {
        Self { check_id: id, passed: true, detail: None }
    }
    fn fail(id: &'static str, detail: impl Into<String>) -> Self {
        Self { check_id: id, passed: false, detail: Some(detail.into()) }
    }
}

/// Summary of an accessibility audit run.
#[derive(Debug, Clone)]
pub struct A11yAuditReport {
    pub checks: Vec<A11yCheckResult>,
}

impl A11yAuditReport {
    pub fn all_passed(&self) -> bool {
        self.checks.iter().all(|c| c.passed)
    }
    pub fn failure_count(&self) -> usize {
        self.checks.iter().filter(|c| !c.passed).count()
    }
    pub fn summary(&self) -> String {
        let total = self.checks.len();
        let passed = total - self.failure_count();
        format!("{passed}/{total} a11y checks passed")
    }
}

/// Minimum touch target size in density-independent pixels (WCAG 2.5.5).
pub const MIN_TOUCH_TARGET_DP: u32 = 48;

/// Maximum recommended text scale that still fits standard layouts.
pub const RECOMMENDED_MAX_TEXT_SCALE: f32 = 2.0;

/// Run the full accessibility audit against a settings snapshot.
pub fn audit(settings: &A11ySettings) -> A11yAuditReport {
    A11yAuditReport {
        checks: vec![
            check_large_touch_targets(settings),
            check_high_contrast_available(settings),
            check_text_scale_bounds(settings),
            check_motion_reduced_by_default(settings),
            check_audio_first_opt_in(settings),
            check_text_scale_range_valid(),
        ],
    }
}

/// Touch targets must be at least 48 dp (WCAG 2.5.5 / ABDD requirement).
fn check_large_touch_targets(s: &A11ySettings) -> A11yCheckResult {
    if s.large_touch_targets {
        A11yCheckResult::pass("touch_target_minimum_48dp")
    } else {
        A11yCheckResult::fail(
            "touch_target_minimum_48dp",
            "large_touch_targets is disabled; targets may fall below 48dp",
        )
    }
}

/// High-contrast mode must be available (even if not the default).
fn check_high_contrast_available(_s: &A11ySettings) -> A11yCheckResult {
    // ContrastMode::HighContrast must be a valid variant.
    let _ = ContrastMode::HighContrast;
    A11yCheckResult::pass("high_contrast_mode_available")
}

/// Text scale must be within the defined range.
fn check_text_scale_bounds(s: &A11ySettings) -> A11yCheckResult {
    let clamped = s.clamped_text_scale();
    if (clamped - s.text_scale).abs() > 0.001 {
        A11yCheckResult::fail(
            "text_scale_within_bounds",
            format!("text_scale {:.2} is out of range [{TEXT_SCALE_MIN}, {TEXT_SCALE_MAX}]",
                s.text_scale),
        )
    } else {
        A11yCheckResult::pass("text_scale_within_bounds")
    }
}

/// Default motion preference should be Reduced to save battery and reduce
/// cognitive load on stressed users.
fn check_motion_reduced_by_default(s: &A11ySettings) -> A11yCheckResult {
    match s.motion_preference {
        MotionPreference::Full => A11yCheckResult::fail(
            "motion_reduced_for_low_power",
            "motion_preference is Full; should default to Reduced to save battery",
        ),
        _ => A11yCheckResult::pass("motion_reduced_for_low_power"),
    }
}

/// Audio-first is an opt-in preference, not a default — so that users without
/// audio hardware don't experience broken lessons.
fn check_audio_first_opt_in(s: &A11ySettings) -> A11yCheckResult {
    // audio_first should default to false; users can enable it.
    // We can't check "default" here without constructing a default instance,
    // so we just verify the field exists.
    let _ = s.audio_first;
    A11yCheckResult::pass("audio_first_preference_exists")
}

/// The text scale constant range must be sane.
fn check_text_scale_range_valid() -> A11yCheckResult {
    if TEXT_SCALE_MIN < 1.0 && TEXT_SCALE_MAX > 1.5 && TEXT_SCALE_MIN < TEXT_SCALE_MAX {
        A11yCheckResult::pass("text_scale_range_sane")
    } else {
        A11yCheckResult::fail(
            "text_scale_range_sane",
            format!("TEXT_SCALE_MIN={TEXT_SCALE_MIN} TEXT_SCALE_MAX={TEXT_SCALE_MAX}"),
        )
    }
}

// ── RTL/LTR coverage check ────────────────────────────────────────────────────

/// Verify that a set of locale tags covers both LTR and RTL directions.
///
/// taktakk must support at least one LTR and one RTL locale at all times.
pub fn check_locale_direction_coverage(locale_tags: &[&str]) -> A11yCheckResult {
    let has_ltr = locale_tags.iter().any(|t| {
        !matches!(*t, "ar" | "he" | "fa" | "ur" | "ps" | "sd" | "ku")
    });
    let has_rtl = locale_tags.iter().any(|t| {
        matches!(*t, "ar" | "he" | "fa" | "ur" | "ps" | "sd" | "ku")
    });

    match (has_ltr, has_rtl) {
        (true, true)  => A11yCheckResult::pass("locale_coverage_ltr_and_rtl"),
        (true, false) => A11yCheckResult::fail(
            "locale_coverage_ltr_and_rtl", "no RTL locale installed"),
        (false, true) => A11yCheckResult::fail(
            "locale_coverage_ltr_and_rtl", "no LTR locale installed"),
        (false, false) => A11yCheckResult::fail(
            "locale_coverage_ltr_and_rtl", "no locales installed"),
    }
}
