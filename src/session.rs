// Focus session planning and execution coordination.

use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::timer::{Clock, Session, SessionPhase, SessionState};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SegmentKind {
    Work,
    ShortBreak,
    LongBreak,
}

impl SegmentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            SegmentKind::Work => "Work",
            SegmentKind::ShortBreak => "Short Break",
            SegmentKind::LongBreak => "Long Break",
        }
    }
}

impl std::fmt::Display for SegmentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Segment {
    pub kind: SegmentKind,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    Quick,
    Deep,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Quick => "quick",
            Mode::Deep => "deep",
        }
    }
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub mode: Mode,
    pub segments: Vec<Segment>,
    pub index: usize,
    pub planned_total: Duration,
    pub auto_advance: bool,
    pub current_segment: Segment,
    // Quick focus generator state
    pub session_count: usize,
    pub sessions_before_long_break: usize,
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
}

pub fn build_deep_plan(
    total: Duration,
    work: Duration,
    short_break: Duration,
    long_break: Duration,
    sessions_before_long_break: usize,
) -> Vec<Segment> {
    if total <= work {
        return vec![Segment {
            kind: SegmentKind::Work,
            duration: total,
        }];
    }

    let mut segments = Vec::new();
    let mut remaining = total;
    let mut work_sessions = 0;

    while remaining > Duration::zero() {
        let (break_kind, break_duration) = if sessions_before_long_break > 0
            && (work_sessions + 1) % sessions_before_long_break == 0
        {
            (SegmentKind::LongBreak, long_break)
        } else {
            (SegmentKind::ShortBreak, short_break)
        };

        if remaining - work - break_duration >= Duration::minutes(10) {
            segments.push(Segment {
                kind: SegmentKind::Work,
                duration: work,
            });
            segments.push(Segment {
                kind: break_kind,
                duration: break_duration,
            });
            work_sessions += 1;
            remaining = remaining - work - break_duration;
        } else {
            segments.push(Segment {
                kind: SegmentKind::Work,
                duration: remaining,
            });
            remaining = Duration::zero();
        }
    }

    segments
}

impl Block {
    pub fn new_quick(
        work: Duration,
        short: Duration,
        long: Duration,
        before_long: usize,
        auto_advance: bool,
    ) -> Self {
        Self {
            mode: Mode::Quick,
            segments: Vec::new(),
            index: 0,
            planned_total: Duration::zero(),
            auto_advance,
            current_segment: Segment {
                kind: SegmentKind::Work,
                duration: work,
            },
            session_count: 0,
            sessions_before_long_break: before_long,
            work_duration: work,
            short_break_duration: short,
            long_break_duration: long,
        }
    }

    pub fn new_deep(
        total: Duration,
        work: Duration,
        short_break: Duration,
        long_break: Duration,
        sessions_before_long_break: usize,
        auto_advance: bool,
    ) -> Self {
        let segs = build_deep_plan(
            total,
            work,
            short_break,
            long_break,
            sessions_before_long_break,
        );
        let current = if !segs.is_empty() {
            segs[0].clone()
        } else {
            Segment {
                kind: SegmentKind::Work,
                duration: total,
            }
        };

        Self {
            mode: Mode::Deep,
            segments: segs,
            index: 0,
            planned_total: total,
            auto_advance,
            current_segment: current,
            session_count: 0,
            sessions_before_long_break,
            work_duration: work,
            short_break_duration: short_break,
            long_break_duration: long_break,
        }
    }

    pub fn remaining(&self, seg_remaining: Duration) -> Duration {
        if self.mode == Mode::Quick {
            return seg_remaining;
        }
        if self.index >= self.segments.len() {
            return Duration::zero();
        }
        let mut rem = seg_remaining;
        for i in (self.index + 1)..self.segments.len() {
            rem += self.segments[i].duration;
        }
        rem
    }

