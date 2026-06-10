//! Learning progress and session state.
//!
//! Progress is tracked at the step level to allow recovery after power loss.
//! No personally identifiable information is stored here.

use serde::{Deserialize, Serialize};

/// The persisted "resume point" for a learner within a lesson.
///
/// Written after each completed step so that power loss only loses at most
/// the current in-progress step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResumeState {
    pub profile_id: String,
    pub lesson_id: String,
    pub last_completed_step_order: u32,
    pub updated_at: i64,
}

/// Progress summary for one lesson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonProgress {
    pub profile_id: String,
    pub lesson_id: String,
    pub status: LessonProgressStatus,
    pub steps_completed: u32,
    pub steps_total: u32,
    pub started_at: i64,
    pub completed_at: Option<i64>,
}

/// Coarse status of lesson completion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LessonProgressStatus {
    NotStarted,
    InProgress,
    Completed,
}

/// Result of a single exercise or drill attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseAttempt {
    pub attempt_id: String,
    pub profile_id: String,
    pub step_id: String,
    /// Whether the attempt was scored as correct.
    pub correct: bool,
    /// Number of attempts so far for this step.
    pub attempt_number: u32,
    pub attempted_at: i64,
}

/// A short-lived active learning session.
///
/// Sessions begin at unlock and end at re-lock, app close, or timeout.
/// They are used for aggregate statistics only; no lesson content is logged.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    pub session_id: String,
    pub profile_id: String,
    pub started_at: i64,
    pub ended_at: Option<i64>,
}
