//! Curriculum domain model: categories, modules, lessons, and steps.
//!
//! The curriculum is organized around two axes:
//! - **Shield**: survival knowledge (hygiene, first aid, safety routes)
//! - **Spear**: empowerment knowledge (math, communication, digital literacy)

use serde::{Deserialize, Serialize};

/// Top-level curriculum category.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurriculumAxis {
    /// Survival-focused modules: protect life and safety.
    Shield,
    /// Empowerment-focused modules: build capability and agency.
    Spear,
}

/// A thematic grouping of modules within an axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCategory {
    pub category_id: String,
    pub axis: CurriculumAxis,
    /// Localization key for the display name.
    pub name_key: String,
    pub sort_order: u32,
}

/// A learning module: the top-level unit of educational content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub module_id: String,
    pub category_id: String,
    /// Localization key for the title.
    pub title_key: String,
    /// Localization key for the short description.
    pub description_key: String,
    pub version: ModuleVersion,
    pub status: ModuleStatus,
    /// Estimated completion time in minutes.
    pub estimated_minutes: Option<u16>,
}

/// Semantic version for a module.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ModuleVersion {
    pub major: u16,
    pub minor: u16,
    pub patch: u16,
}

impl ModuleVersion {
    pub fn new(major: u16, minor: u16, patch: u16) -> Self {
        Self { major, minor, patch }
    }
}

impl std::fmt::Display for ModuleVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

/// Installation status of a module.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleStatus {
    /// Fully installed and verified.
    Available,
    /// Partially downloaded; cannot be used yet.
    Partial,
    /// Signature or hash check failed; quarantined.
    Quarantined,
    /// Soft-deleted; data may still exist on disk.
    Disabled,
}

/// A lesson within a module: one focused learning unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub lesson_id: String,
    pub module_id: String,
    pub title_key: String,
    pub sort_order: u32,
    pub step_count: u32,
}

/// An individual step within a lesson.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LessonStep {
    pub step_id: String,
    pub lesson_id: String,
    pub sort_order: u32,
    pub content_type: StepContentType,
    /// Reference to the content object (by hash in the object store).
    pub content_object_hash: Option<String>,
    /// Inline text content (localization key or direct text for simple steps).
    pub text_key: Option<String>,
}

/// The kind of content presented in a step.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepContentType {
    Text,
    Svg,
    Audio,
    /// Interactive Wasm module.
    Wasm,
    /// Multiple-choice or drill exercise.
    Exercise,
}
