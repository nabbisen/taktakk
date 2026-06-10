//! Open module use case.

use crate::domain::curriculum::Module;
use crate::error::{CoreError, CoreResult};
use crate::ports::storage::CurriculumRepository;

/// Request to open a module for learning.
pub struct OpenModuleRequest {
    pub module_id: String,
    pub profile_id: String,
}

/// Result of opening a module.
pub struct OpenModuleResult {
    pub module: Module,
    /// ID of the lesson to resume or start (first lesson if no prior progress).
    pub resume_lesson_id: String,
    /// Step order to resume from.
    pub resume_step_order: u32,
}

/// Open a module, resolving the resume point for the given profile.
pub fn open_module(
    curriculum_repo: &dyn CurriculumRepository,
    req: OpenModuleRequest,
) -> CoreResult<OpenModuleResult> {
    let module = curriculum_repo
        .get_module(&req.module_id)?
        .ok_or_else(|| CoreError::ModuleNotFound { id: req.module_id.clone() })?;

    let lessons = curriculum_repo.list_lessons(&req.module_id)?;
    if lessons.is_empty() {
        return Err(CoreError::ModuleNotFound { id: req.module_id });
    }

    let first_lesson = lessons
        .iter()
        .min_by_key(|l| l.sort_order)
        .expect("lessons is non-empty");

    Ok(OpenModuleResult {
        module,
        resume_lesson_id: first_lesson.lesson_id.clone(),
        resume_step_order: 0,
    })
}
