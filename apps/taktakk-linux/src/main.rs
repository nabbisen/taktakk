//! taktakk Linux CLI entry point (M1–M4 integration demo).

use taktakk_a11y::settings::A11ySettings;
use taktakk_facade_clock::gesture::{FacadeInput, GestureConfig, GestureOutcome, GestureParser};
use taktakk_i18n::{
    direction::TextDirection,
    fixtures::fixture_bundle,
    locale::LocaleTag,
    navigation::NavigationArrows,
};
use taktakk_module_engine::{
    catalog::{build_tile, DashboardView},
    exercise::ExerciseAnswer,
    runner::{LessonRunner, RunnerEvent},
    state::LessonState,
    step::{ExerciseKind, ExerciseSpec, StepContent, StepKind},
};
use taktakk_core::domain::curriculum::{Module, ModuleStatus, ModuleVersion};

fn main() {
    println!("taktakk v{}", env!("CARGO_PKG_VERSION"));
    println!();

    demo_unlock();
    demo_i18n_rtl();
    demo_dashboard();
    demo_lesson_runner();
    demo_a11y();
}

// ── M1: Clock facade & unlock ─────────────────────────────────────────────────

fn demo_unlock() {
    println!("=== Clock Facade / Unlock (M1) ===");
    let cfg = GestureConfig::default_config();
    let mut p = GestureParser::new(cfg);
    p.process(FacadeInput::AlarmSet { hour: 3, minute: 14 });
    let out = p.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3000 });
    match out {
        GestureOutcome::Unlock => println!("  Unlock gesture -> shell opened"),
        GestureOutcome::Duress => println!("  Duress gesture -> wipe initiated"),
        _ => println!("  No action"),
    }
    println!();
}

// ── M1+M4: i18n + RTL navigation ─────────────────────────────────────────────

fn demo_i18n_rtl() {
    println!("=== i18n / RTL Navigation (M1+M4) ===");
    let bundle = fixture_bundle();

    for locale_str in ["en", "ar", "sw"] {
        let locale = LocaleTag::new(locale_str);
        let dir = locale.direction();
        let dir_label = match dir {
            TextDirection::Ltr => "LTR",
            TextDirection::Rtl => "RTL",
        };
        let nav = NavigationArrows::for_direction(dir);
        let back_label  = bundle.t(&locale, "nav.back");
        let next_label  = bundle.t(&locale, "nav.next");
        let back_arrow  = if nav.back  == taktakk_i18n::navigation::ArrowDir::Left { "<-" } else { "->" };
        let next_arrow  = if nav.forward == taktakk_i18n::navigation::ArrowDir::Right { "->" } else { "<-" };
        println!("  [{locale_str}] ({dir_label})  [{back_arrow} {back_label}]  [{next_label} {next_arrow}]");
    }
    println!();
}

// ── M4: Dashboard catalog ─────────────────────────────────────────────────────

fn demo_dashboard() {
    println!("=== Shield / Spear Dashboard (M4) ===");
    let shield_modules = vec![
        make_module("shield-water",     "shield-hygiene", "shield.water.title"),
        make_module("shield-first-aid", "shield-medical", "shield.firstaid.title"),
    ];
    let spear_modules = vec![
        make_module("spear-math",  "spear-logic", "spear.math.title"),
        make_module("spear-comms", "spear-comms", "spear.comms.title"),
    ];

    let progress = [
        ("shield-water", 8u32, 10u32),
        ("shield-first-aid", 0, 12),
        ("spear-math", 5, 5),
        ("spear-comms", 2, 8),
    ];

    let mut shield_tiles = vec![];
    let mut spear_tiles  = vec![];

    for m in &shield_modules {
        let (_, done, total) = progress.iter().find(|(id, _, _)| *id == m.module_id).unwrap();
        shield_tiles.push(build_tile(m, *done, *total));
    }
    for m in &spear_modules {
        let (_, done, total) = progress.iter().find(|(id, _, _)| *id == m.module_id).unwrap();
        spear_tiles.push(build_tile(m, *done, *total));
    }

    let dash = DashboardView::new(shield_tiles, spear_tiles);
    for tile in dash.all_tiles() {
        let pct = (tile.progress.fraction() * 100.0) as u32;
        println!("  [{:?}] {} — {}%  {:?}", tile.axis, tile.module_id, pct, tile.progress);
    }
    println!("  Completed modules: {}", dash.completed_count());
    println!();
}

// ── M4: Lesson runner with exercise ──────────────────────────────────────────

fn demo_lesson_runner() {
    println!("=== Lesson Runner (M4) ===");
    let steps = vec![
        // Step 0: Text
        StepContent {
            step_id: "s0".to_string(), sort_order: 0,
            kind: StepKind::Text { text_key: "lesson.intro".to_string() },
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 1: Multiple-choice exercise
        StepContent {
            step_id: "s1".to_string(), sort_order: 1,
            kind: StepKind::Exercise(ExerciseSpec {
                exercise_id: "ex-1".to_string(),
                kind: ExerciseKind::MultipleChoice {
                    question_key: "q.water.safe".to_string(),
                    options: vec![
                        ("A".to_string(), "opt.boil".to_string()),
                        ("B".to_string(), "opt.filter".to_string()),
                        ("C".to_string(), "opt.pray".to_string()),
                    ],
                    correct_option_id: "A".to_string(),
                },
            }),
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 2: Text (final)
        StepContent {
            step_id: "s2".to_string(), sort_order: 2,
            kind: StepKind::Text { text_key: "lesson.summary".to_string() },
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
    ];

    let state = LessonState::new("profile-001".to_string(), "shield-water-01".to_string(), 3, 0);
    let mut runner = LessonRunner::new(state, steps);

    // Step 0: text advance
    let r = runner.handle(RunnerEvent::Advance).unwrap();
    println!("  Step 0 advance -> {:?}", r);

    // Step 1: wrong answer first, then correct
    let wrong = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "C".to_string() },
    )).unwrap();
    println!("  Step 1 wrong answer -> {:?}", wrong);

    let correct = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() },
    )).unwrap();
    println!("  Step 1 correct answer -> {:?}", correct);

    // Step 2: final step
    let done = runner.handle(RunnerEvent::Advance).unwrap();
    println!("  Step 2 advance -> {:?}", done);
    println!("  Progress: {:.0}%", runner.state.progress_fraction() * 100.0);
    println!();
}

// ── M1: Accessibility ─────────────────────────────────────────────────────────

fn demo_a11y() {
    println!("=== Accessibility Settings (M1) ===");
    let s = A11ySettings::default();
    println!(
        "  Contrast: {:?}  Motion: {:?}  Touch: {}  Scale: {:.1}x",
        s.contrast_mode, s.motion_preference,
        if s.large_touch_targets { "large(48dp+)" } else { "normal" },
        s.clamped_text_scale(),
    );
    println!();
}

fn make_module(id: &str, cat: &str, title_key: &str) -> Module {
    Module {
        module_id: id.to_string(),
        category_id: cat.to_string(),
        title_key: title_key.to_string(),
        description_key: format!("{id}.desc"),
        version: ModuleVersion::new(1, 0, 0),
        status: ModuleStatus::Available,
        estimated_minutes: Some(15),
    }
}
