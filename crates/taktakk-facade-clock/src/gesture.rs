//! Stealth unlock and duress gesture parser.
//!
//! The unlock mechanism is disguised as normal clock interactions.
//! No "Unlock" buttons or password fields are ever shown.
//!
//! # Unlock sequence (RFC 002)
//!
//! **Primary trigger:** Set an alarm to a specific "magic time" and then
//! long-press the Confirm button for at least [`LONG_PRESS_THRESHOLD_MS`] ms.
//!
//! **Alternative trigger:** Tap the clock face center in a specific rhythm
//! pattern defined in [`TapPattern`].
//!
//! # Duress trigger (RFC 017)
//!
//! A different "magic time" silently wipes all protected data instead of
//! unlocking. No confirmation dialog is shown; the app resets to a
//! factory-fresh clock state.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Minimum long-press duration to qualify as an unlock attempt (ms).
pub const LONG_PRESS_THRESHOLD_MS: u32 = 2_800;

/// Errors produced by the gesture parser.
#[derive(Debug, Error)]
pub enum GestureError {
    #[error("sequence too short")]
    TooShort,
    #[error("invalid tap pattern")]
    InvalidPattern,
}

/// A tap on the clock face with a position hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tap {
    /// Normalised X position 0..=255 (0 = left edge, 255 = right edge).
    pub x: u8,
    /// Normalised Y position 0..=255 (0 = top edge, 255 = bottom edge).
    pub y: u8,
    /// Time since first tap in this sequence, in milliseconds.
    pub offset_ms: u32,
}

/// An interaction event on the clock facade.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FacadeInput {
    /// User set an alarm to a particular hour:minute.
    AlarmSet { hour: u8, minute: u8 },
    /// User pressed and held the alarm-confirm button.
    AlarmConfirmLongPress { duration_ms: u32 },
    /// User tapped the centre of the analog/digital clock face.
    ClockFaceTap(Tap),
    /// User entered a numeric code in the stopwatch field.
    StopwatchCode { digits: Vec<u8> },
}

/// Outcome after processing one or more facade inputs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GestureOutcome {
    /// Nothing notable; continue showing the clock.
    Idle,
    /// A valid unlock sequence is accumulating; stay in clock mode.
    Pending,
    /// Unlock sequence complete — transition to the protected shell.
    Unlock,
    /// Duress sequence detected — initiate silent wipe.
    Duress,
}

/// Serialisable configuration stored in `facade.sqlite` under an
/// innocuous key name (e.g. `alarm_offset_drift`).
///
/// Fields use obfuscated names to avoid suspicion if the database is
/// inspected without the key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureConfig {
    /// Hour component of the magic alarm time for unlock.
    pub drift_h: u8,
    /// Minute component of the magic alarm time for unlock.
    pub drift_m: u8,
    /// Hour component of the magic alarm time for duress wipe.
    pub offset_h: u8,
    /// Minute component of the magic alarm time for duress wipe.
    pub offset_m: u8,
    /// Tap rhythm pattern for the alternative unlock trigger.
    pub tap_pattern: TapPattern,
}

impl GestureConfig {
    /// Factory default configuration (used before first setup).
    ///
    /// The default magic times are deliberately unusual so they are
    /// unlikely to be set by accident. They can be changed by the user
    /// after the first unlock.
    pub fn default_config() -> Self {
        Self {
            drift_h: 3,
            drift_m: 14,
            offset_h: 12,
            offset_m: 34,
            tap_pattern: TapPattern::default(),
        }
    }
}

/// A tap rhythm pattern: inter-tap intervals in milliseconds.
///
/// The pattern is matched with ±[`TAP_TOLERANCE_MS`] tolerance per interval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TapPattern {
    /// Sequence of inter-tap intervals (ms). Length must be 3–7.
    pub intervals_ms: Vec<u32>,
}

/// Tolerance window for matching a tap interval (ms).
pub const TAP_TOLERANCE_MS: u32 = 250;

