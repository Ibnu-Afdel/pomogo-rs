pub mod ambient;
pub mod bigclock;
pub mod borders;
pub mod centered;
pub mod classic;
pub mod command_center;
pub mod compact;
pub mod dashboard;
pub mod focus_stack;
pub mod minimal;
pub mod monolith;
pub mod retro;
pub mod terminal_rice;
pub mod text;
pub mod tinybar;
pub mod widgets;

use chrono::{Duration, Local};
use crate::theme::Theme;
use crate::timer::SessionPhase;

#[derive(Debug, Clone, Default)]
pub struct DisplayState {
    pub mode_label: String,
    pub project: String,
    pub task: String,
    pub phase_kind: SessionPhase,
    pub segment_remaining: Duration,
    pub block_remaining: Duration,
    pub progress: f64,
    pub segment_index: usize,
    pub segment_count: usize,
    pub paused: bool,
    pub running: bool,
    pub idle: bool,
    pub status_message: String,
    pub hints_visibility: bool,
    pub theme_name: String,
    pub layout_name: String,
    pub zen: bool,
    pub git_branch: String,
    pub tmux_session: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Frame {
    pub width: usize,
    pub height: usize,
}

pub type LayoutFn = fn(&DisplayState, &Theme, &Frame) -> String;

pub struct LayoutSpec {
    pub layout: LayoutFn,
    pub min_width: usize,
    pub min_height: usize,
}

pub fn get_layout_specs() -> [(&'static str, LayoutSpec); 11] {
    [
        ("classic", LayoutSpec { layout: classic::classic, min_width: 50, min_height: 16 }),
        ("minimal", LayoutSpec { layout: minimal::minimal, min_width: 40, min_height: 10 }),
        ("centered", LayoutSpec { layout: centered::centered, min_width: 50, min_height: 14 }),
        ("compact", LayoutSpec { layout: compact::compact, min_width: 40, min_height: 8 }),
        ("retro", LayoutSpec { layout: retro::retro, min_width: 50, min_height: 16 }),
        ("dashboard", LayoutSpec { layout: dashboard::dashboard, min_width: 60, min_height: 16 }),
        ("monolith", LayoutSpec { layout: monolith::monolith, min_width: 56, min_height: 14 }),
        ("tinybar", LayoutSpec { layout: tinybar::tinybar, min_width: 40, min_height: 4 }),
        ("terminal-rice", LayoutSpec { layout: terminal_rice::terminal_rice, min_width: 56, min_height: 16 }),
        ("focus-stack", LayoutSpec { layout: focus_stack::focus_stack, min_width: 54, min_height: 14 }),
        ("command-center", LayoutSpec { layout: command_center::command_center, min_width: 70, min_height: 18 }),
    ]
}

pub fn resolve_layout(name: &str, width: usize, height: usize) -> (&'static str, LayoutFn) {
    let specs = get_layout_specs();

    // Check if requested layout exists and fits
    for (l_name, spec) in &specs {
        if *l_name == name {
            if width >= spec.min_width && height >= spec.min_height {
                return (*l_name, spec.layout);
            }
        }
    }

    // Preference fallback order for smaller terminals
    let order = [
        "tinybar", "minimal", "compact", "focus-stack", "centered",
        "classic", "dashboard", "monolith", "retro", "terminal-rice", "command-center",
    ];

    for ord in order {
        for (l_name, spec) in &specs {
            if *l_name == ord && width >= spec.min_width && height >= spec.min_height {
                return (*l_name, spec.layout);
            }
        }
    }

    // Default minimal
    ("minimal", minimal::minimal)
}

pub fn resolve_layout_name(configured: &str) -> String {
    let layouts = [
        "classic", "minimal", "centered", "compact", "retro", "dashboard",
        "monolith", "tinybar", "terminal-rice", "focus-stack", "command-center",
    ];

    if configured == "random" {
        let now_seed = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) + std::process::id() as i64;
        let idx = (now_seed.abs() as usize) % layouts.len();
        return layouts[idx].to_string();
    }

    if configured == "daily" {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let mut hash: i64 = 0;
        for b in date_str.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as i64);
        }
        let idx = (hash.abs() as usize) % layouts.len();
        return layouts[idx].to_string();
    }

    if configured.is_empty() {
        return "classic".to_string();
    }

    configured.to_string()
}

pub fn resolve_effects_name(configured: &str) -> String {
    if configured == "random" {
        let effects = ["stars", "snow", "rain", "embers", "scanline"];
        let now_seed = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) + std::process::id() as i64;
        let idx = (now_seed.abs() as usize) % effects.len();
        return effects[idx].to_string();
    }
    configured.to_string()
}

pub fn phase_color(phase: SessionPhase, th: &Theme) -> &str {
    match phase {
        SessionPhase::Work => th.work(),
        SessionPhase::ShortBreak => th.brk(),
        SessionPhase::LongBreak => th.long_break(),
    }
}

pub fn format_clock(ds: &DisplayState) -> String {
    if ds.block_remaining > Duration::zero() {
        let hours = ds.block_remaining.num_hours();
        let mins = ds.block_remaining.num_minutes() % 60;
        let secs = ds.block_remaining.num_seconds() % 60;
        format!("{:02}:{:02}:{:02}", hours, mins, secs)
    } else {
        let mins = ds.segment_remaining.num_minutes();
        let secs = ds.segment_remaining.num_seconds() % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_clock() {
        let mut ds = DisplayState::default();
        ds.segment_remaining = Duration::minutes(25);
        assert_eq!(format_clock(&ds), "25:00");

        ds.block_remaining = Duration::hours(1) + Duration::minutes(15) + Duration::seconds(4);
        assert_eq!(format_clock(&ds), "01:15:04");
    }

    #[test]
    fn test_resolve_layout_name() {
        assert_eq!(resolve_layout_name(""), "classic");
        assert_eq!(resolve_layout_name("monolith"), "monolith");
        let daily = resolve_layout_name("daily");
        assert!(!daily.is_empty());
        let random = resolve_layout_name("random");
        assert!(!random.is_empty());
    }

    #[test]
    fn test_all_11_layouts_render_without_panic() {
        let th = crate::theme::get("tokyo-night");
        let frame = Frame {
            width: 90,
            height: 30,
        };

        let mut ds = DisplayState::default();
        ds.mode_label = "Quick Focus".to_string();
        ds.project = "Pomogo-Rust".to_string();
        ds.task = "Testing Layouts".to_string();
        ds.phase_kind = SessionPhase::Work;
        ds.segment_remaining = Duration::minutes(24) + Duration::seconds(30);
        ds.progress = 0.5;
        ds.running = true;
        ds.hints_visibility = true;
        ds.theme_name = "tokyo-night".to_string();
        ds.git_branch = "main".to_string();

        let specs = get_layout_specs();
        for (name, spec) in &specs {
            let rendered = (spec.layout)(&ds, &th, &frame);
            assert!(
                !rendered.is_empty(),
                "Layout {} produced empty rendering",
                name
            );
        }
    }
}


