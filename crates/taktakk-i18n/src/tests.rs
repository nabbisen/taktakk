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
