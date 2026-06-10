//! Unit tests for taktakk-a11y.

use crate::settings::{
    A11ySettings, ContrastMode, MotionPreference, TEXT_SCALE_MAX, TEXT_SCALE_MIN,
};

#[test]
fn defaults_are_conservative() {
    let s = A11ySettings::default();
    assert_eq!(s.contrast_mode, ContrastMode::Dark);
    assert_eq!(s.motion_preference, MotionPreference::Reduced);
    assert!(s.large_touch_targets);
    assert!((s.text_scale - 1.0).abs() < f32::EPSILON);
}

#[test]
fn text_scale_clamped_below_min() {
    let s = A11ySettings { text_scale: 0.1, ..Default::default() };
    assert!((s.clamped_text_scale() - TEXT_SCALE_MIN).abs() < f32::EPSILON);
}

#[test]
fn text_scale_clamped_above_max() {
    let s = A11ySettings { text_scale: 99.0, ..Default::default() };
    assert!((s.clamped_text_scale() - TEXT_SCALE_MAX).abs() < f32::EPSILON);
}

#[test]
fn text_scale_in_range_unchanged() {
    let s = A11ySettings { text_scale: 1.5, ..Default::default() };
    assert!((s.clamped_text_scale() - 1.5).abs() < f32::EPSILON);
}

// ── Accessibility audit (M7) ──────────────────────────────────────────────────

use crate::audit::{audit, check_locale_direction_coverage, A11yCheckResult};

#[test]
fn default_settings_pass_audit() {
    let settings = A11ySettings::default();
    let report = audit(&settings);
    assert!(report.all_passed(), "default settings should pass audit: {}", report.summary());
}

#[test]
fn disabled_touch_targets_fails_audit() {
    let s = A11ySettings { large_touch_targets: false, ..Default::default() };
    let report = audit(&s);
    assert!(!report.all_passed());
    let failed: Vec<_> = report.checks.iter().filter(|c| !c.passed).collect();
    assert!(failed.iter().any(|c| c.check_id.contains("touch_target")));
}

#[test]
fn out_of_range_text_scale_fails_audit() {
    let s = A11ySettings { text_scale: 10.0, ..Default::default() };
    let report = audit(&s);
    let failed: Vec<_> = report.checks.iter().filter(|c| !c.passed).collect();
    assert!(failed.iter().any(|c| c.check_id.contains("text_scale")));
}

#[test]
fn full_motion_preference_fails_audit() {
    use crate::settings::MotionPreference;
    let s = A11ySettings { motion_preference: MotionPreference::Full, ..Default::default() };
    let report = audit(&s);
    let failed: Vec<_> = report.checks.iter().filter(|c| !c.passed).collect();
    assert!(failed.iter().any(|c| c.check_id.contains("motion")));
}

#[test]
fn locale_coverage_ltr_and_rtl_passes() {
    let result = check_locale_direction_coverage(&["en", "ar", "sw"]);
    assert!(result.passed);
}

#[test]
fn locale_coverage_missing_rtl_fails() {
    let result = check_locale_direction_coverage(&["en", "sw", "fr"]);
    assert!(!result.passed);
    assert!(result.detail.as_deref().unwrap_or("").contains("RTL"));
}

#[test]
fn locale_coverage_missing_ltr_fails() {
    let result = check_locale_direction_coverage(&["ar", "fa"]);
    assert!(!result.passed);
    assert!(result.detail.as_deref().unwrap_or("").contains("LTR"));
}

#[test]
fn locale_coverage_empty_fails() {
    let result = check_locale_direction_coverage(&[]);
    assert!(!result.passed);
}
