//! taktakk Linux CLI entry point.
//!
//! This binary is primarily used for development, testing, and CI.
//! The production application targets Android (APK) and PWA.

use taktakk_a11y::settings::A11ySettings;
use taktakk_facade_clock::clock::FacadeClockState;
use taktakk_facade_clock::gesture::{FacadeInput, GestureConfig, GestureOutcome, GestureParser};
use taktakk_i18n::direction::TextDirection;
use taktakk_i18n::locale::LocaleTag;
use taktakk_i18n::lookup::I18nBundle;

fn main() {
    println!("taktakk v{}", env!("CARGO_PKG_VERSION"));
    println!();

    demo_clock_and_unlock();
    demo_i18n();
    demo_a11y();
}

fn demo_clock_and_unlock() {
    println!("=== Clock Facade Demo ===");

    let _clock = FacadeClockState::default();
    println!("Clock facade initialised (digital mode).");

    let config = GestureConfig::default_config();
    println!(
        "Unlock trigger: alarm {:02}:{:02} + long-press",
        config.drift_h, config.drift_m
    );
    println!(
        "Duress trigger: alarm {:02}:{:02} + long-press",
        config.offset_h, config.offset_m
    );

    let mut parser = GestureParser::new(config);
    parser.process(FacadeInput::AlarmSet { hour: 3, minute: 14 });
    let outcome = parser.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3000 });

    match outcome {
        GestureOutcome::Unlock => println!("-> Unlock gesture recognised."),
        GestureOutcome::Duress => println!("-> Duress gesture recognised."),
        GestureOutcome::Pending => println!("-> Gesture accumulating..."),
        GestureOutcome::Idle => println!("-> No special gesture."),
    }
    println!();
}

fn demo_i18n() {
    println!("=== i18n Demo ===");

    let mut bundle = I18nBundle::new("en");
    bundle.add_locale("en", [("welcome".to_string(), "Welcome".to_string())].into());
    bundle.add_locale("ar", [("welcome".to_string(), "\u{0645}\u{0631}\u{062d}\u{0628}\u{0627}\u{064b}".to_string())].into());

    for tag_str in ["en", "ar", "sw"] {
        let tag = LocaleTag::new(tag_str);
        let dir_label = match tag.direction() {
            TextDirection::Ltr => "LTR",
            TextDirection::Rtl => "RTL",
        };
        let text = bundle.t(&tag, "welcome");
        println!("  [{tag_str}] ({dir_label}) -> \"{text}\"");
    }
    println!();
}

fn demo_a11y() {
    println!("=== Accessibility Demo ===");
    let settings = A11ySettings::default();
    println!(
        "  Contrast: {:?}  Motion: {:?}  Touch: {}",
        settings.contrast_mode,
        settings.motion_preference,
        if settings.large_touch_targets { "large" } else { "normal" }
    );
    println!();
}
