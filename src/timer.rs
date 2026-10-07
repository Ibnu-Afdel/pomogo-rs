// Core Pomodoro session state machine.
// Pure Rust with no I/O dependencies. Time is injected via Clock trait.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionState {
    #[default]
    Idle,
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionState {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionState::Idle => "Idle",
            SessionState::Work => "Work",
            SessionState::ShortBreak => "ShortBreak",
            SessionState::LongBreak => "LongBreak",
        }
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SessionPhase {
    #[default]
    Work,
    ShortBreak,
    LongBreak,
}

impl SessionPhase {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionPhase::Work => "Work",
            SessionPhase::ShortBreak => "ShortBreak",
            SessionPhase::LongBreak => "LongBreak",
        }
    }
}

impl std::fmt::Display for SessionPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Start,
    Pause,
    Resume,
    Skip,
    Complete,
    Reset,
}

pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

#[derive(Default, Clone, Copy)]
pub struct RealClock;

impl Clock for RealClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

#[derive(Clone)]
pub struct MockClock {
    pub current: DateTime<Utc>,
}

impl MockClock {
    pub fn new(t: DateTime<Utc>) -> Self {
        Self { current: t }
    }

    pub fn advance(&mut self, d: Duration) {
        self.current = self.current + d;
    }
}

impl Clock for MockClock {
    fn now(&self) -> DateTime<Utc> {
        self.current
    }
}

#[derive(Debug, Clone)]
pub struct Session {
    pub state: SessionState,
    pub phase: SessionPhase,

    // Durations configured at start
    pub work_duration: Duration,
    pub short_break_duration: Duration,
    pub long_break_duration: Duration,
    pub sessions_before_long_break: usize,

    // Session tracking
    pub session_count: usize,
    pub sessions_until_long_break: usize,

    // Timing
    pub started_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub paused_at: Option<DateTime<Utc>>,
    pub remaining_time: Duration,

    // Flags
    pub is_running: bool,
    pub is_paused: bool,
}

impl Session {
    pub fn new(
        work_duration: Duration,
        short_break_duration: Duration,
        long_break_duration: Duration,
        sessions_before_long_break: usize,
    ) -> Self {
        Self {
            state: SessionState::Idle,
            phase: SessionPhase::Work,
            work_duration,
            short_break_duration,
            long_break_duration,
            sessions_before_long_break,
            session_count: 0,
            sessions_until_long_break: sessions_before_long_break,
            started_at: None,
            ends_at: None,
            paused_at: None,
            remaining_time: Duration::zero(),
            is_running: false,
            is_paused: false,
        }
    }

    pub fn start(&mut self, clock: &dyn Clock) -> Result<(), &'static str> {
        if self.is_running {
            return Err("session already running");
        }

        if self.state == SessionState::Idle {
            self.phase = SessionPhase::Work;
            self.state = SessionState::Work;
        } else {
            self.state = match self.phase {
                SessionPhase::Work => SessionState::Work,
                SessionPhase::ShortBreak => SessionState::ShortBreak,
                SessionPhase::LongBreak => SessionState::LongBreak,
            };
        }

        self.is_running = true;
        self.is_paused = false;

        let now = clock.now();
        self.started_at = Some(now);
        self.remaining_time = self.duration_for_phase();
        self.ends_at = Some(now + self.remaining_time);

