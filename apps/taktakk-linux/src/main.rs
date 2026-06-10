//! taktakk Linux CLI — end-to-end integration demo (v0.9.0).
//!
//! Demonstrates the complete platform pipeline:
//! 1. Clock facade + stealth unlock gesture
//! 2. Sample package install and verification
//! 3. Lesson runner with exercise
//! 4. Sync inventory diff between two devices
//! 5. State wipe + health check
//! 6. i18n RTL/LTR + accessibility audit

use taktakk_a11y::{audit::audit, settings::A11ySettings};
use taktakk_content::{
    fixtures::test_trust_anchor,
    install::{install_package, InstallOutcome},
    samples::{build_shield_water_package, build_spear_math_package},
};
use taktakk_core::{
    use_cases::health_check::run_static_health_checks,
};
use taktakk_facade_clock::gesture::{
    FacadeInput, GestureConfig, GestureOutcome, GestureParser,
};
use taktakk_i18n::{
    direction::TextDirection,
    fixtures::fixture_bundle,
    locale::LocaleTag,
    navigation::NavigationArrows,
};
use taktakk_module_engine::{
    catalog::build_tile,
    exercise::ExerciseAnswer,
    runner::{LessonRunner, RunnerEvent},
    state::LessonState,
    step::{ExerciseKind, ExerciseSpec, StepContent, StepKind},
};
use taktakk_storage::{repo, wipe as storage_wipe};
use taktakk_sync::{
    inventory::LocalInventory,
    manifest::{build_transfer_plan, SyncAction},
};

#[tokio::main]
async fn main() {
    println!("┌─────────────────────────────────────────────┐");
    println!("│  taktakk v{}  — end-to-end demo         │", env!("CARGO_PKG_VERSION"));
    println!("└─────────────────────────────────────────────┘");
    println!();

    // 1. Clock facade + unlock
    demo_clock_and_unlock();

    // 2. Package install (async — uses tokio::main)
    demo_package_install().await;

    // 3. Lesson runner
    demo_lesson_runner();

    // 4. Sync inventory
    demo_sync_inventory();

    // 5. i18n + RTL
    demo_i18n_rtl();

    // 6. Accessibility audit
    demo_a11y_audit();
}

// ── 1. Clock facade + stealth unlock ─────────────────────────────────────────

fn demo_clock_and_unlock() {
    println!("═══ 1. Clock Facade + Stealth Unlock ═══════════");
    let cfg = GestureConfig::default_config();
    println!("Facade shows a plain clock. Unlock: set alarm {:02}:{:02} + hold Save.",
        cfg.drift_h, cfg.drift_m);
    println!("Duress: set alarm {:02}:{:02} + hold Save → silent wipe.",
        cfg.offset_h, cfg.offset_m);

    let mut parser = GestureParser::new(cfg.clone());

    // Simulate unlock
    parser.process(FacadeInput::AlarmSet { hour: cfg.drift_h, minute: cfg.drift_m });
    let out = parser.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3000 });
    println!("Gesture result: {:?}", out);
    assert_eq!(out, GestureOutcome::Unlock);

    // Simulate duress
    parser.reset();
    parser.process(FacadeInput::AlarmSet { hour: cfg.offset_h, minute: cfg.offset_m });
    let duress = parser.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3500 });
    println!("Duress result:  {:?}", duress);
    assert_eq!(duress, GestureOutcome::Duress);
    println!();
}

// ── 2. Package install ────────────────────────────────────────────────────────

async fn demo_package_install() {
    println!("═══ 2. Package Install + Catalog ═══════════════");

    let data_dir = std::env::temp_dir().join(format!("taktakk-demo-{}", nonce()));
    std::fs::create_dir_all(&data_dir).unwrap();

    let db = taktakk_storage::db::Database::open(&data_dir).await
        .expect("open database");
    let store = taktakk_storage::object_store::FsObjectStore::new(
        data_dir.join("objects")
    );
    let anchors = vec![test_trust_anchor()];

    for (i, (module_id, builder)) in [
        ("shield-water-purification", build_shield_water_package()),
        ("spear-basic-math",          build_spear_math_package()),
    ].iter().enumerate() {
        let nmp = builder.as_ref().expect("build sample package");
        let outcome = install_package(nmp, &format!("pkg-{i:03}"), &anchors, &store, i as i64);
        match outcome {
            InstallOutcome::Installed { package } => {
                println!("  ✓ Installed: {} (v{})", module_id, package.version);
                repo::package::save(&db.core, &package).await.unwrap();
            }
            InstallOutcome::Quarantined { reason } => {
                println!("  ✗ Quarantined {module_id}: {reason}");
            }
        }
    }

    // Build catalog tiles
    let pkgs = repo::package::list(&db.core).await.unwrap();
    println!("  Catalog ({} installed packages):", pkgs.len());

    for pkg in &pkgs {
        let module = taktakk_core::domain::curriculum::Module {
            module_id: pkg.module_id.clone(),
            category_id: if pkg.module_id.starts_with("shield") {
                "shield-hygiene".to_string()
            } else {
                "spear-logic".to_string()
            },
            title_key: format!("{}.title", pkg.module_id),
            description_key: format!("{}.desc", pkg.module_id),
            version: pkg.version.clone(),
            status: taktakk_core::domain::curriculum::ModuleStatus::Available,
            estimated_minutes: Some(10),
        };
        let tile = build_tile(&module, 0, 5);
        println!("    [{:?}] {} — {:?}", tile.axis, tile.module_id, tile.progress);
    }

    // State wipe demo
    storage_wipe::state_wipe(&db.core).await.unwrap();
    println!("  State wipe: progress cleared, packages intact.");

    // Health check
    let report = run_static_health_checks(pkgs.len(), anchors.len(), 2, Some(500_000_000), 0);
    println!("  Health: {}", report.summary());

    let _ = std::fs::remove_dir_all(&data_dir);
    println!();
}

