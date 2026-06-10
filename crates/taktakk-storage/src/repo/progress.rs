//! Learning progress repository: resume state, lesson progress, sessions.

use sqlx::SqlitePool;

use taktakk_core::domain::progress::{
    ExerciseAttempt, LearningSession, LessonProgress, LessonProgressStatus, ResumeState,
};
use taktakk_core::error::{CoreError, CoreResult};

// ── Resume state ─────────────────────────────────────────────────────────────

pub async fn save_resume_state(pool: &SqlitePool, s: &ResumeState) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO resume_state
             (profile_id, lesson_id, last_completed_step_order, updated_at)
         VALUES (?, ?, ?, ?)
         ON CONFLICT(profile_id, lesson_id) DO UPDATE SET
             last_completed_step_order = excluded.last_completed_step_order,
             updated_at                = excluded.updated_at",
    )
    .bind(&s.profile_id)
    .bind(&s.lesson_id)
    .bind(s.last_completed_step_order)
    .bind(s.updated_at)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn get_resume_state(
    pool: &SqlitePool,
    profile_id: &str,
    lesson_id: &str,
) -> CoreResult<Option<ResumeState>> {
    let row = sqlx::query_as::<_, (String, String, i64, i64)>(
        "SELECT profile_id, lesson_id, last_completed_step_order, updated_at
         FROM resume_state
         WHERE profile_id = ? AND lesson_id = ?",
    )
    .bind(profile_id)
    .bind(lesson_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(|(profile_id, lesson_id, last_completed_step_order, updated_at)| ResumeState {
        profile_id,
        lesson_id,
        last_completed_step_order: last_completed_step_order as u32,
        updated_at,
    }))
}

/// Delete all resume state — used during state wipe.
pub async fn wipe_resume_state(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query("DELETE FROM resume_state")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

// ── Lesson progress ───────────────────────────────────────────────────────────

pub async fn save_lesson_progress(pool: &SqlitePool, p: &LessonProgress) -> CoreResult<()> {
    let status_str = match p.status {
        LessonProgressStatus::NotStarted => "not_started",
        LessonProgressStatus::InProgress => "in_progress",
        LessonProgressStatus::Completed => "completed",
    };
    sqlx::query(
        "INSERT INTO lesson_progress
             (profile_id, lesson_id, status, steps_completed, steps_total,
              started_at, completed_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(profile_id, lesson_id) DO UPDATE SET
             status          = excluded.status,
             steps_completed = excluded.steps_completed,
             steps_total     = excluded.steps_total,
             completed_at    = excluded.completed_at",
    )
    .bind(&p.profile_id)
    .bind(&p.lesson_id)
    .bind(status_str)
    .bind(p.steps_completed)
    .bind(p.steps_total)
    .bind(p.started_at)
    .bind(p.completed_at)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn get_lesson_progress(
    pool: &SqlitePool,
    profile_id: &str,
    lesson_id: &str,
) -> CoreResult<Option<LessonProgress>> {
    let row = sqlx::query_as::<_, (String, String, String, i64, i64, i64, Option<i64>)>(
        "SELECT profile_id, lesson_id, status, steps_completed, steps_total,
                started_at, completed_at
         FROM lesson_progress
         WHERE profile_id = ? AND lesson_id = ?",
    )
    .bind(profile_id)
    .bind(lesson_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;

    Ok(row.map(
        |(profile_id, lesson_id, status_str, steps_completed, steps_total, started_at, completed_at)| {
            let status = match status_str.as_str() {
                "completed"  => LessonProgressStatus::Completed,
                "in_progress" => LessonProgressStatus::InProgress,
                _ => LessonProgressStatus::NotStarted,
            };
            LessonProgress {
                profile_id,
                lesson_id,
                status,
                steps_completed: steps_completed as u32,
                steps_total: steps_total as u32,
                started_at,
                completed_at,
            }
        },
    ))
}

/// Delete all lesson progress — used during state wipe.
pub async fn wipe_lesson_progress(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query("DELETE FROM lesson_progress")
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

// ── Learning sessions ─────────────────────────────────────────────────────────

pub async fn save_session(pool: &SqlitePool, s: &LearningSession) -> CoreResult<()> {
    sqlx::query(
        "INSERT OR IGNORE INTO learning_sessions
             (session_id, profile_id, started_at, ended_at)
         VALUES (?, ?, ?, ?)",
    )
    .bind(&s.session_id)
    .bind(&s.profile_id)
    .bind(s.started_at)
    .bind(s.ended_at)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

pub async fn end_session(pool: &SqlitePool, session_id: &str, ended_at: i64) -> CoreResult<()> {
    sqlx::query("UPDATE learning_sessions SET ended_at = ? WHERE session_id = ?")
        .bind(ended_at)
        .bind(session_id)
        .execute(pool)
        .await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

/// Save an exercise attempt.
pub async fn save_exercise_attempt(pool: &SqlitePool, a: &ExerciseAttempt) -> CoreResult<()> {
    sqlx::query(
        "INSERT INTO exercise_attempts
             (attempt_id, profile_id, step_id, correct, attempt_number, attempted_at)
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&a.attempt_id)
    .bind(&a.profile_id)
    .bind(&a.step_id)
    .bind(a.correct)
    .bind(a.attempt_number)
    .bind(a.attempted_at)
    .execute(pool)
    .await
    .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}

/// Delete all sessions and attempts — used during state wipe.
pub async fn wipe_sessions(pool: &SqlitePool) -> CoreResult<()> {
    sqlx::query("DELETE FROM exercise_attempts").execute(pool).await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    sqlx::query("DELETE FROM learning_sessions").execute(pool).await
        .map_err(|e| CoreError::Storage(e.to_string()))?;
    Ok(())
}
