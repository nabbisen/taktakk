//! Storage port: abstract repository traits.

use crate::domain::{
    curriculum::{Lesson, Module},
    package::ContentPackage,
    profile::LocalProfile,
    progress::{ExerciseAttempt, LearningSession, LessonProgress, ResumeState},
    sync::SyncSession,
};
use crate::error::CoreResult;

/// Repository for curriculum metadata.
pub trait CurriculumRepository: Send + Sync {
    fn list_modules(&self) -> CoreResult<Vec<Module>>;
    fn get_module(&self, module_id: &str) -> CoreResult<Option<Module>>;
    fn list_lessons(&self, module_id: &str) -> CoreResult<Vec<Lesson>>;
    fn get_lesson(&self, lesson_id: &str) -> CoreResult<Option<Lesson>>;
}

/// Repository for installed content packages.
pub trait PackageRepository: Send + Sync {
    fn save_package(&self, pkg: &ContentPackage) -> CoreResult<()>;
    fn get_package(&self, package_id: &str) -> CoreResult<Option<ContentPackage>>;
    fn list_packages(&self) -> CoreResult<Vec<ContentPackage>>;
    fn update_status(
        &self,
        package_id: &str,
        status: crate::domain::package::PackageStatus,
    ) -> CoreResult<()>;
}

/// Repository for learner profiles.
pub trait ProfileRepository: Send + Sync {
    fn save_profile(&self, profile: &LocalProfile) -> CoreResult<()>;
    fn get_profile(&self, profile_id: &str) -> CoreResult<Option<LocalProfile>>;
    fn get_active_profile(&self) -> CoreResult<Option<LocalProfile>>;
}

/// Repository for learning progress and resume state.
pub trait ProgressRepository: Send + Sync {
    fn save_resume_state(&self, state: &ResumeState) -> CoreResult<()>;
    fn get_resume_state(&self, profile_id: &str, lesson_id: &str)
        -> CoreResult<Option<ResumeState>>;
    fn save_lesson_progress(&self, progress: &LessonProgress) -> CoreResult<()>;
    fn get_lesson_progress(
        &self,
        profile_id: &str,
        lesson_id: &str,
    ) -> CoreResult<Option<LessonProgress>>;
    fn save_exercise_attempt(&self, attempt: &ExerciseAttempt) -> CoreResult<()>;
    fn save_session(&self, session: &LearningSession) -> CoreResult<()>;
    fn end_session(&self, session_id: &str, ended_at: i64) -> CoreResult<()>;
}

/// Repository for sync session history.
pub trait SyncRepository: Send + Sync {
    fn save_sync_session(&self, session: &SyncSession) -> CoreResult<()>;
    fn get_sync_session(&self, session_id: &str) -> CoreResult<Option<SyncSession>>;
}
