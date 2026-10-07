// Session recovery from saved state files.

use chrono::{DateTime, Duration};
use crate::session::{Block, Runner, Segment, SegmentKind};
use crate::statefile::{is_expired, is_stale, StateManager};
use crate::timer::{SessionPhase, SessionState};

pub fn can_restore() -> bool {
    let mgr = match StateManager::new() {
        Ok(m) => m,
        Err(_) => return false,
    };

    let state = match mgr.read() {
        Ok(Some(s)) => s,
        _ => return false,
    };

    if state.state == "idle" || state.state.is_empty() {
        return false;
    }

    if is_expired(&state) {
        return false;
    }

    if !is_stale(&state) {
        return false; // process still alive
    }

    true
}

#[derive(Debug, Clone)]
pub struct RestoreDurations {
    pub quick_work: Duration,
    pub quick_short_break: Duration,
    pub quick_long_break: Duration,
    pub quick_sessions_before_long_break: usize,
    pub quick_auto_advance: bool,

    pub deep_work: Duration,
    pub deep_short_break: Duration,
    pub deep_long_break: Duration,
    pub deep_sessions_before_long_break: usize,
}

pub fn restore_runner(d: &RestoreDurations) -> Result<Runner, String> {
    let mgr = StateManager::new()?;
    let state = mgr.read()?.ok_or_else(|| "no saved state".to_string())?;

    if is_expired(&state) {
        let _ = mgr.remove();
        return Err("saved session has expired".to_string());
    }

    let block = if state.mode == "deep" {
        let planned_total = Duration::seconds(state.planned_total_secs);
        let mut b = Block::new_deep(
            planned_total,
            d.deep_work,
            d.deep_short_break,
            d.deep_long_break,
            d.deep_sessions_before_long_break,
            true,
        );
        b.index = state.segment_index;
        if b.index < b.segments.len() {
            b.current_segment = b.segments[b.index].clone();
        }
        b
    } else {
        let mut b = Block::new_quick(
            d.quick_work,
            d.quick_short_break,
            d.quick_long_break,
            d.quick_sessions_before_long_break,
            d.quick_auto_advance,
        );
        b.session_count = state.session_count;
        let kind = match state.session_type.as_str() {
            "short_break" => SegmentKind::ShortBreak,
            "long_break" => SegmentKind::LongBreak,
            _ => SegmentKind::Work,
        };
        let dur = match kind {
            SegmentKind::Work => d.quick_work,
            SegmentKind::ShortBreak => d.quick_short_break,
            SegmentKind::LongBreak => d.quick_long_break,
        };
        b.current_segment = Segment { kind, duration: dur };
        b
    };

    let mut runner = Runner::new(block);
    let phase = match state.session_type.as_str() {
        "short_break" => SessionPhase::ShortBreak,
        "long_break" => SessionPhase::LongBreak,
        _ => SessionPhase::Work,
    };
    let session_st = match state.state.as_str() {
        "work" => SessionState::Work,
        "short_break" => SessionState::ShortBreak,
        "long_break" => SessionState::LongBreak,
        _ => SessionState::Idle,
    };

    runner.timer.phase = phase;
    runner.timer.state = session_st;
    runner.timer.is_running = session_st != SessionState::Idle;
    runner.timer.is_paused = state.paused;
    runner.timer.session_count = state.session_count;

    if state.started_at > 0 {
        runner.timer.started_at = DateTime::from_timestamp(state.started_at, 0);
    }
    if state.ends_at > 0 {
        runner.timer.ends_at = DateTime::from_timestamp(state.ends_at, 0);
    }
    runner.timer.remaining_time = Duration::seconds(state.remaining_secs);

    Ok(runner)
}

