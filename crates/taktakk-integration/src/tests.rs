//! End-to-end integration tests.

use taktakk_content::install::{install_package, InstallOutcome};
use taktakk_content::samples::{
    build_all_sample_packages, build_shield_water_package, build_spear_math_package,
};
use taktakk_core::domain::package::PackageStatus;
use taktakk_core::domain::profile::LocalProfile;
use taktakk_core::domain::progress::ResumeState;
use taktakk_core::use_cases::health_check::run_static_health_checks;
use taktakk_facade_clock::gesture::{
    FacadeInput, GestureConfig, GestureOutcome, GestureParser,
};
use taktakk_module_engine::{
    exercise::ExerciseAnswer,
    runner::{LessonRunner, RunnerEvent, RunnerResponse},
    state::LessonState,
    step::{ExerciseKind, ExerciseSpec, StepContent, StepKind},
};
use taktakk_security::wipe::overwrite_all_keys;
use taktakk_storage::{repo, wipe as storage_wipe};
use taktakk_sync::inventory::LocalInventory;
use taktakk_sync::manifest::build_transfer_plan;

use crate::harness::TestHarness;

// ── Unlock gesture pipeline ───────────────────────────────────────────────────

#[test]
fn e2e_unlock_gesture_succeeds_with_correct_magic_time() {
    let config = GestureConfig::default_config();
    let mut parser = GestureParser::new(config.clone());

    parser.process(FacadeInput::AlarmSet {
        hour: config.drift_h,
        minute: config.drift_m,
    });
    let outcome = parser.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3000 });
    assert_eq!(outcome, GestureOutcome::Unlock);
}

#[test]
fn e2e_duress_gesture_triggers_wipe_outcome() {
    let config = GestureConfig::default_config();
    let mut parser = GestureParser::new(config.clone());

    parser.process(FacadeInput::AlarmSet {
        hour: config.offset_h,
        minute: config.offset_m,
    });
    let outcome = parser.process(FacadeInput::AlarmConfirmLongPress { duration_ms: 3000 });
    assert_eq!(outcome, GestureOutcome::Duress);
}

// ── Package install → catalog ─────────────────────────────────────────────────

#[tokio::test]
async fn e2e_install_shield_water_package() {
    let h = TestHarness::new("install-water").await;
    let store = h.object_store();

    let nmp = build_shield_water_package().expect("build package");
    let outcome = install_package(&nmp, "pkg-water-001", &h.trust_anchors, &store, 1000);

    match outcome {
        InstallOutcome::Installed { package } => {
            assert_eq!(package.module_id, "shield-water-purification");
            assert_eq!(package.status, PackageStatus::Installed);
            assert!(package.installed_at.is_some());

            repo::package::save(&h.db.core, &package).await.unwrap();
            let pkgs = repo::package::list(&h.db.core).await.unwrap();
            assert_eq!(pkgs.len(), 1);
            assert_eq!(pkgs[0].module_id, "shield-water-purification");
        }
        InstallOutcome::Quarantined { reason } => {
            panic!("expected Installed, got Quarantined: {reason}");
        }
    }
}

#[tokio::test]
async fn e2e_install_all_sample_packages() {
    let h = TestHarness::new("install-all").await;
    let store = h.object_store();

    let packages = build_all_sample_packages();
    assert_eq!(packages.len(), 3, "expected 3 sample packages");

    for (i, (module_id, nmp)) in packages.iter().enumerate() {
        let outcome = install_package(
            nmp,
            &format!("pkg-{i:03}"),
            &h.trust_anchors,
            &store,
            i as i64 * 1000,
        );
        assert!(
            matches!(outcome, InstallOutcome::Installed { .. }),
            "package {module_id} should install cleanly"
        );
    }

    let pkgs = repo::package::list(&h.db.core).await.unwrap();
    assert_eq!(pkgs.len(), 0, "packages are not in DB until explicitly saved");
}

#[tokio::test]
async fn e2e_quarantine_on_tampered_package() {
    use taktakk_content::fixtures::build_tampered_package;

    let h = TestHarness::new("quarantine").await;
    let store = h.object_store();

    let nmp = build_tampered_package("shield-water-purification").unwrap();
    let outcome = install_package(&nmp, "pkg-bad-001", &h.trust_anchors, &store, 0);

    assert!(
        matches!(outcome, InstallOutcome::Quarantined { .. }),
        "tampered package must be quarantined"
    );
}

