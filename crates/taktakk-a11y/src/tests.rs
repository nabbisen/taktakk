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
