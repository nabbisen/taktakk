//! Serializable lesson state for crash-safe resume.
//!
//! The state is written after every completed step (not during input handling),
//! so power loss within a step costs at most one step's progress.

use serde::{Deserialize, Serialize};

/// Complete serialized state of an in-progress lesson session.
///
/// Written to `resume_state` in `core.sqlite` after each step completion.
/// On relaunch, this is deserialized to reconstruct the runner's position.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LessonState {
    pub profile_id: String,
    pub lesson_id: String,
    /// Zero-indexed order of the last completed step.
    /// `None` means the lesson has been started but no step is finished yet.
    pub last_completed_step_order: Option<u32>,
    pub total_steps: u32,
    pub status: LessonStateStatus,
    /// Unix timestamp (seconds) when the lesson was first opened.
    pub started_at: i64,
}

/// Coarse status reflected from the state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LessonStateStatus {
    InProgress,
    Completed,
}

impl LessonState {
    /// Create initial state when a lesson is first opened.
    pub fn new(profile_id: String, lesson_id: String, total_steps: u32, now: i64) -> Self {
        Self {
            profile_id,
            lesson_id,
            last_completed_step_order: None,
            total_steps,
            status: LessonStateStatus::InProgress,
            started_at: now,
        }
    }

    /// The zero-indexed step order to present next.
    pub fn next_step_order(&self) -> u32 {
        match self.last_completed_step_order {
            None => 0,
            Some(n) => n + 1,
        }
    }

    /// Mark the step at `order` as completed. Returns `true` if this was the
    /// final step.
    pub fn complete_step(&mut self, order: u32) -> bool {
        self.last_completed_step_order = Some(order);
        let is_last = order + 1 >= self.total_steps;
        if is_last {
            self.status = LessonStateStatus::Completed;
        }
        is_last
    }

    /// Progress as a fraction 0.0..=1.0.
    pub fn progress_fraction(&self) -> f32 {
        if self.total_steps == 0 {
            return 1.0;
        }
        match self.last_completed_step_order {
            None => 0.0,
            Some(n) => (n + 1) as f32 / self.total_steps as f32,
        }
    }

    /// Number of completed steps.
    pub fn completed_count(&self) -> u32 {
        self.last_completed_step_order.map(|n| n + 1).unwrap_or(0)
    }

    /// Serialize to JSON bytes for storage.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize from JSON bytes.
    pub fn from_json(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}