// ── Profile + progress lifecycle ─────────────────────────────────────────────

#[tokio::test]
async fn e2e_profile_create_and_touch() {
    let h = TestHarness::new("profile").await;

    let profile = LocalProfile::new("user-001".to_string(), 1000);
    repo::profile::save(&h.db.core, &profile).await.unwrap();

    repo::profile::touch(&h.db.core, "user-001", 9999).await.unwrap();
    let p = repo::profile::get(&h.db.core, "user-001").await.unwrap().unwrap();
    assert_eq!(p.last_active_at, Some(9999));
}

#[tokio::test]
async fn e2e_lesson_progress_persists_across_open() {
    let h = TestHarness::new("progress").await;

    // Simulate completing step 2 of a lesson.
    repo::progress::save_resume_state(&h.db.core, &ResumeState {
        profile_id: "user-001".to_string(),
        lesson_id: "shield-water-lesson-01".to_string(),
        last_completed_step_order: 2,
        updated_at: 1000,
    }).await.unwrap();

    // Re-open: resume state should be preserved.
    let state = repo::progress::get_resume_state(
        &h.db.core, "user-001", "shield-water-lesson-01",
    ).await.unwrap().unwrap();
    assert_eq!(state.last_completed_step_order, 2);
}

// ── Lesson runner end-to-end ──────────────────────────────────────────────────

fn make_water_steps() -> Vec<StepContent> {
    vec![
        // Step 0: text
        StepContent {
            step_id: "water-s0".to_string(), sort_order: 0,
            kind: StepKind::Text { text_key: "shield.water.s0.text".to_string() },
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 1: SVG (text for test purposes)
        StepContent {
            step_id: "water-s1".to_string(), sort_order: 1,
            kind: StepKind::Text { text_key: "shield.water.s1.text".to_string() },
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 2: acknowledge
        StepContent {
            step_id: "water-s2".to_string(), sort_order: 2,
            kind: StepKind::Exercise(ExerciseSpec {
                exercise_id: "water-ack".to_string(),
                kind: ExerciseKind::Acknowledge {
                    confirm_key: "shield.water.s2.confirm".to_string(),
                },
            }),
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 3: multiple choice — correct answer is "A"
        StepContent {
            step_id: "water-s3".to_string(), sort_order: 3,
            kind: StepKind::Exercise(ExerciseSpec {
                exercise_id: "water-mc".to_string(),
                kind: ExerciseKind::MultipleChoice {
                    question_key: "shield.water.s3.question".to_string(),
                    options: vec![
                        ("A".to_string(), "shield.water.s3.opt_a".to_string()),
                        ("B".to_string(), "shield.water.s3.opt_b".to_string()),
                        ("C".to_string(), "shield.water.s3.opt_c".to_string()),
                    ],
                    correct_option_id: "A".to_string(),
                },
            }),
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
        // Step 4: summary text
        StepContent {
            step_id: "water-s4".to_string(), sort_order: 4,
            kind: StepKind::Text { text_key: "shield.water.s4.summary".to_string() },
            caption_key: None, audio_object_hash: None, aria_label_key: None,
        },
    ]
}

#[test]
fn e2e_lesson_runner_complete_water_lesson() {
    let state = LessonState::new("user-001".to_string(),
        "shield-water-lesson-01".to_string(), 5, 0);
    let mut runner = LessonRunner::new(state, make_water_steps());

    // Step 0: text → advance
    assert_eq!(
        runner.handle(RunnerEvent::Advance).unwrap(),
        RunnerResponse::StepAdvanced { new_order: 1 }
    );
    // Step 1: text → advance
    assert_eq!(
        runner.handle(RunnerEvent::Advance).unwrap(),
        RunnerResponse::StepAdvanced { new_order: 2 }
    );
    // Step 2: acknowledge
    let r = runner.handle(RunnerEvent::Answer(ExerciseAnswer::Acknowledge)).unwrap();
    assert!(matches!(r, RunnerResponse::AnswerCorrect { new_order: 3 }));

    // Step 3: wrong answer first
    let wrong = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "C".to_string() },
    )).unwrap();
    assert!(matches!(wrong, RunnerResponse::AnswerIncorrect { attempts_used: 1, max_attempts: 3 }));

    // Step 3: correct answer
    let correct = runner.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() },
    )).unwrap();
    assert!(matches!(correct, RunnerResponse::AnswerCorrect { new_order: 4 }));

    // Step 4: final text
    assert_eq!(runner.handle(RunnerEvent::Advance).unwrap(), RunnerResponse::LessonComplete);
    assert!((runner.state.progress_fraction() - 1.0).abs() < f32::EPSILON);
}

