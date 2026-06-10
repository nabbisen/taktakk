//! Lesson step content types.
//!
//! Each step presents exactly one piece of content plus optional audio.
//! Audio is always replayable; text is secondary to pictogram/SVG.

use serde::{Deserialize, Serialize};

/// The rendered content of one lesson step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepContent {
    pub step_id: String,
    pub sort_order: u32,
    pub kind: StepKind,
    /// Localization key for the caption text (optional; UI may hide it).
    pub caption_key: Option<String>,
    /// SHA-256 hash of the Opus audio file for this step (optional).
    pub audio_object_hash: Option<String>,
    /// ARIA label key for accessibility (required if pictogram/SVG only).
    pub aria_label_key: Option<String>,
}

/// The primary content of a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepKind {
    /// Inline UTF-8 text (localization key resolved by caller).
    Text { text_key: String },
    /// Reference to an SVG in the object store.
    Svg { object_hash: String },
    /// An interactive drill or exercise.
    Exercise(ExerciseSpec),
    /// Reference to a Wasm module (sandboxed execution, M4+).
    Wasm { object_hash: String, initial_state_json: String },
}

/// Specification for a drill exercise embedded in a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExerciseSpec {
    pub exercise_id: String,
    pub kind: ExerciseKind,
}

/// The variety of exercise that can appear in a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExerciseKind {
    /// Choose the single correct option from a list.
    MultipleChoice {
        /// Localization key for the question prompt.
        question_key: String,
        /// Each option: (option_id, label_key).
        options: Vec<(String, String)>,
        correct_option_id: String,
    },
    /// Arrange items into the correct sequence.
    Ordering {
        /// Localization key for the prompt.
        prompt_key: String,
        /// (item_id, label_key) — shuffled for display.
        items: Vec<(String, String)>,
        /// Correct order as a list of item_ids.
        correct_order: Vec<String>,
    },
    /// Free-acknowledge step: tap to confirm you have read/seen the content.
    Acknowledge { confirm_key: String },
}
