// Desktop notification and sound cue support for Linux and Omarchy.

use std::process::Command;
use notify_rust::{Notification, Urgency};
use crate::timer::{SessionPhase, SessionState};

pub const DEFAULT_SOUND_START_EVENT: &str = "message-new-instant";
pub const DEFAULT_SOUND_END_EVENT: &str = "complete";

#[derive(Debug, Clone)]
pub struct SoundProfile {
    pub name: &'static str,
    pub description: &'static str,
    pub start_event: &'static str,
    pub end_event: &'static str,
}

pub fn sound_profiles() -> Vec<SoundProfile> {
    vec![
        SoundProfile {
            name: "soft",
            description: "Subtle focus cue and completion chime",
            start_event: "message-new-instant",
            end_event: "complete",
        },
        SoundProfile {
            name: "bell",
            description: "Classic terminal bell cues",
            start_event: "bell",
            end_event: "bell",
        },
        SoundProfile {
            name: "dialog",
            description: "Desktop notification style",
            start_event: "dialog-information",
            end_event: "complete",
        },
        SoundProfile {
            name: "service",
            description: "Login/logout style transitions",
            start_event: "service-login",
            end_event: "service-logout",
        },
    ]
}

#[derive(Debug, Clone)]
pub struct Notifier {
    pub enabled: bool,
    pub sound_enabled: bool,
    pub sound_start_event: String,
    pub sound_end_event: String,
}

impl Notifier {
    pub fn new(
        enabled: bool,
        sound_enabled: bool,
        start_event: Option<String>,
        end_event: Option<String>,
    ) -> Self {
        Self {
            enabled,
            sound_enabled,
            sound_start_event: start_event.unwrap_or_else(|| DEFAULT_SOUND_START_EVENT.to_string()),
            sound_end_event: end_event.unwrap_or_else(|| DEFAULT_SOUND_END_EVENT.to_string()),
        }
    }

    pub fn set_sound_events(&mut self, start: String, end: String) {
        if !start.is_empty() {
            self.sound_start_event = start;
        }
        if !end.is_empty() {
            self.sound_end_event = end;
        }
    }

    pub fn play_sound(&self, state: SessionState) {
        if !self.sound_enabled {
            return;
        }
        let event_id = match state {
            SessionState::ShortBreak | SessionState::LongBreak => &self.sound_end_event,
            SessionState::Work => &self.sound_start_event,
            _ => return,
        };
        self.play_event(event_id);
    }

    pub fn preview_sound_event(&self, event_id: &str) {
        if !event_id.is_empty() {
            self.play_event(event_id);
        }
    }

    fn play_event(&self, event_id: &str) {
        let event = event_id.to_string();
        std::thread::spawn(move || {
            // Attempt canberra-gtk-play
            let res = Command::new("canberra-gtk-play")
                .arg("-i")
                .arg(&event)
                .output();

            if res.is_err() {
                // Fallback to terminal bell
                print!("\x07");
            }
        });
    }

    pub fn notify_transition(&self, state: SessionState, _phase: SessionPhase) {
        self.play_sound(state);

        if !self.enabled {
            return;
        }

        let (title, message, urgency) = match state {
            SessionState::Work => (
                "PomoGo — Focus Time",
                "Break's over. Time to focus.",
                Urgency::Normal,
            ),
            SessionState::ShortBreak => (
                "PomoGo — Short Break",
                "Session done. Stretch, hydrate, rest your eyes.",
                Urgency::Normal,
            ),
            SessionState::LongBreak => (
                "PomoGo — Long Break",
                "You've earned it. Take 15 minutes to recharge.",
                Urgency::Normal,
            ),
            SessionState::Idle => (
                "PomoGo — Session Complete",
                "Great work! Ready for the next session?",
                Urgency::Low,
            ),
        };

        let title_s = title.to_string();
        let msg_s = message.to_string();

        std::thread::spawn(move || {
            let mut n = Notification::new();
            n.appname("pomogo")
                .summary(&title_s)
                .body(&msg_s)
                .urgency(urgency);

            // Send via D-Bus notify-rust
            if n.show().is_err() {
                // Fallback to notify-send CLI
                let u_str = match urgency {
                    Urgency::Low => "low",
                    Urgency::Normal => "normal",
                    Urgency::Critical => "critical",
                };
                let _ = Command::new("notify-send")
                    .args(["-a", "pomogo", "-u", u_str, &title_s, &msg_s])
                    .spawn();
            }
        });
    }

    pub fn notify_custom(&self, title: &str, message: &str, urgency: Urgency) {
        if !self.enabled {
            return;
        }
        let t = title.to_string();
        let m = message.to_string();
        std::thread::spawn(move || {
            let _ = Notification::new()
                .appname("pomogo")
                .summary(&t)
                .body(&m)
                .urgency(urgency)
                .show();
        });
    }
}

