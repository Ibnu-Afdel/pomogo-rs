// Body reminders while you focus: rest your eyes, drink water, move.
//
// Inspired by break-reminder tools such as Stretchly and Workrave, but tied to
// the focus timer: reminders only count time spent *focusing*, and taking a
// break resets the ones a break already takes care of (eyes, stretching).
// Water keeps counting through breaks, since a short break is no guarantee
// you refilled your glass.

use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nudge {
    Eyes,
    Water,
    Stretch,
}

impl Nudge {
    pub const ALL: [Nudge; 3] = [Nudge::Eyes, Nudge::Water, Nudge::Stretch];

    pub fn as_str(&self) -> &'static str {
        match self {
            Nudge::Eyes => "eyes",
            Nudge::Water => "water",
            Nudge::Stretch => "stretch",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            Nudge::Eyes => "Rest your eyes",
            Nudge::Water => "Drink some water",
            Nudge::Stretch => "Stand up and stretch",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Nudge::Eyes => "Look 20 feet away for 20 seconds.",
            Nudge::Water => "Have a glass, then press w.",
            Nudge::Stretch => "Roll your shoulders and stretch your back.",
        }
    }

    pub fn glyph(&self) -> &'static str {
        match self {
            Nudge::Eyes => "◉",
            Nudge::Water => "◍",
            Nudge::Stretch => "↟",
        }
    }

    /// Breaks already rest your eyes and get you out of the chair.
    fn reset_by_break(&self) -> bool {
        matches!(self, Nudge::Eyes | Nudge::Stretch)
    }
}

/// Reminder intervals in minutes of focus; 0 turns a reminder off.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct WellnessConfig {
    pub eyes_minutes: u32,
    pub water_minutes: u32,
    pub stretch_minutes: u32,
}

impl Default for WellnessConfig {
    fn default() -> Self {
        Self { eyes_minutes: 20, water_minutes: 45, stretch_minutes: 60 }
    }
}

impl WellnessConfig {
    pub fn off() -> Self {
        Self { eyes_minutes: 0, water_minutes: 0, stretch_minutes: 0 }
    }

    pub fn interval(&self, nudge: Nudge) -> Option<Duration> {
        let minutes = match nudge {
            Nudge::Eyes => self.eyes_minutes,
            Nudge::Water => self.water_minutes,
            Nudge::Stretch => self.stretch_minutes,
        };
        (minutes > 0).then(|| Duration::minutes(minutes as i64))
    }

    pub fn any_enabled(&self) -> bool {
        Nudge::ALL.iter().any(|n| self.interval(*n).is_some())
    }
}

/// Tracks focus time since each reminder last fired.
#[derive(Debug, Clone)]
pub struct Wellness {
    cfg: WellnessConfig,
    since: [Duration; 3],
}

impl Wellness {
    pub fn new(cfg: WellnessConfig) -> Self {
        Self { cfg, since: [Duration::zero(); 3] }
    }

    fn slot(n: Nudge) -> usize {
        match n {
            Nudge::Eyes => 0,
            Nudge::Water => 1,
            Nudge::Stretch => 2,
        }
    }

    /// Adds focused time and returns the reminders that came due, at most one
    /// per kind. Call once per second while a focus segment is running.
    pub fn focus_elapsed(&mut self, d: Duration) -> Vec<Nudge> {
        let mut due = Vec::new();
        for n in Nudge::ALL {
            let Some(interval) = self.cfg.interval(n) else { continue };
            let slot = &mut self.since[Self::slot(n)];
            *slot += d;
            if *slot >= interval {
                *slot = Duration::zero();
                due.push(n);
            }
        }
        due
    }

    /// A break started: clear the reminders a break takes care of.
    pub fn break_started(&mut self) {
        for n in Nudge::ALL {
            if n.reset_by_break() {
                self.since[Self::slot(n)] = Duration::zero();
            }
        }
    }

