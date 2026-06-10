//! Locale fixture packs for testing LTR (English, Swahili) and RTL (Arabic).
//!
//! These are minimal in-memory bundles, not full translations.
//! Use them in tests and CI; do NOT ship as production content.

use crate::lookup::{I18nBundle, StringMap};

/// Build a fixture bundle with English (en), Arabic (ar), and Swahili (sw).
pub fn fixture_bundle() -> I18nBundle {
    let mut bundle = I18nBundle::new("en");
    bundle.add_locale("en", english_strings());
    bundle.add_locale("ar", arabic_strings());
    bundle.add_locale("sw", swahili_strings());
    bundle
}

fn english_strings() -> StringMap {
    [
        ("app.name",         "Clock"),
        ("nav.back",         "Back"),
        ("nav.next",         "Next"),
        ("nav.listen",       "Listen"),
        ("lesson.start",     "Start"),
        ("lesson.complete",  "Lesson Complete"),
        ("exercise.correct", "Correct!"),
        ("exercise.wrong",   "Try again"),
        ("shield.title",     "Shield"),
        ("spear.title",      "Spear"),
        ("welcome",          "Welcome"),
        ("settings.contrast","High Contrast"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn arabic_strings() -> StringMap {
    [
        ("app.name",         "\u{0633}\u{0627}\u{0639}\u{0629}"),        // ساعة
        ("nav.back",         "\u{0631}\u{062c}\u{0648}\u{0639}"),        // رجوع
        ("nav.next",         "\u{0627}\u{0644}\u{062a}\u{0627}\u{0644}\u{064a}"), // التالي
        ("nav.listen",       "\u{0627}\u{0633}\u{062a}\u{0645}\u{0639}"), // استمع
        ("lesson.start",     "\u{0627}\u{0628}\u{062f}\u{0623}"),
        ("lesson.complete",  "\u{0627}\u{0643}\u{062a}\u{0645}\u{0644}\u{062a} \u{0627}\u{0644}\u{062f}\u{0631}\u{0633}"),
        ("exercise.correct", "\u{0635}\u{062d}\u{064a}\u{062d}!"),        // صحيح!
        ("exercise.wrong",   "\u{062d}\u{0627}\u{0648}\u{0644} \u{0645}\u{0631}\u{0629} \u{0623}\u{062e}\u{0631}\u{0649}"),
        ("shield.title",     "\u{062f}\u{0631}\u{0639}"),                // درع
        ("spear.title",      "\u{0631}\u{0645}\u{062d}"),                // رمح
        ("welcome",          "\u{0645}\u{0631}\u{062d}\u{0628}\u{0627}\u{064b}"), // مرحباً
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}

fn swahili_strings() -> StringMap {
    [
        ("app.name",         "Saa"),
        ("nav.back",         "Nyuma"),
        ("nav.next",         "Mbele"),
        ("nav.listen",       "Sikiliza"),
        ("lesson.start",     "Anza"),
        ("lesson.complete",  "Somo Limekamilika"),
        ("exercise.correct", "Sahihi!"),
        ("exercise.wrong",   "Jaribu tena"),
        ("shield.title",     "Ngao"),
        ("spear.title",      "Mkuki"),
        ("welcome",          "Karibu"),
    ]
    .into_iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect()
}
