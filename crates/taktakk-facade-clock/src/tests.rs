//! Unit tests for taktakk-facade-clock.

use crate::clock::{ClockTime, CountdownTimer, RepeatDays, Stopwatch, TimerState};
use crate::gesture::{
    FacadeInput, GestureConfig, GestureOutcome, GestureParser, Tap, TapPattern,
    LONG_PRESS_THRESHOLD_MS,
};
use crate::settings::FacadeSettings;

// --- ClockTime ---

#[test]
fn clock_time_valid() {
    assert!(ClockTime::new(0, 0, 0).is_some());
    assert!(ClockTime::new(23, 59, 59).is_some());
}

#[test]
fn clock_time_invalid_hour() {
    assert!(ClockTime::new(24, 0, 0).is_none());
}

#[test]
fn clock_time_invalid_minute() {
    assert!(ClockTime::new(12, 60, 0).is_none());
}

#[test]
fn clock_time_total_seconds() {
    let t = ClockTime::new(1, 1, 1).unwrap();
    assert_eq!(t.total_seconds(), 3600 + 60 + 1);
}

#[test]
fn clock_time_display_zero_padded() {
    let t = ClockTime::new(3, 4, 5).unwrap();
    assert_eq!(t.to_string(), "03:04:05");
}

// --- RepeatDays ---

#[test]
fn repeat_days_set_and_check() {
    let days = RepeatDays::NONE.set(0).set(6);
    assert!(days.is_set(0));
    assert!(days.is_set(6));
    assert!(!days.is_set(3));
}

#[test]
fn repeat_days_every_day() {
    let days = RepeatDays::EVERY_DAY;
    for d in 0..7 {
        assert!(days.is_set(d));
    }
}

// --- CountdownTimer ---

#[test]
fn timer_counts_down() {
    let mut t = CountdownTimer::new(3);
    t.state = TimerState::Running;
    t.tick();
    assert_eq!(t.remaining_seconds, 2);
    t.tick();
    t.tick();
    assert_eq!(t.remaining_seconds, 0);
    t.tick();
    assert_eq!(t.state, TimerState::Finished);
}

#[test]
fn timer_idle_does_not_tick() {
    let mut t = CountdownTimer::new(10);
    // state is Idle by default
    t.tick();
    assert_eq!(t.remaining_seconds, 10);
}

// --- Stopwatch ---

#[test]
fn stopwatch_records_laps() {
    let mut sw = Stopwatch::new();
    sw.elapsed_ms = 1000;
    sw.record_lap();
    sw.elapsed_ms = 2500;
    sw.record_lap();
    assert_eq!(sw.laps, vec![1000, 2500]);
}

// --- GestureParser: primary unlock ---

fn default_parser() -> GestureParser {
    GestureParser::new(GestureConfig::default_config())
}

#[test]
fn alarm_long_press_correct_time_unlocks() {
    let mut p = default_parser();
    p.process(FacadeInput::AlarmSet { hour: 3, minute: 14 });
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS,
    });
    assert_eq!(out, GestureOutcome::Unlock);
}

#[test]
fn alarm_long_press_duress_time_triggers_duress() {
    let mut p = default_parser();
    p.process(FacadeInput::AlarmSet { hour: 12, minute: 34 });
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS,
    });
    assert_eq!(out, GestureOutcome::Duress);
}

#[test]
fn alarm_long_press_wrong_time_is_idle() {
    let mut p = default_parser();
    p.process(FacadeInput::AlarmSet { hour: 7, minute: 0 });
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS,
    });
    assert_eq!(out, GestureOutcome::Idle);
}

#[test]
fn alarm_short_press_never_unlocks() {
    let mut p = default_parser();
    p.process(FacadeInput::AlarmSet { hour: 3, minute: 14 });
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS - 1,
    });
    assert_eq!(out, GestureOutcome::Idle);
}

#[test]
fn alarm_confirm_without_set_is_idle() {
    let mut p = default_parser();
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS,
    });
    assert_eq!(out, GestureOutcome::Idle);
}

// --- GestureParser: stopwatch code ---

#[test]
fn stopwatch_code_unlock() {
    let mut p = default_parser();
    // default_config: drift_h=3, drift_m=14 → digits [0,3,1,4]
    let out = p.process(FacadeInput::StopwatchCode {
        digits: vec![0, 3, 1, 4],
    });
    assert_eq!(out, GestureOutcome::Unlock);
}