// ── 3. Lesson runner ──────────────────────────────────────────────────────────

fn demo_lesson_runner() {
    println!("═══ 3. Lesson Runner ════════════════════════════");
    let steps = vec![
        text_step("s0", 0, "shield.water.s0.text"),
        mc_step("s1", 1, "A", &["A: Boil 1 min", "B: Add sand", "C: Sunlight"]),
        text_step("s2", 2, "shield.water.s4.summary"),
    ];

    let state = LessonState::new("user-001".to_string(), "lesson-01".to_string(), 3, 0);
    let mut runner = LessonRunner::new(state, steps);

    // Step 0: advance
    let r0 = runner.handle(RunnerEvent::Advance).unwrap();
    println!("  Step 0 advance → {:?}", r0);

    // Step 1: wrong answer, then correct
    let wrong = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "C".to_string() },
    )).unwrap();
    println!("  Step 1 wrong  → {:?}", wrong);

    let ok = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() },
    )).unwrap();
    println!("  Step 1 correct → {:?}", ok);

    // Step 2: final
    let done = runner.handle(RunnerEvent::Advance).unwrap();
    println!("  Step 2 final  → {:?}", done);
    println!("  Progress: {:.0}%", runner.state.progress_fraction() * 100.0);
    println!();
}

fn text_step(id: &str, order: u32, key: &str) -> StepContent {
    StepContent {
        step_id: id.to_string(), sort_order: order,
        kind: StepKind::Text { text_key: key.to_string() },
        caption_key: None, audio_object_hash: None, aria_label_key: None,
    }
}

fn mc_step(id: &str, order: u32, correct: &str, opts: &[&str]) -> StepContent {
    StepContent {
        step_id: id.to_string(), sort_order: order,
        kind: StepKind::Exercise(ExerciseSpec {
            exercise_id: format!("ex-{id}"),
            kind: ExerciseKind::MultipleChoice {
                question_key: "q".to_string(),
                options: opts.iter().enumerate().map(|(i, _)| {
                    let id = (b'A' + i as u8) as char;
                    (id.to_string(), id.to_string())
                }).collect(),
                correct_option_id: correct.to_string(),
            },
        }),
        caption_key: None, audio_object_hash: None, aria_label_key: None,
    }
}

// ── 4. Sync inventory diff ────────────────────────────────────────────────────

fn demo_sync_inventory() {
    println!("═══ 4. Sync Inventory (P2P diff) ════════════════");

    let device_a = LocalInventory::build(vec![
        ("shield-water-purification".to_string(), "1.0.0".to_string(), "hash-a".to_string()),
    ]);
    let device_b = LocalInventory::build(vec![
        ("shield-water-purification".to_string(), "1.0.0".to_string(), "hash-a".to_string()),
        ("spear-basic-math".to_string(),          "1.0.0".to_string(), "hash-b".to_string()),
        ("shield-first-aid-basics".to_string(),   "1.0.0".to_string(), "hash-c".to_string()),
    ]);

    println!("  Device A has {} package(s).", device_a.items.len());
    println!("  Device B has {} package(s).", device_b.items.len());

    let plan = build_transfer_plan(&device_a, &device_b);
    for item in &plan {
        let label = match item.action {
            SyncAction::Receive    => "← RECEIVE",
            SyncAction::Send       => "→ SEND   ",
            SyncAction::Skip       => "= SKIP   ",
            SyncAction::VerifyOnly => "? VERIFY ",
        };
        println!("    {label}  {}", item.package_id);
    }
    println!();
}

// ── 5. i18n + RTL ────────────────────────────────────────────────────────────

fn demo_i18n_rtl() {
    println!("═══ 5. i18n + RTL Navigation ════════════════════");
    let bundle = fixture_bundle();

    for tag_str in ["en", "ar", "sw"] {
        let tag = LocaleTag::new(tag_str);
        let dir = tag.direction();
        let nav = NavigationArrows::for_direction(dir);
        let dir_label = match dir { TextDirection::Ltr => "LTR", TextDirection::Rtl => "RTL" };
        let back_arrow  = if nav.back  == taktakk_i18n::navigation::ArrowDir::Left  { "←" } else { "→" };
        let fwd_arrow   = if nav.forward == taktakk_i18n::navigation::ArrowDir::Right { "→" } else { "←" };
        let back_label  = bundle.t(&tag, "nav.back");
        let next_label  = bundle.t(&tag, "nav.next");
        println!("  [{tag_str}] ({dir_label})  [{back_arrow} {back_label}]  [{next_label} {fwd_arrow}]");
    }
    println!();
}

// ── 6. Accessibility audit ────────────────────────────────────────────────────

fn demo_a11y_audit() {
    println!("═══ 6. Accessibility Audit (ABDD) ═══════════════");
    let settings = A11ySettings::default();
    let report = audit(&settings);
    println!("  {}", report.summary());
    for check in &report.checks {
        let mark = if check.passed { "✓" } else { "✗" };
        println!("  {mark} {}", check.check_id);
    }
    println!();
}

fn nonce() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos() as u64
}