impl Default for TapPattern {
    fn default() -> Self {
        // Short-short-long pattern (similar to Morse ·· — ).
        Self { intervals_ms: vec![200, 200, 600] }
    }
}

impl TapPattern {
    /// Check whether `observed_ms` matches this pattern.
    pub fn matches(&self, observed_ms: &[u32]) -> bool {
        if observed_ms.len() != self.intervals_ms.len() {
            return false;
        }
        self.intervals_ms
            .iter()
            .zip(observed_ms)
            .all(|(expected, actual)| actual.abs_diff(*expected) <= TAP_TOLERANCE_MS)
    }
}

/// Stateful gesture accumulator.
///
/// Feed [`FacadeInput`] events and read the current [`GestureOutcome`].
/// Reset after each terminal outcome (`Unlock` or `Duress`).
#[derive(Debug)]
pub struct GestureParser {
    config: GestureConfig,
    alarm_armed: bool,
    pending_alarm_hour: Option<u8>,
    pending_alarm_minute: Option<u8>,
    tap_timestamps: Vec<u32>,
}

impl GestureParser {
    pub fn new(config: GestureConfig) -> Self {
        Self {
            config,
            alarm_armed: false,
            pending_alarm_hour: None,
            pending_alarm_minute: None,
            tap_timestamps: Vec::new(),
        }
    }

    /// Process one facade input event. Returns the current outcome.
    pub fn process(&mut self, input: FacadeInput) -> GestureOutcome {
        match input {
            FacadeInput::AlarmSet { hour, minute } => {
                self.pending_alarm_hour = Some(hour);
                self.pending_alarm_minute = Some(minute);
                self.alarm_armed = false;
                GestureOutcome::Idle
            }

            FacadeInput::AlarmConfirmLongPress { duration_ms } => {
                if duration_ms < LONG_PRESS_THRESHOLD_MS {
                    return GestureOutcome::Idle;
                }
                let (h, m) = match (self.pending_alarm_hour, self.pending_alarm_minute) {
                    (Some(h), Some(m)) => (h, m),
                    _ => return GestureOutcome::Idle,
                };

                self.reset_alarm_state();

                if h == self.config.drift_h && m == self.config.drift_m {
                    return GestureOutcome::Unlock;
                }
                if h == self.config.offset_h && m == self.config.offset_m {
                    return GestureOutcome::Duress;
                }
                GestureOutcome::Idle
            }

            FacadeInput::ClockFaceTap(tap) => {
                self.tap_timestamps.push(tap.offset_ms);
                let needed = self.config.tap_pattern.intervals_ms.len() + 1;
                if self.tap_timestamps.len() < needed {
                    return GestureOutcome::Pending;
                }
                // Compute inter-tap intervals from timestamps.
                let intervals: Vec<u32> = self
                    .tap_timestamps
                    .windows(2)
                    .map(|w| w[1].saturating_sub(w[0]))
                    .collect();
                let matched = self.config.tap_pattern.matches(&intervals);
                self.tap_timestamps.clear();
                if matched { GestureOutcome::Unlock } else { GestureOutcome::Idle }
            }

            FacadeInput::StopwatchCode { digits } => {
                // Encode the digit sequence as a two-digit hour:minute pair
                // for a simpler comparison path.
                if digits.len() == 4 {
                    let h = digits[0] * 10 + digits[1];
                    let m = digits[2] * 10 + digits[3];
                    if h == self.config.drift_h && m == self.config.drift_m {
                        return GestureOutcome::Unlock;
                    }
                    if h == self.config.offset_h && m == self.config.offset_m {
                        return GestureOutcome::Duress;
                    }
                }
                GestureOutcome::Idle
            }
        }
    }

    /// Reset the alarm arm state.
    fn reset_alarm_state(&mut self) {
        self.alarm_armed = false;
        self.pending_alarm_hour = None;
        self.pending_alarm_minute = None;
    }

    /// Reset all accumulated state (call after unlock or duress outcome).
    pub fn reset(&mut self) {
        self.reset_alarm_state();
        self.tap_timestamps.clear();
    }
}
