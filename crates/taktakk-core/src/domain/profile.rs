//! Local learner profile model.
//!
//! Profiles are anonymous by design: no name, no email, no phone number.
//! A profile is a local device identity used only for progress tracking.

use serde::{Deserialize, Serialize};

/// An anonymous local learner profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalProfile {
    /// Randomly generated opaque identifier. Never transmitted externally.
    pub profile_id: String,
    /// Display alias chosen by the learner (optional, never synced).
    pub display_alias: Option<String>,
    /// Preferred locale tag (BCP 47).
    pub locale: Option<String>,
    /// Unix timestamp (seconds) when the profile was created.
    pub created_at: i64,
    /// Unix timestamp (seconds) of last active session.
    pub last_active_at: Option<i64>,
}

impl LocalProfile {
    /// Creates a new anonymous profile with a random ID.
    pub fn new(profile_id: String, now: i64) -> Self {
        Self {
            profile_id,
            display_alias: None,
            locale: None,
            created_at: now,
            last_active_at: None,
        }
    }
}
