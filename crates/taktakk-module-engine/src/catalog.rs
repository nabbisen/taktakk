//! Dashboard catalog model.
//!
//! The dashboard shows two pillars (Shield / Spear) with module tiles.
//! Each tile carries a progress badge and a completion indicator.
//! No network calls; all data comes from local storage.

use serde::{Deserialize, Serialize};
use taktakk_core::domain::curriculum::{CurriculumAxis, Module, ModuleStatus};

/// Progress badge on a module tile.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressBadge {
    /// Module is available but the learner has not started it.
    NotStarted,
    /// Learning is in progress; carries completed/total step counts.
    InProgress { completed: u32, total: u32 },
    /// All lessons completed.
    Completed,
    /// Package was quarantined or is otherwise unavailable.
    Unavailable,
}

/// A single tile in the module catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleTile {
    pub module_id: String,
    pub axis: CurriculumAxis,
    /// Localization key for the title.
    pub title_key: String,
    /// Localization key for the short description.
    pub description_key: String,
    pub progress: ProgressBadge,
    /// Estimated minutes to complete (for UI display).
    pub estimated_minutes: Option<u16>,
}

impl ProgressBadge {
    /// Progress as 0.0..=1.0, used for the segmented progress bar.
    pub fn fraction(&self) -> f32 {
        match self {
            Self::NotStarted => 0.0,
            Self::Completed => 1.0,
            Self::Unavailable => 0.0,
            Self::InProgress { completed, total } => {
                if *total == 0 { 0.0 } else { *completed as f32 / *total as f32 }
            }
        }
    }
}

/// The two-pillar dashboard view model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardView {
    pub shield_tiles: Vec<ModuleTile>,
    pub spear_tiles:  Vec<ModuleTile>,
}

impl DashboardView {
    pub fn new(shield: Vec<ModuleTile>, spear: Vec<ModuleTile>) -> Self {
        Self { shield_tiles: shield, spear_tiles: spear }
    }

    /// All tiles across both axes.
    pub fn all_tiles(&self) -> impl Iterator<Item = &ModuleTile> {
        self.shield_tiles.iter().chain(self.spear_tiles.iter())
    }

    /// Find a tile by module_id.
    pub fn tile(&self, module_id: &str) -> Option<&ModuleTile> {
        self.all_tiles().find(|t| t.module_id == module_id)
    }

    /// Count of completed modules across both axes.
    pub fn completed_count(&self) -> usize {
        self.all_tiles()
            .filter(|t| t.progress == ProgressBadge::Completed)
            .count()
    }
}

/// Build a `ModuleTile` from a module record and its aggregate progress.
pub fn build_tile(
    module: &Module,
    completed_steps: u32,
    total_steps: u32,
) -> ModuleTile {
    let progress = match module.status {
        ModuleStatus::Quarantined | ModuleStatus::Disabled => ProgressBadge::Unavailable,
        _ => {
            if completed_steps == 0 {
                ProgressBadge::NotStarted
            } else if completed_steps >= total_steps && total_steps > 0 {
                ProgressBadge::Completed
            } else {
                ProgressBadge::InProgress {
                    completed: completed_steps,
                    total: total_steps,
                }
            }
        }
    };

    // Look up the category axis via the module's category_id.
    // For now, we resolve by convention: "shield-*" → Shield, "spear-*" → Spear.
    let axis = if module.category_id.starts_with("shield") {
        CurriculumAxis::Shield
    } else {
        CurriculumAxis::Spear
    };

    ModuleTile {
        module_id: module.module_id.clone(),
        axis,
        title_key: module.title_key.clone(),
        description_key: module.description_key.clone(),
        progress,
        estimated_minutes: module.estimated_minutes,
    }
}