        Ok(())
    }

    pub fn pause(&mut self, clock: &dyn Clock) -> Result<(), &'static str> {
        if !self.is_running || self.is_paused {
            return Err("cannot pause: session not running or already paused");
        }

        self.is_paused = true;
        let now = clock.now();
        self.paused_at = Some(now);
        if let Some(ends) = self.ends_at {
            self.remaining_time = ends - now;
            if self.remaining_time < Duration::zero() {
                self.remaining_time = Duration::zero();
            }
        }
        Ok(())
    }

    pub fn resume(&mut self, clock: &dyn Clock) -> Result<(), &'static str> {
        if !self.is_running || !self.is_paused {
            return Err("cannot resume: session not paused");
        }

        self.is_paused = false;
        if let Some(paused_at) = self.paused_at {
            let paused_duration = clock.now() - paused_at;
            if let Some(started) = self.started_at {
                self.started_at = Some(started + paused_duration);
            }
            if let Some(ends) = self.ends_at {
                self.ends_at = Some(ends + paused_duration);
            }
        }
        self.paused_at = None;
        Ok(())
    }

    pub fn tick(&mut self, clock: &dyn Clock) -> bool {
        if !self.is_running || self.is_paused {
            return false;
        }

        let now = clock.now();
        if let Some(ends) = self.ends_at {
            if now >= ends {
                self.complete();
                return true;
            }
            self.remaining_time = ends - now;
            if self.remaining_time < Duration::zero() {
                self.remaining_time = Duration::zero();
            }
        }
        false
    }

    pub fn skip(&mut self) -> SessionState {
        if self.phase == SessionPhase::Work {
            self.session_count += 1;
            if self.sessions_until_long_break > 0 {
                self.sessions_until_long_break -= 1;
            }
        }
        self.is_running = false;
        self.is_paused = false;
        self.remaining_time = Duration::zero();
        self.started_at = None;
        self.ends_at = None;
        self.paused_at = None;

        let next = self.advance_phase();
        if next == SessionState::LongBreak {
            self.sessions_until_long_break = self.sessions_before_long_break;
        }
        next
    }

    pub fn complete(&mut self) -> SessionState {
        if self.phase == SessionPhase::Work {
            self.session_count += 1;
            if self.sessions_until_long_break > 0 {
                self.sessions_until_long_break -= 1;
            }
        }

        self.is_running = false;
        self.is_paused = false;
        let next = self.advance_phase();
        if next == SessionState::LongBreak {
            self.sessions_until_long_break = self.sessions_before_long_break;
        }
        next
    }

    fn advance_phase(&mut self) -> SessionState {
        match self.phase {
            SessionPhase::Work => {
                if self.sessions_until_long_break == 0 {
                    self.phase = SessionPhase::LongBreak;
                    self.state = SessionState::LongBreak;
                    SessionState::LongBreak
                } else {
                    self.phase = SessionPhase::ShortBreak;
                    self.state = SessionState::ShortBreak;
                    SessionState::ShortBreak
                }
            }
            SessionPhase::ShortBreak | SessionPhase::LongBreak => {
                self.phase = SessionPhase::Work;
                self.state = SessionState::Work;
                SessionState::Work
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = SessionState::Idle;
        self.phase = SessionPhase::Work;
        self.is_running = false;
        self.is_paused = false;
        self.started_at = None;
        self.ends_at = None;
        self.paused_at = None;
        self.remaining_time = Duration::zero();
    }

    pub fn add_time(&mut self, d: Duration) {
        if !self.is_running {
            return;
        }
        self.remaining_time = self.remaining_time + d;
        if !self.is_paused {
            if let Some(ends) = self.ends_at {
                self.ends_at = Some(ends + d);
            }
        }
    }

    pub fn duration_for_phase(&self) -> Duration {
        match self.phase {
            SessionPhase::Work => self.work_duration,
            SessionPhase::ShortBreak => self.short_break_duration,
            SessionPhase::LongBreak => self.long_break_duration,
        }
    }

    pub fn handle(&mut self, event: Event, clock: &dyn Clock) -> Result<(), &'static str> {
        match event {
            Event::Start => self.start(clock),
            Event::Pause => self.pause(clock),
            Event::Resume => self.resume(clock),
            Event::Skip => {
                self.skip();
                Ok(())
            }
            Event::Complete => {
                self.complete();
                Ok(())
            }
            Event::Reset => {
                self.reset();
                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_session() {
        let s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);
        assert_eq!(s.state, SessionState::Idle);
        assert_eq!(s.is_running, false);
        assert_eq!(s.is_paused, false);
        assert_eq!(s.session_count, 0);
        assert_eq!(s.sessions_until_long_break, 4);
    }

    #[test]
    fn test_start_and_pause_resume() {
        let mut clock = MockClock::new(Utc::now());
        let mut s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);

        assert!(s.start(&clock).is_ok());
        assert_eq!(s.state, SessionState::Work);
        assert_eq!(s.is_running, true);
        assert_eq!(s.remaining_time, Duration::minutes(25));

        // Advance 5 minutes
        clock.advance(Duration::minutes(5));
        assert_eq!(s.tick(&clock), false);
        assert_eq!(s.remaining_time, Duration::minutes(20));

        // Pause
        assert!(s.pause(&clock).is_ok());
        assert_eq!(s.is_paused, true);

        // Advance while paused
        clock.advance(Duration::minutes(10));
        assert!(s.resume(&clock).is_ok());
        assert_eq!(s.is_paused, false);
        assert_eq!(s.remaining_time, Duration::minutes(20));
        // Advance 20 minutes -> complete
        clock.advance(Duration::minutes(20));
        assert_eq!(s.tick(&clock), true);
        assert_eq!(s.state, SessionState::ShortBreak);
        assert_eq!(s.session_count, 1);
        assert_eq!(s.sessions_until_long_break, 3);
    }

    #[test]
    fn test_skip() {
        let clock = MockClock::new(Utc::now());
        let mut s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);
        s.start(&clock).unwrap();

        assert_eq!(s.state, SessionState::Work);
        assert_eq!(s.skip(), SessionState::ShortBreak);
        assert_eq!(s.session_count, 1);

        assert_eq!(s.skip(), SessionState::Work);
        assert_eq!(s.session_count, 1);
    }

    #[test]
    fn test_reset() {
        let clock = MockClock::new(Utc::now());
        let mut s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);
        s.start(&clock).unwrap();

        s.reset();
        assert_eq!(s.state, SessionState::Idle);
        assert_eq!(s.is_running, false);
        assert_eq!(s.is_paused, false);
    }

    #[test]
    fn test_extend() {
        let clock = MockClock::new(Utc::now());
        let mut s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);
        s.start(&clock).unwrap();

        s.add_time(Duration::minutes(5));
        assert_eq!(s.remaining_time, Duration::minutes(30));
    }

    #[test]
    fn test_long_break_transition() {
        let mut clock = MockClock::new(Utc::now());
        let mut s = Session::new(Duration::minutes(25), Duration::minutes(5), Duration::minutes(15), 4);

        for i in 1..=4 {
            // Start Work
            s.start(&clock).unwrap();
            assert_eq!(s.state, SessionState::Work);
            // Work phase finishes
            clock.advance(Duration::minutes(25));
            assert_eq!(s.tick(&clock), true);

            if i < 4 {
                assert_eq!(s.state, SessionState::ShortBreak);
                // Start and finish ShortBreak
                s.start(&clock).unwrap();
                clock.advance(Duration::minutes(5));
                assert_eq!(s.tick(&clock), true);
                assert_eq!(s.state, SessionState::Work);
            } else {
                assert_eq!(s.state, SessionState::LongBreak);
            }
        }
    }
}

