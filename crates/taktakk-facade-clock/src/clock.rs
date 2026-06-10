//! Plain clock domain model: time, alarms, timer, stopwatch.
//!
//! Nothing here references the educational platform.

use serde::{Deserialize, Serialize};

/// A time-of-day value with hour, minute, and second.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClockTime {
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
}

impl ClockTime {
    /// Create a new `ClockTime`, returning `None` if any field is out of range.
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        Some(Self { hour, minute, second })
    }

    /// Total seconds since midnight.
    pub fn total_seconds(self) -> u32 {
        u32::from(self.hour) * 3600 + u32::from(self.minute) * 60 + u32::from(self.second)
    }
}

impl std::fmt::Display for ClockTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02}:{:02}:{:02}", self.hour, self.minute, self.second)
    }
}

/// An alarm entry in the facade clock.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmEntry {
    /// Opaque identifier stored in `facade.sqlite`.
    pub alarm_id: String,
    pub hour: u8,
    pub minute: u8,
    pub label: Option<String>,
    pub enabled: bool,
    pub repeat_days: RepeatDays,
}

impl AlarmEntry {
    pub fn time(&self) -> Option<ClockTime> {
        ClockTime::new(self.hour, self.minute, 0)
    }
}

/// Days-of-week bitmask (bit 0 = Monday … bit 6 = Sunday).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RepeatDays(pub u8);

impl RepeatDays {
    pub const NONE: Self = Self(0);
    pub const EVERY_DAY: Self = Self(0b0111_1111);

    pub fn is_set(self, day: u8) -> bool {
        day < 7 && (self.0 >> day) & 1 == 1
    }

    pub fn set(self, day: u8) -> Self {
        Self(self.0 | (1 << day))
    }
}

/// A countdown timer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountdownTimer {
    pub duration_seconds: u32,
    pub remaining_seconds: u32,
    pub state: TimerState,
}

impl CountdownTimer {
    pub fn new(duration_seconds: u32) -> Self {
        Self {
            duration_seconds,
            remaining_seconds: duration_seconds,
            state: TimerState::Idle,
        }
    }

    pub fn tick(&mut self) {
        if self.state == TimerState::Running {
            if self.remaining_seconds > 0 {
                self.remaining_seconds -= 1;
            } else {
                self.state = TimerState::Finished;
            }
        }
    }
}

/// State of a countdown timer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimerState {
    Idle,
    Running,
    Paused,
    Finished,
}

/// A stopwatch with lap support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stopwatch {
    pub elapsed_ms: u64,
    pub state: StopwatchState,
    pub laps: Vec<u64>,
}

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            elapsed_ms: 0,
            state: StopwatchState::Idle,
            laps: Vec::new(),
        }
    }

    pub fn record_lap(&mut self) {
        self.laps.push(self.elapsed_ms);
    }
}

impl Default for Stopwatch {
    fn default() -> Self {
        Self::new()
    }
}

/// State of the stopwatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum StopwatchState {
    Idle,
    Running,
    Paused,
}

/// Toggle between analog and digital display.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClockDisplay {
    Digital,
    Analog,
}

/// The complete facade clock view state (no educational content).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FacadeClockState {
    pub display: ClockDisplay,
    pub alarms: Vec<AlarmEntry>,
    pub timer: CountdownTimer,
    pub stopwatch: Stopwatch,
}

impl Default for FacadeClockState {
    fn default() -> Self {
        Self {
            display: ClockDisplay::Digital,
            alarms: Vec::new(),
            timer: CountdownTimer::new(0),
            stopwatch: Stopwatch::new(),
        }
    }
}
