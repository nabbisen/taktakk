//! Resume learning use case.

use crate::domain::progress::{LessonProgressStatus, ResumeState};
use crate::error::CoreResult;
use crate::ports::storage::ProgressRepository;

/// Resolved resume point for a lesson.
pub struct ResumePoint {
    pub lesson_id: String,
    /// Zero-indexed step order to render next.
    pub next_step_order: u32,
}

/// Determine where a learner should resume in a lesson.
///
/// If no prior progress exists, returns step order 0.
pub fn resolve_resume_point(
    progress_repo: &dyn ProgressRepository,
    profile_id: &str,
    lesson_id: &str,
) -> CoreResult<ResumePoint> {
    let resume_state = progress_repo.get_resume_state(profile_id, lesson_id)?;

    let next_step_order = match resume_state {
        Some(ResumeState { last_completed_step_order, .. }) => {
            last_completed_step_order + 1
        }
        None => 0,
    };

    Ok(ResumePoint {
        lesson_id: lesson_id.to_string(),
        next_step_order,
    })
}

/// Mark a step as completed and persist the resume point.
pub fn complete_step(
    progress_repo: &dyn ProgressRepository,
    profile_id: &str,
    lesson_id: &str,
    completed_step_order: u32,
    now: i64,
) -> CoreResult<()> {
    let state = ResumeState {
        profile_id: profile_id.to_string(),
        lesson_id: lesson_id.to_string(),
        last_completed_step_order: completed_step_order,
        updated_at: now,
    };
    progress_repo.save_resume_state(&state)
}

/// Mark a lesson as fully completed.
pub fn complete_lesson(
    progress_repo: &dyn ProgressRepository,
    profile_id: &str,
    lesson_id: &str,
    steps_total: u32,
    started_at: i64,
    now: i64,
) -> CoreResult<()> {
    let progress = crate::domain::progress::LessonProgress {
        profile_id: profile_id.to_string(),
        lesson_id: lesson_id.to_string(),
        status: LessonProgressStatus::Completed,
        steps_completed: steps_total,
        steps_total,
        started_at,
        completed_at: Some(now),
    };
    progress_repo.save_lesson_progress(&progress)
}
