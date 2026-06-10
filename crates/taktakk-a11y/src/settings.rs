//! Accessibility settings model.

use serde::{Deserialize, Serialize};

/// Text scale factor relative to the default size.
///
/// 1.0 = default; 1.5 = 50 % larger; 2.0 = 100 % larger.
/// Clamped to [`TEXT_SCALE_MIN`]..=[`TEXT_SCALE_MAX`] on write.
pub const TEXT_SCALE_MIN: f32 = 0.8;
pub const TEXT_SCALE_MAX: f32 = 3.0;

/// High-contrast display theme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContrastMode {
    /// OLED-optimised pure black background, dim grey text (default).
    Dark,
    /// Pure white background, near-black text.
    Light,
    /// Maximum contrast: pure black/white with accent colours removed.
    HighContrast,
}

/// Preference for animation and motion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotionPreference {
    /// All animations enabled (default).
    Full,
    /// Reduce non-essential animation (e.g. slide transitions → instant).
    Reduced,
    /// No animation whatsoever.
    None,
}

/// Caption preference for audio content.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaptionPreference {
    /// Show captions when available.
    WhenAvailable,
    /// Always show captions.
    Always,
    /// Never show captions.
    Off,
}

/// Complete accessibility settings for a local profile.
///
/// Defaults follow the ABDD requirement: dark, large-touch-target, audio
/// fallback enabled, motion reduced on first launch to save battery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11ySettings {
    pub contrast_mode: ContrastMode,
    pub text_scale: f32,
    pub motion_preference: MotionPreference,
    pub caption_preference: CaptionPreference,
    /// If `true`, lessons prefer audio narration over text where available.
    pub audio_first: bool,
    /// If `true`, use large touch targets (≥ 48 dp) even in dense layouts.
    pub large_touch_targets: bool,
}

impl Default for A11ySettings {
    fn default() -> Self {
        Self {
            contrast_mode: ContrastMode::Dark,
            text_scale: 1.0,
            motion_preference: MotionPreference::Reduced,
            caption_preference: CaptionPreference::WhenAvailable,
            audio_first: false,
            large_touch_targets: true,
        }
    }
}

impl A11ySettings {
    /// Return text_scale clamped to the allowed range.
    pub fn clamped_text_scale(&self) -> f32 {
        self.text_scale.clamp(TEXT_SCALE_MIN, TEXT_SCALE_MAX)
    }
}
