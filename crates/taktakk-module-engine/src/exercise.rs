//! Exercise evaluation: check answers without storing sensitive user responses.
//!
//! Correct/incorrect is stored; the actual answer given by the user is not.

use crate::error::{EngineError, EngineResult};
use crate::step::{ExerciseKind, ExerciseSpec};

/// The learner's response to an exercise step.
#[derive(Debug, Clone)]
pub enum ExerciseAnswer {
    /// The ID of the chosen option.
    MultipleChoice { chosen_option_id: String },
    /// Item IDs in the order the learner arranged them.
    Ordering { arranged_ids: Vec<String> },
    /// Acknowledgement tap.
    Acknowledge,
}

/// Result of evaluating an exercise answer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalResult {
    Correct,
    Incorrect,
    /// For exercises with partial scoring (future extension).
    Partial,
}

/// Evaluate a learner's answer against the exercise specification.
///
/// Intentionally does not store what the learner answered — only whether
/// they were correct.
pub fn evaluate(spec: &ExerciseSpec, answer: &ExerciseAnswer) -> EngineResult<EvalResult> {
    match (&spec.kind, answer) {
        (
            ExerciseKind::MultipleChoice { correct_option_id, .. },
            ExerciseAnswer::MultipleChoice { chosen_option_id },
        ) => {
            if chosen_option_id == correct_option_id {
                Ok(EvalResult::Correct)
            } else {
                Ok(EvalResult::Incorrect)
            }
        }

        (
            ExerciseKind::Ordering { correct_order, .. },
            ExerciseAnswer::Ordering { arranged_ids },
        ) => {
            if arranged_ids == correct_order {
                Ok(EvalResult::Correct)
            } else {
                Ok(EvalResult::Incorrect)
            }
        }

        (ExerciseKind::Acknowledge { .. }, ExerciseAnswer::Acknowledge) => {
            Ok(EvalResult::Correct)
        }

        _ => Err(EngineError::InvalidAnswer),
    }
}

/// Return `true` if the exercise allows an unlimited number of retries.
///
/// Acknowledge steps are always immediately correct. Multiple-choice allows
/// up to 3 attempts before auto-advancing (displayed via UI feedback only).
pub fn max_attempts(spec: &ExerciseSpec) -> Option<u32> {
    match &spec.kind {
        ExerciseKind::Acknowledge { .. } => Some(1),
        ExerciseKind::MultipleChoice { .. } => Some(3),
        ExerciseKind::Ordering { .. } => Some(3),
    }
}