    /// The user acted on a reminder early (for example logged water).
    pub fn done(&mut self, n: Nudge) {
        self.since[Self::slot(n)] = Duration::zero();
    }

    /// The reminder that comes due soonest, with the focus time left.
    pub fn next_due(&self) -> Option<(Nudge, Duration)> {
        Nudge::ALL
            .iter()
            .filter_map(|n| {
                let interval = self.cfg.interval(*n)?;
                Some((*n, interval - self.since[Self::slot(*n)]))
            })
            .min_by_key(|(_, left)| *left)
    }
}

/// Something to do with a break, rotated so breaks don't feel the same.
const BREAK_TIPS: [&str; 14] = [
    "Stand up and roll your shoulders back ten times.",
    "Refill your water and drink half of it.",
    "Look out of a window and let your eyes relax.",
    "Stretch your arms overhead and lean side to side.",
    "Walk around the room once. Leave the phone.",
    "Close your eyes and take five slow breaths.",
    "Loosen your neck: tilt ear to shoulder, both sides.",
    "Open your hands wide, then make fists. Ten times.",
    "Stand on your toes for a few seconds, then relax.",
    "Tidy one thing on your desk.",
    "Step outside for a minute of fresh air.",
    "Stretch your hamstrings: reach for your toes, no bouncing.",
    "Unclench your jaw and drop your shoulders.",
    "Write down the very next step for when you return.",
];

const LONG_BREAK_TIPS: [&str; 5] = [
    "Take a proper walk. You've earned it.",
    "Eat something real, away from the screen.",
    "Get some daylight: step outside for a while.",
    "Do a few minutes of full-body stretching.",
    "Call or message someone you like.",
];

pub fn break_tip(long: bool, seed: usize) -> &'static str {
    if long {
        LONG_BREAK_TIPS[seed % LONG_BREAK_TIPS.len()]
    } else {
        BREAK_TIPS[seed % BREAK_TIPS.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(w: &mut Wellness, minutes: i64) -> Vec<Nudge> {
        let mut all = Vec::new();
        for _ in 0..minutes * 60 {
            all.extend(w.focus_elapsed(Duration::seconds(1)));
        }
        all
    }

    #[test]
    fn test_reminders_fire_on_their_intervals() {
        let mut w = Wellness::new(WellnessConfig::default());
        assert_eq!(tick(&mut w, 19), vec![]);
        assert_eq!(tick(&mut w, 1), vec![Nudge::Eyes]);
        // 45 minutes in: second eyes reminder was at 40, water now.
        assert_eq!(tick(&mut w, 25), vec![Nudge::Eyes, Nudge::Water]);
        assert_eq!(tick(&mut w, 15), vec![Nudge::Eyes, Nudge::Stretch]);
    }

    #[test]
    fn test_break_resets_eyes_and_stretch_but_not_water() {
        let mut w = Wellness::new(WellnessConfig::default());
        tick(&mut w, 18);
        w.break_started();
        assert_eq!(tick(&mut w, 19), vec![]);
        // Water kept counting: 18 + 27 = 45.
        assert_eq!(tick(&mut w, 8), vec![Nudge::Eyes, Nudge::Water]);
    }

    #[test]
    fn test_disabled_and_done() {
        let mut w = Wellness::new(WellnessConfig { eyes_minutes: 0, water_minutes: 10, stretch_minutes: 0 });
        tick(&mut w, 9);
        w.done(Nudge::Water);
        assert_eq!(tick(&mut w, 9), vec![]);
        assert_eq!(w.next_due().map(|(n, _)| n), Some(Nudge::Water));
        assert!(Wellness::new(WellnessConfig::off()).next_due().is_none());
    }

    #[test]
    fn test_next_due_picks_the_soonest() {
        let mut w = Wellness::new(WellnessConfig::default());
        tick(&mut w, 5);
        assert_eq!(w.next_due(), Some((Nudge::Eyes, Duration::minutes(15))));
    }
}