#[tokio::test]
async fn e2e_lesson_resumes_after_simulated_crash() {
    let h = TestHarness::new("resume").await;

    // Simulate crash at step 2 — persist resume state
    repo::progress::save_resume_state(&h.db.core, &ResumeState {
        profile_id: "user-001".to_string(),
        lesson_id: "shield-water-lesson-01".to_string(),
        last_completed_step_order: 1,
        updated_at: 500,
    }).await.unwrap();

    // Restore runner from persisted state
    let restored = repo::progress::get_resume_state(
        &h.db.core, "user-001", "shield-water-lesson-01",
    ).await.unwrap().unwrap();

    let mut lesson_state = LessonState::new(
        "user-001".to_string(),
        "shield-water-lesson-01".to_string(),
        5,
        0,
    );
    // Replay completed steps
    for order in 0..=restored.last_completed_step_order {
        lesson_state.complete_step(order);
    }

    assert_eq!(lesson_state.next_step_order(), 2, "should resume at step 2");
}

// ── Sync inventory ────────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_sync_inventory_exchange_and_diff() {
    let h = TestHarness::new("sync").await;
    let store = h.object_store();

    // Install one package
    let nmp = build_shield_water_package().unwrap();
    let outcome = install_package(&nmp, "pkg-001", &h.trust_anchors, &store, 0);
    let pkg = match outcome {
        InstallOutcome::Installed { package } => package,
        _ => panic!("install failed"),
    };
    repo::package::save(&h.db.core, &pkg).await.unwrap();

    // Build local inventory
    let pkgs = repo::package::list(&h.db.core).await.unwrap();
    let inv_local = LocalInventory::build(
        pkgs.iter().map(|p| (
            p.package_id.clone(),
            p.version.to_string(),
            p.manifest_hash.clone(),
        )).collect(),
    );

    // Simulate remote with an extra package
    let inv_remote = LocalInventory::build(vec![
        (pkg.package_id.clone(), pkg.version.to_string(), pkg.manifest_hash.clone()),
        ("spear-basic-math".to_string(), "1.0.0".to_string(), "aaaa".repeat(16)),
    ]);

    let plan = build_transfer_plan(&inv_local, &inv_remote);
    let receives: Vec<_> = plan.iter()
        .filter(|i| i.action == taktakk_sync::manifest::SyncAction::Receive)
        .collect();
    assert_eq!(receives.len(), 1);
    assert_eq!(receives[0].package_id, "spear-basic-math");
}

// ── State wipe ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn e2e_state_wipe_removes_progress_keeps_packages() {
    let h = TestHarness::new("wipe-state").await;
    let store = h.object_store();

    // Install a package
    let nmp = build_shield_water_package().unwrap();
    let outcome = install_package(&nmp, "pkg-001", &h.trust_anchors, &store, 0);
    let pkg = match outcome {
        InstallOutcome::Installed { package } => package,
        _ => panic!("install failed"),
    };
    repo::package::save(&h.db.core, &pkg).await.unwrap();

    // Save some progress
    repo::progress::save_resume_state(&h.db.core, &ResumeState {
        profile_id: "user-001".to_string(),
        lesson_id: "shield-water-lesson-01".to_string(),
        last_completed_step_order: 3,
        updated_at: 0,
    }).await.unwrap();

    // State wipe
    storage_wipe::state_wipe(&h.db.core).await.unwrap();

    // Progress gone
    let rs = repo::progress::get_resume_state(
        &h.db.core, "user-001", "shield-water-lesson-01",
    ).await.unwrap();
    assert!(rs.is_none(), "resume state should be wiped");

    // Package still present
    let pkgs = repo::package::list(&h.db.core).await.unwrap();
    assert_eq!(pkgs.len(), 1, "package should survive state wipe");
}

