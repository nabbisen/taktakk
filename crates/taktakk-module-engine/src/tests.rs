//! Unit tests for taktakk-module-engine (M4).

use crate::catalog::{build_tile, DashboardView, ModuleTile, ProgressBadge};
use crate::error::EngineError;
use crate::exercise::{evaluate, max_attempts, EvalResult, ExerciseAnswer};
use crate::runner::{LessonRunner, RunnerEvent, RunnerResponse};
use crate::state::{LessonState, LessonStateStatus};
use crate::step::{ExerciseKind, ExerciseSpec, StepContent, StepKind};
use taktakk_core::domain::curriculum::{CurriculumAxis, Module, ModuleStatus, ModuleVersion};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn text_step(id: &str, order: u32) -> StepContent {
    StepContent {
        step_id: id.to_string(),
        sort_order: order,
        kind: StepKind::Text { text_key: format!("step.{id}.text") },
        caption_key: None,
        audio_object_hash: None,
        aria_label_key: None,
    }
}

fn mc_step(id: &str, order: u32, correct: &str, opts: &[&str]) -> StepContent {
    StepContent {
        step_id: id.to_string(),
        sort_order: order,
        kind: StepKind::Exercise(ExerciseSpec {
            exercise_id: format!("ex-{id}"),
            kind: ExerciseKind::MultipleChoice {
                question_key: "q".to_string(),
                options: opts.iter().map(|o| (o.to_string(), o.to_string())).collect(),
                correct_option_id: correct.to_string(),
            },
        }),
        caption_key: None,
        audio_object_hash: None,
        aria_label_key: None,
    }
}

fn ack_step(id: &str, order: u32) -> StepContent {
    StepContent {
        step_id: id.to_string(),
        sort_order: order,
        kind: StepKind::Exercise(ExerciseSpec {
            exercise_id: format!("ack-{id}"),
            kind: ExerciseKind::Acknowledge { confirm_key: "ok".to_string() },
        }),
        caption_key: None,
        audio_object_hash: None,
        aria_label_key: None,
    }
}

fn make_runner(steps: Vec<StepContent>, now: i64) -> LessonRunner {
    let total = steps.len() as u32;
    let state = LessonState::new("p1".to_string(), "lesson-01".to_string(), total, now);
    LessonRunner::new(state, steps)
}

// ── LessonState ───────────────────────────────────────────────────────────────

#[test]
fn state_initial_next_step_is_zero() {
    let s = LessonState::new("p1".to_string(), "l1".to_string(), 5, 0);
    assert_eq!(s.next_step_order(), 0);
    assert_eq!(s.completed_count(), 0);
    assert!((s.progress_fraction()).abs() < f32::EPSILON);
}

#[test]
fn state_complete_step_advances() {
    let mut s = LessonState::new("p1".to_string(), "l1".to_string(), 5, 0);
    let last = s.complete_step(0);
    assert!(!last);
    assert_eq!(s.next_step_order(), 1);
    assert_eq!(s.completed_count(), 1);
}

#[test]
fn state_complete_last_step_marks_done() {
    let mut s = LessonState::new("p1".to_string(), "l1".to_string(), 3, 0);
    s.complete_step(0);
    s.complete_step(1);
    let last = s.complete_step(2);
    assert!(last);
    assert_eq!(s.status, LessonStateStatus::Completed);
    assert!((s.progress_fraction() - 1.0).abs() < f32::EPSILON);
}

#[test]
fn state_progress_fraction_midway() {
    let mut s = LessonState::new("p1".to_string(), "l1".to_string(), 4, 0);
    s.complete_step(0);
    s.complete_step(1);
    let expected = 2.0 / 4.0;
    assert!((s.progress_fraction() - expected).abs() < f32::EPSILON);
}

#[test]
fn state_json_round_trip() {
    let mut s = LessonState::new("p1".to_string(), "l1".to_string(), 3, 1234);
    s.complete_step(1);
    let json = s.to_json().unwrap();
    let restored = LessonState::from_json(&json).unwrap();
    assert_eq!(s, restored);
}

// ── Exercise evaluation ───────────────────────────────────────────────────────

fn mc_spec(correct: &str, options: &[&str]) -> ExerciseSpec {
    ExerciseSpec {
        exercise_id: "ex-1".to_string(),
        kind: ExerciseKind::MultipleChoice {
            question_key: "q".to_string(),
            options: options.iter().map(|o| (o.to_string(), o.to_string())).collect(),
            correct_option_id: correct.to_string(),
        },
    }
}

#[test]
fn exercise_correct_multiple_choice() {
    let spec = mc_spec("B", &["A", "B", "C"]);
    let ans = ExerciseAnswer::MultipleChoice { chosen_option_id: "B".to_string() };
    assert_eq!(evaluate(&spec, &ans).unwrap(), EvalResult::Correct);
}