    pub fn advance(&mut self) -> (Segment, bool) {
        self.index += 1;
        if self.mode == Mode::Deep {
            if self.index >= self.segments.len() {
                return (self.current_segment.clone(), false);
            }
            self.current_segment = self.segments[self.index].clone();
            return (self.current_segment.clone(), true);
        }

        // Quick focus cycle
        match self.current_segment.kind {
            SegmentKind::Work => {
                self.session_count += 1;
                let is_long = self.sessions_before_long_break > 0
                    && self.session_count.is_multiple_of(self.sessions_before_long_break);
                let kind = if is_long {
                    SegmentKind::LongBreak
                } else {
                    SegmentKind::ShortBreak
                };
                let dur = if is_long {
                    self.long_break_duration
                } else {
                    self.short_break_duration
                };
                self.current_segment = Segment {
                    kind,
                    duration: dur,
                };
            }
            SegmentKind::ShortBreak | SegmentKind::LongBreak => {
                self.current_segment = Segment {
                    kind: SegmentKind::Work,
                    duration: self.work_duration,
                };
            }
        }
        (self.current_segment.clone(), true)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerEventType {
    PhaseChanged,
    BlockEnded,
}

#[derive(Debug, Clone)]
pub struct RunnerEvent {
    pub event_type: RunnerEventType,
    pub state: SessionState,
    pub phase: SessionPhase,
}

#[derive(Debug, Clone)]
pub struct Runner {
    pub block: Block,
    pub timer: Session,
}

impl Runner {
    pub fn new(block: Block) -> Self {
        let timer = match block.mode {
            Mode::Quick => Session::new(
                block.work_duration,
                block.short_break_duration,
                block.long_break_duration,
                block.sessions_before_long_break,
            ),
            Mode::Deep => {
                let initial_dur = block.current_segment.duration;
                let mut s = Session::new(
                    initial_dur,
                    block.short_break_duration,
                    block.long_break_duration,
                    block.sessions_before_long_break,
                );
                s.phase = match block.current_segment.kind {
                    SegmentKind::Work => SessionPhase::Work,
                    SegmentKind::ShortBreak => SessionPhase::ShortBreak,
                    SegmentKind::LongBreak => SessionPhase::LongBreak,
                };
                s
            }
        };

        Self { block, timer }
    }

    pub fn start(&mut self, clock: &dyn Clock) -> Result<(), &'static str> {
        self.timer.start(clock)
    }

    pub fn tick(&mut self, clock: &dyn Clock) -> Option<RunnerEvent> {
        if self.timer.tick(clock) {
            // Segment ended
            let (_next_seg, ok) = self.block.advance();
            if !ok {
                return Some(RunnerEvent {
                    event_type: RunnerEventType::BlockEnded,
                    state: SessionState::Idle,
                    phase: SessionPhase::Work,
                });
            }

            self.sync_timer_with_block();
            if self.block.auto_advance {
                let _ = self.timer.start(clock);
            }

            return Some(RunnerEvent {
                event_type: RunnerEventType::PhaseChanged,
                state: self.timer.state,
                phase: self.timer.phase,
            });
        }
        None
    }

    pub fn skip(&mut self, clock: &dyn Clock) -> (RunnerEvent, bool) {
        let (_next_seg, ok) = self.block.advance();
        if !ok {
            self.timer.reset();
            return (
                RunnerEvent {
                    event_type: RunnerEventType::BlockEnded,
                    state: SessionState::Idle,
                    phase: SessionPhase::Work,
                },
                false,
            );
        }

        self.sync_timer_with_block();
        if self.block.auto_advance {
            let _ = self.timer.start(clock);
        }

        (
            RunnerEvent {
                event_type: RunnerEventType::PhaseChanged,
                state: self.timer.state,
                phase: self.timer.phase,
            },
            true,
        )
    }

    pub fn sync_timer_with_block(&mut self) {
        if self.block.mode == Mode::Deep {
            self.timer.work_duration = self.block.current_segment.duration;
            self.timer.remaining_time = self.block.current_segment.duration;
            self.timer.phase = match self.block.current_segment.kind {
                SegmentKind::Work => SessionPhase::Work,
                SegmentKind::ShortBreak => SessionPhase::ShortBreak,
                SegmentKind::LongBreak => SessionPhase::LongBreak,
            };
            self.timer.state = match self.timer.phase {
                SessionPhase::Work => SessionState::Work,
                SessionPhase::ShortBreak => SessionState::ShortBreak,
                SessionPhase::LongBreak => SessionState::LongBreak,
            };
        } else {
            self.timer.phase = match self.block.current_segment.kind {
                SegmentKind::Work => SessionPhase::Work,
                SegmentKind::ShortBreak => SessionPhase::ShortBreak,
                SegmentKind::LongBreak => SessionPhase::LongBreak,
            };
            self.timer.state = match self.timer.phase {
                SessionPhase::Work => SessionState::Work,
                SessionPhase::ShortBreak => SessionState::ShortBreak,
                SessionPhase::LongBreak => SessionState::LongBreak,
            };
            self.timer.remaining_time = self.block.current_segment.duration;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_deep_plan() {
        let plan = build_deep_plan(
            Duration::minutes(120),
            Duration::minutes(25),
            Duration::minutes(5),
            Duration::minutes(15),
            4,
        );
        assert!(!plan.is_empty());
        let total: Duration = plan.iter().map(|s| s.duration).sum();
        assert_eq!(total, Duration::minutes(120));
    }
}

