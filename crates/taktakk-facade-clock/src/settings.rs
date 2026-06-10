//! Facade clock settings stored in `facade.sqlite`.
//!
//! All keys and values are chosen to look like ordinary clock configuration.
//! No educational or security terminology appears here.

use serde::{Deserialize, Serialize};

use crate::clock::ClockDisplay;

/// Settings for the facade clock, as persisted in `facade.sqlite`.
///
/// These settings survive a panic wipe: a wiped device looks like
/// an ordinary, unconfigured clock app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacadeSettings {
    /// Default clock display mode.
    pub display_mode: ClockDisplay,
    /// Whether to show seconds on the digital face.
    pub show_seconds: bool,
    /// Whether to use 24-hour format.
    pub use_24h: bool,
    /// IANA time-zone string (e.g. "Asia/Beirut"). Empty means local.
    pub timezone: String,
    /// Whether to vibrate on alarm.
    pub alarm_vibrate: bool,
}

impl Default for FacadeSettings {
    fn default() -> Self {
        Self {
            display_mode: ClockDisplay::Digital,
            show_seconds: true,
            use_24h: true,
            timezone: String::new(),
            alarm_vibrate: true,
        }
    }
}