#[test]
fn exercise_incorrect_multiple_choice() {
    let spec = mc_spec("B", &["A", "B", "C"]);
    let ans = ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() };
    assert_eq!(evaluate(&spec, &ans).unwrap(), EvalResult::Incorrect);
}

#[test]
fn exercise_ordering_correct() {
    let spec = ExerciseSpec {
        exercise_id: "ord".to_string(),
        kind: ExerciseKind::Ordering {
            prompt_key: "p".to_string(),
            items: vec![("A".to_string(), "a".to_string()), ("B".to_string(), "b".to_string())],
            correct_order: vec!["A".to_string(), "B".to_string()],
        },
    };
    let ans = ExerciseAnswer::Ordering {
        arranged_ids: vec!["A".to_string(), "B".to_string()],
    };
    assert_eq!(evaluate(&spec, &ans).unwrap(), EvalResult::Correct);
}

#[test]
fn exercise_ordering_wrong_order() {
    let spec = ExerciseSpec {
        exercise_id: "ord".to_string(),
        kind: ExerciseKind::Ordering {
            prompt_key: "p".to_string(),
            items: vec![("A".to_string(), "a".to_string()), ("B".to_string(), "b".to_string())],
            correct_order: vec!["A".to_string(), "B".to_string()],
        },
    };
    let ans = ExerciseAnswer::Ordering {
        arranged_ids: vec!["B".to_string(), "A".to_string()],
    };
    assert_eq!(evaluate(&spec, &ans).unwrap(), EvalResult::Incorrect);
}

#[test]
fn exercise_acknowledge_always_correct() {
    let spec = ExerciseSpec {
        exercise_id: "ack".to_string(),
        kind: ExerciseKind::Acknowledge { confirm_key: "ok".to_string() },
    };
    assert_eq!(evaluate(&spec, &ExerciseAnswer::Acknowledge).unwrap(), EvalResult::Correct);
}

#[test]
fn exercise_type_mismatch_returns_error() {
    let spec = mc_spec("A", &["A", "B"]);
    let ans = ExerciseAnswer::Acknowledge;
    assert!(evaluate(&spec, &ans).is_err());
}

#[test]
fn max_attempts_mc_is_three() {
    let spec = mc_spec("A", &["A", "B"]);
    assert_eq!(max_attempts(&spec), Some(3));
}

// ── LessonRunner ──────────────────────────────────────────────────────────────

#[test]
fn runner_advance_text_steps_sequentially() {
    let mut r = make_runner(vec![text_step("s0", 0), text_step("s1", 1), text_step("s2", 2)], 0);
    let r1 = r.handle(RunnerEvent::Advance).unwrap();
    assert_eq!(r1, RunnerResponse::StepAdvanced { new_order: 1 });
    let r2 = r.handle(RunnerEvent::Advance).unwrap();
    assert_eq!(r2, RunnerResponse::StepAdvanced { new_order: 2 });
    let r3 = r.handle(RunnerEvent::Advance).unwrap();
    assert_eq!(r3, RunnerResponse::LessonComplete);
}

#[test]
fn runner_single_step_lesson_completes_immediately() {
    let mut r = make_runner(vec![text_step("s0", 0)], 0);
    assert_eq!(r.handle(RunnerEvent::Advance).unwrap(), RunnerResponse::LessonComplete);
}

#[test]
fn runner_correct_answer_advances() {
    let mut r = make_runner(vec![mc_step("s0", 0, "B", &["A", "B", "C"])], 0);
    let resp = r.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "B".to_string() },
    )).unwrap();
    assert_eq!(resp, RunnerResponse::LessonComplete);
}

#[test]
fn runner_wrong_answer_does_not_advance() {
    let mut r = make_runner(vec![
        mc_step("s0", 0, "B", &["A", "B", "C"]),
        text_step("s1", 1),
    ], 0);
    let resp = r.handle(RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() },
    )).unwrap();
    assert!(matches!(resp, RunnerResponse::AnswerIncorrect { attempts_used: 1, max_attempts: 3 }));
    // Still on step 0.
    assert_eq!(r.current_order(), 0);
}

#[test]
fn runner_max_attempts_exhausted_auto_advances() {
    let mut r = make_runner(vec![mc_step("s0", 0, "B", &["A", "B"]), text_step("s1", 1)], 0);
    let wrong = || RunnerEvent::Answer(
        ExerciseAnswer::MultipleChoice { chosen_option_id: "A".to_string() },
    );
    r.handle(wrong()).unwrap(); // attempt 1
    r.handle(wrong()).unwrap(); // attempt 2
    let resp = r.handle(wrong()).unwrap(); // attempt 3 — max
    assert!(matches!(resp, RunnerResponse::MaxAttemptsReached { new_order: 1 }));
}

#[test]
fn runner_back_at_first_step_returns_at_first() {
    let mut r = make_runner(vec![text_step("s0", 0)], 0);
    assert_eq!(r.handle(RunnerEvent::Back).unwrap(), RunnerResponse::AtFirstStep);
}