#[test]
fn stopwatch_code_duress() {
    let mut p = default_parser();
    // default_config: offset_h=12, offset_m=34 → digits [1,2,3,4]
    let out = p.process(FacadeInput::StopwatchCode {
        digits: vec![1, 2, 3, 4],
    });
    assert_eq!(out, GestureOutcome::Duress);
}

#[test]
fn stopwatch_wrong_code_is_idle() {
    let mut p = default_parser();
    let out = p.process(FacadeInput::StopwatchCode {
        digits: vec![9, 9, 9, 9],
    });
    assert_eq!(out, GestureOutcome::Idle);
}

// --- GestureParser: tap pattern ---

#[test]
fn tap_pattern_match_unlocks() {
    let pattern = TapPattern { intervals_ms: vec![200, 200, 600] };
    let config = GestureConfig {
        tap_pattern: pattern,
        ..GestureConfig::default_config()
    };
    let mut p = GestureParser::new(config);

    // 4 taps produce 3 intervals: 200, 200, 600 (within tolerance).
    let taps = [(0, 0), (200, 0), (400, 0), (1000, 0)];
    let mut outcome = GestureOutcome::Idle;
    for (offset_ms, _) in taps {
        outcome = p.process(FacadeInput::ClockFaceTap(Tap { x: 128, y: 128, offset_ms }));
    }
    assert_eq!(outcome, GestureOutcome::Unlock);
}

#[test]
fn tap_pattern_mismatch_is_idle() {
    let mut p = default_parser();
    // Wrong rhythm: all taps 100ms apart, pattern expects [200, 200, 600].
    let taps = [0u32, 100, 200, 300];
    let mut outcome = GestureOutcome::Idle;
    for offset_ms in taps {
        outcome = p.process(FacadeInput::ClockFaceTap(Tap { x: 128, y: 128, offset_ms }));
    }
    assert_eq!(outcome, GestureOutcome::Idle);
}

#[test]
fn tap_accumulation_pending_before_complete() {
    let mut p = default_parser();
    // Pattern needs 3 intervals → 4 taps; first 3 taps are Pending.
    let out1 = p.process(FacadeInput::ClockFaceTap(Tap { x: 128, y: 128, offset_ms: 0 }));
    let out2 = p.process(FacadeInput::ClockFaceTap(Tap { x: 128, y: 128, offset_ms: 200 }));
    let out3 = p.process(FacadeInput::ClockFaceTap(Tap { x: 128, y: 128, offset_ms: 400 }));
    assert_eq!(out1, GestureOutcome::Pending);
    assert_eq!(out2, GestureOutcome::Pending);
    assert_eq!(out3, GestureOutcome::Pending);
}

// --- GestureParser: reset ---

#[test]
fn parser_reset_clears_state() {
    let mut p = default_parser();
    p.process(FacadeInput::AlarmSet { hour: 3, minute: 14 });
    p.reset();
    // After reset, confirm press should not unlock.
    let out = p.process(FacadeInput::AlarmConfirmLongPress {
        duration_ms: LONG_PRESS_THRESHOLD_MS,
    });
    assert_eq!(out, GestureOutcome::Idle);
}

// --- FacadeSettings ---

#[test]
fn facade_settings_defaults_are_neutral() {
    let s = FacadeSettings::default();
    assert!(s.show_seconds);
    assert!(s.use_24h);
    assert!(s.timezone.is_empty());
}

// --- TapPattern ---

#[test]
fn tap_pattern_exact_match() {
    let pat = TapPattern { intervals_ms: vec![300, 500] };
    assert!(pat.matches(&[300, 500]));
}

#[test]
fn tap_pattern_within_tolerance() {
    let pat = TapPattern { intervals_ms: vec![300, 500] };
    assert!(pat.matches(&[300 + 200, 500 - 200])); // ±250 tolerance
}

#[test]
fn tap_pattern_outside_tolerance() {
    let pat = TapPattern { intervals_ms: vec![300, 500] };
    assert!(!pat.matches(&[300 + 300, 500])); // 300 > 250 tolerance
}

#[test]
fn tap_pattern_wrong_length() {
    let pat = TapPattern { intervals_ms: vec![300, 500] };
    assert!(!pat.matches(&[300]));
}
