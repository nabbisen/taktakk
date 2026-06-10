//! Unit tests for taktakk-i18n.

use crate::direction::TextDirection;
use crate::locale::LocaleTag;
use crate::lookup::I18nBundle;

// --- TextDirection ---

#[test]
fn arabic_is_rtl() {
    assert_eq!(TextDirection::for_language("ar"), TextDirection::Rtl);
}

#[test]
fn english_is_ltr() {
    assert_eq!(TextDirection::for_language("en"), TextDirection::Ltr);
}

#[test]
fn swahili_is_ltr() {
    assert_eq!(TextDirection::for_language("sw"), TextDirection::Ltr);
}

#[test]
fn persian_is_rtl() {
    assert_eq!(TextDirection::for_language("fa"), TextDirection::Rtl);
}

#[test]
fn direction_html_attr() {
    assert_eq!(TextDirection::Rtl.html_attr(), "rtl");
    assert_eq!(TextDirection::Ltr.html_attr(), "ltr");
}

// --- LocaleTag ---

#[test]
fn locale_tag_language() {
    let tag = LocaleTag::new("ar-SY");
    assert_eq!(tag.language(), "ar");
}

#[test]
fn locale_tag_region() {
    let tag = LocaleTag::new("en-US");
    assert_eq!(tag.region(), Some("US"));
}

#[test]
fn locale_tag_no_region() {
    let tag = LocaleTag::new("sw");
    assert_eq!(tag.region(), None);
}

#[test]
fn locale_tag_direction_from_language() {
    let ar = LocaleTag::new("ar-EG");
    assert_eq!(ar.direction(), TextDirection::Rtl);

    let en = LocaleTag::new("en-GB");
    assert_eq!(en.direction(), TextDirection::Ltr);
}

// --- I18nBundle (3-tier fallback) ---

fn make_bundle() -> I18nBundle {
    let mut bundle = I18nBundle::new("en");
    bundle.add_locale("en", [
        ("greeting".to_string(), "Hello".to_string()),
        ("start".to_string(), "Start".to_string()),
    ].into());
    bundle.add_locale("ar", [
        ("greeting".to_string(), "مرحبا".to_string()),
    ].into());
    bundle.add_locale("ar-SY", [
        ("greeting".to_string(), "أهلاً".to_string()),
    ].into());
    bundle
}

#[test]
fn exact_locale_match() {
    let b = make_bundle();
    let tag = LocaleTag::new("ar-SY");
    assert_eq!(b.get(&tag, "greeting"), Some("أهلاً"));
}

#[test]
fn falls_back_to_language() {
    let b = make_bundle();
    // "ar-EG" has no specific entry; should fall back to "ar".
    let tag = LocaleTag::new("ar-EG");
    assert_eq!(b.get(&tag, "greeting"), Some("مرحبا"));
}

#[test]
fn falls_back_to_fallback_locale() {
    let b = make_bundle();
    // Arabic has no "start" key; falls back to English.
    let tag = LocaleTag::new("ar");
    assert_eq!(b.get(&tag, "start"), Some("Start"));
}

#[test]
fn missing_key_returns_none() {
    let b = make_bundle();
    let tag = LocaleTag::new("en");
    assert_eq!(b.get(&tag, "nonexistent"), None);
}

#[test]
fn t_returns_key_when_missing() {
    let b = make_bundle();
    let tag = LocaleTag::new("en");
    assert_eq!(b.t(&tag, "missing_key"), "missing_key");
}

// ── Navigation direction tests ────────────────────────────────────────────────

use crate::navigation::{ArrowDir, NavigationArrows, icon_mirror_policy, IconMirrorPolicy};

#[test]
fn ltr_forward_is_right() {
    let nav = NavigationArrows::for_direction(TextDirection::Ltr);
    assert_eq!(nav.forward, ArrowDir::Right);
    assert_eq!(nav.back,    ArrowDir::Left);
}

#[test]
fn rtl_forward_is_left() {
    let nav = NavigationArrows::for_direction(TextDirection::Rtl);
    assert_eq!(nav.forward, ArrowDir::Left);
    assert_eq!(nav.back,    ArrowDir::Right);
}

#[test]
fn arrow_icon_mirrors_in_rtl() {
    assert_eq!(icon_mirror_policy("arrow-back"),    IconMirrorPolicy::Mirror);
    assert_eq!(icon_mirror_policy("arrow-forward"), IconMirrorPolicy::Mirror);
}

#[test]
fn safety_icon_never_mirrors() {
    assert_eq!(icon_mirror_policy("emergency-exit"), IconMirrorPolicy::NeverMirror);
    assert_eq!(icon_mirror_policy("water-drop"),     IconMirrorPolicy::NeverMirror);
}

// ── Fixture bundle tests ──────────────────────────────────────────────────────

use crate::fixtures::fixture_bundle;

#[test]
fn fixture_bundle_english_nav_keys() {
    let b = fixture_bundle();
    let en = LocaleTag::new("en");
    assert_eq!(b.t(&en, "nav.back"),    "Back");
    assert_eq!(b.t(&en, "nav.next"),    "Next");
    assert_eq!(b.t(&en, "shield.title"), "Shield");
}

#[test]
fn fixture_bundle_arabic_has_rtl_direction() {
    let ar = LocaleTag::new("ar");
    assert_eq!(ar.direction(), TextDirection::Rtl);
}

#[test]
fn fixture_bundle_arabic_welcome() {
    let b = fixture_bundle();
    let ar = LocaleTag::new("ar");
    let val = b.get(&ar, "welcome").expect("Arabic welcome should exist");
    assert!(!val.is_empty());
}

#[test]
fn fixture_bundle_swahili_is_ltr() {
    let sw = LocaleTag::new("sw");
    assert_eq!(sw.direction(), TextDirection::Ltr);
}

#[test]
fn fixture_bundle_fallback_from_ar_eg_to_ar() {
    let b = fixture_bundle();
    let ar_eg = LocaleTag::new("ar-EG");
    // ar-EG has no specific entry; should fall through to ar
    let val = b.get(&ar_eg, "nav.back");
    assert!(val.is_some());
}

#[test]
fn fixture_bundle_fallback_to_english() {
    let b = fixture_bundle();
    // Swahili has no "settings.contrast" key; falls back to English
    let sw = LocaleTag::new("sw");
    assert_eq!(b.get(&sw, "settings.contrast"), Some("High Contrast"));
}