#[test]
fn runner_back_after_first_advance_returns_to_step_zero() {
    let mut r = make_runner(vec![text_step("s0", 0), text_step("s1", 1)], 0);
    r.handle(RunnerEvent::Advance).unwrap();
    let resp = r.handle(RunnerEvent::Back).unwrap();
    assert!(matches!(resp, RunnerResponse::StepBack { new_order: 0 }));
}

#[test]
fn runner_completed_lesson_returns_lesson_complete() {
    let mut r = make_runner(vec![text_step("s0", 0)], 0);
    r.handle(RunnerEvent::Advance).unwrap();
    assert_eq!(r.handle(RunnerEvent::Advance).unwrap(), RunnerResponse::LessonComplete);
}

#[test]
fn runner_acknowledge_step_completes_on_answer() {
    let mut r = make_runner(vec![ack_step("a0", 0), text_step("s1", 1)], 0);
    let resp = r.handle(RunnerEvent::Answer(ExerciseAnswer::Acknowledge)).unwrap();
    assert!(matches!(resp, RunnerResponse::AnswerCorrect { new_order: 1 }));
}

#[test]
fn runner_advance_on_exercise_step_returns_error() {
    let mut r = make_runner(vec![mc_step("s0", 0, "A", &["A", "B"])], 0);
    assert!(r.handle(RunnerEvent::Advance).is_err());
}

#[test]
fn runner_restore_from_mid_lesson_state() {
    // Simulate restoring a runner after step 1 completed.
    let steps = vec![text_step("s0", 0), text_step("s1", 1), text_step("s2", 2)];
    let mut state = LessonState::new("p1".to_string(), "l1".to_string(), 3, 0);
    state.complete_step(0);
    state.complete_step(1);
    let mut r = LessonRunner::new(state, steps);
    // Should be at step 2.
    assert_eq!(r.current_order(), 2);
    assert_eq!(r.handle(RunnerEvent::Advance).unwrap(), RunnerResponse::LessonComplete);
}

// ── Dashboard catalog ─────────────────────────────────────────────────────────

fn make_module(id: &str, cat: &str) -> Module {
    Module {
        module_id: id.to_string(),
        category_id: cat.to_string(),
        title_key: format!("{id}.title"),
        description_key: format!("{id}.desc"),
        version: ModuleVersion::new(1, 0, 0),
        status: ModuleStatus::Available,
        estimated_minutes: Some(15),
    }
}

#[test]
fn build_tile_not_started() {
    let m = make_module("shield-water", "shield-hygiene");
    let tile = build_tile(&m, 0, 10);
    assert_eq!(tile.progress, ProgressBadge::NotStarted);
    assert_eq!(tile.axis, CurriculumAxis::Shield);
}

#[test]
fn build_tile_in_progress() {
    let m = make_module("spear-math", "spear-logic");
    let tile = build_tile(&m, 3, 10);
    assert_eq!(tile.progress, ProgressBadge::InProgress { completed: 3, total: 10 });
    assert_eq!(tile.axis, CurriculumAxis::Spear);
}

#[test]
fn build_tile_completed() {
    let m = make_module("shield-first-aid", "shield-medical");
    let tile = build_tile(&m, 10, 10);
    assert_eq!(tile.progress, ProgressBadge::Completed);
}

#[test]
fn build_tile_quarantined_is_unavailable() {
    let mut m = make_module("shield-nav", "shield-safety");
    m.status = ModuleStatus::Quarantined;
    let tile = build_tile(&m, 0, 5);
    assert_eq!(tile.progress, ProgressBadge::Unavailable);
}

#[test]
fn progress_badge_fraction() {
    assert!((ProgressBadge::NotStarted.fraction()).abs() < f32::EPSILON);
    assert!((ProgressBadge::Completed.fraction() - 1.0).abs() < f32::EPSILON);
    let ip = ProgressBadge::InProgress { completed: 2, total: 4 };
    assert!((ip.fraction() - 0.5).abs() < f32::EPSILON);
}

#[test]
fn dashboard_completed_count() {
    let shield = vec![
        ModuleTile { module_id: "a".to_string(), axis: CurriculumAxis::Shield,
            title_key: "t".to_string(), description_key: "d".to_string(),
            progress: ProgressBadge::Completed, estimated_minutes: None },
        ModuleTile { module_id: "b".to_string(), axis: CurriculumAxis::Shield,
            title_key: "t".to_string(), description_key: "d".to_string(),
            progress: ProgressBadge::NotStarted, estimated_minutes: None },
    ];
    let spear = vec![
        ModuleTile { module_id: "c".to_string(), axis: CurriculumAxis::Spear,
            title_key: "t".to_string(), description_key: "d".to_string(),
            progress: ProgressBadge::Completed, estimated_minutes: None },
    ];
    let dash = DashboardView::new(shield, spear);
    assert_eq!(dash.completed_count(), 2);
}