#[tokio::test]
async fn e2e_factory_reset_leaves_empty_db() {
    let h = TestHarness::new("factory-reset").await;
    let store = h.object_store();

    // Install and save a package
    let nmp = build_spear_math_package().unwrap();
    let outcome = install_package(&nmp, "pkg-001", &h.trust_anchors, &store, 0);
    let pkg = match outcome {
        InstallOutcome::Installed { package } => package,
        _ => panic!("install failed"),
    };
    repo::package::save(&h.db.core, &pkg).await.unwrap();

    // Factory reset
    storage_wipe::factory_reset(&h.db.facade, &h.db.core).await.unwrap();

    // DB is empty
    let pkgs = repo::package::list(&h.db.core).await.unwrap();
    assert!(pkgs.is_empty(), "all packages should be gone after factory reset");
}

// ── Key slot wipe ─────────────────────────────────────────────────────────────

#[test]
fn e2e_key_wipe_destroys_all_slots() {
    use taktakk_security::key_slot::{CryptoKeySlot, KeyPurpose, KeyStatus};

    let mut slots = vec![
        CryptoKeySlot {
            key_id: "k1".to_string(),
            purpose: KeyPurpose::State,
            wrapped_key: vec![0xAA; 32],
            alg: "xchacha20poly1305".to_string(),
            created_at: 0,
            rotated_at: None,
            status: KeyStatus::Active,
        },
        CryptoKeySlot {
            key_id: "k2".to_string(),
            purpose: KeyPurpose::Catalog,
            wrapped_key: vec![0xBB; 32],
            alg: "xchacha20poly1305".to_string(),
            created_at: 0,
            rotated_at: None,
            status: KeyStatus::Active,
        },
    ];

    let destroyed = overwrite_all_keys(&mut slots);
    assert_eq!(destroyed, 2);
    for slot in &slots {
        assert_eq!(slot.status, KeyStatus::Destroyed);
        // Wrapped key must differ from the original fill pattern
        assert_ne!(slot.wrapped_key, vec![0xAAu8; 32]);
        assert_ne!(slot.wrapped_key, vec![0xBBu8; 32]);
    }
}

// ── Health check integration ──────────────────────────────────────────────────

#[tokio::test]
async fn e2e_health_check_after_install() {
    let h = TestHarness::new("health").await;
    let store = h.object_store();

    // Install a package and save trust anchor count
    let nmp = build_shield_water_package().unwrap();
    let outcome = install_package(&nmp, "pkg-001", &h.trust_anchors, &store, 0);
    let pkg = match outcome {
        InstallOutcome::Installed { package } => package,
        _ => panic!("install failed"),
    };
    repo::package::save(&h.db.core, &pkg).await.unwrap();

    let pkgs = repo::package::list(&h.db.core).await.unwrap();
    let report = run_static_health_checks(
        pkgs.len(),
        h.trust_anchors.len(),
        2, // two locale packs (en + ar)
        Some(500 * 1024 * 1024),
        0,
    );

    assert!(report.is_healthy(), "health check failed: {}", report.summary());
}

// ── i18n + RTL verification ───────────────────────────────────────────────────

#[test]
fn e2e_rtl_bundle_resolves_arabic_keys() {
    use taktakk_i18n::{fixtures::fixture_bundle, locale::LocaleTag};

    let bundle = fixture_bundle();
    let ar = LocaleTag::new("ar");
    let ar_eg = LocaleTag::new("ar-EG");

    // Direct lookup
    assert!(bundle.get(&ar, "nav.back").is_some());
    // Fallback from ar-EG to ar
    assert_eq!(
        bundle.get(&ar_eg, "nav.back"),
        bundle.get(&ar, "nav.back"),
    );
}

#[test]
fn e2e_ltr_bundle_resolves_swahili_with_english_fallback() {
    use taktakk_i18n::{fixtures::fixture_bundle, locale::LocaleTag};

    let bundle = fixture_bundle();
    let sw = LocaleTag::new("sw");

    assert_eq!(bundle.t(&sw, "nav.next"), "Mbele");
    // Key only in English — falls back correctly
    assert_eq!(bundle.t(&sw, "settings.contrast"), "High Contrast");
}
