// Focus layout: the default screen. No box, lots of air, one thing per line.
//
//            FOCUS  ·  write the release notes
//
//     ██████ ██████      ██████ ██████
//     ...     big clock
//
//     ━━━━━━━━━━━━━━━━━━━━━━━━━━━──────────────
//
//     ◍ Drink some water  Have a glass of water…
//
//     today 1h 35m of 4h  ▰▰▰▰▱▱▱▱  ·  3-day streak
//
//  enter pause · n skip · t task · w water · ? keys     (bottom row)

use chrono::Duration;

use crate::render::bigclock::{ansi_bold_fg, ansi_fg, big_clock_rows};
use crate::render::text::{truncate_text, visible_width};
use crate::render::widgets::{center_text, fit_hints, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;
use crate::timer::SessionPhase;

/// tty-clock style digits: 6 columns by 5 rows.
const HEAVY_DIGITS: [[&str; 5]; 10] = [
    ["██████", "██  ██", "██  ██", "██  ██", "██████"],
    ["    ██", "    ██", "    ██", "    ██", "    ██"],
    ["██████", "    ██", "██████", "██    ", "██████"],
    ["██████", "    ██", "██████", "    ██", "██████"],
    ["██  ██", "██  ██", "██████", "    ██", "    ██"],
    ["██████", "██    ", "██████", "    ██", "██████"],
    ["██████", "██    ", "██████", "██  ██", "██████"],
    ["██████", "    ██", "    ██", "    ██", "    ██"],
    ["██████", "██  ██", "██████", "██  ██", "██████"],
    ["██████", "██  ██", "██████", "    ██", "██████"],
];
const HEAVY_COLON: [&str; 5] = ["  ", "██", "  ", "██", "  "];

fn heavy_clock_rows(time: &str, color: &str) -> Vec<String> {
    let mut rows = vec![String::new(); 5];
    for (i, ch) in time.chars().enumerate() {
        let glyph: &[&str; 5] = match ch {
            '0'..='9' => &HEAVY_DIGITS[(ch as u8 - b'0') as usize],
            ':' => &HEAVY_COLON,
            _ => continue,
        };
        for (row, part) in rows.iter_mut().zip(glyph.iter()) {
            if i > 0 {
                row.push_str("  ");
            }
            row.push_str(part);
        }
    }
    rows.into_iter().map(|r| ansi_bold_fg(color, &r)).collect()
}

fn heavy_width(time: &str) -> usize {
    time.chars()
        .map(|c| if c == ':' { 2 } else { 6 })
        .sum::<usize>()
        + 2 * time.chars().count().saturating_sub(1)
}

fn fmt_minutes(d: Duration) -> String {
    let mins = d.num_minutes().max(0);
    match (mins / 60, mins % 60) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

/// Ten-cell meter like ▰▰▰▰▱▱▱▱▱▱.
fn meter(fraction: f64, color: &str, track: &str) -> String {
    let filled = (fraction.clamp(0.0, 1.0) * 10.0).round() as usize;
    format!(
        "{}{}",
        ansi_fg(color, &"▰".repeat(filled)),
        ansi_fg(track, &"▱".repeat(10 - filled))
    )
}

pub fn focus(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = if ds.paused || ds.idle { th.muted() } else { phase_color(ds.phase_kind, th) };
    let accent = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let text = th.text();
    let width = f.width;
    let inner = width.saturating_sub(8).clamp(20, 64);

    // Header: phase and task.
    let phase = if ds.idle {
        "READY"
    } else if ds.paused {
        "PAUSED"
    } else {
        match ds.phase_kind {
            SessionPhase::Work => "FOCUS",
            SessionPhase::ShortBreak => "BREAK",
            SessionPhase::LongBreak => "LONG BREAK",
        }
    };
    let mut header = ansi_bold_fg(accent, phase);
    let label = if !ds.task.is_empty() {
        ds.task.clone()
    } else if !ds.project.is_empty() {
        ds.project.clone()
    } else {
        String::new()
    };
    if !label.is_empty() {
        let budget = inner.saturating_sub(visible_width(phase) + 5);
        header.push_str(&ansi_fg(muted, "  ·  "));
        header.push_str(&ansi_fg(text, &truncate_text(&label, budget, "…")));
    }

    // Clock: heavy digits when they fit, thin digits next, plain text last.
    let time = format_clock(ds);
    let clock: Vec<String> = if heavy_width(&time) + 4 <= width && f.height >= 14 {
        heavy_clock_rows(&time, color)
    } else if time.len() * 5 + 4 <= width && f.height >= 12 {
        big_clock_rows(&time, color)
    } else {
        vec![ansi_bold_fg(color, &time)]
    };

    let mut lines: Vec<String> = Vec::new();
    lines.push(center_text(&header, width));
    lines.push(String::new());
    for row in clock {
        lines.push(center_text(&row, width));
    }
    lines.push(String::new());

    if !ds.zen {
        let bar = progress_bar(ds.progress, inner, accent, th.progress_track());
        lines.push(center_text(&bar, width));
        lines.push(String::new());

        // One context line: a reminder beats a break tip beats what's next.
        let context = if let Some((glyph, title, message)) = &ds.nudge {
            let head = format!("{} {}", glyph, title);
            let rest = truncate_text(message, inner.saturating_sub(visible_width(&head) + 2), "…");
            format!("{}  {}", ansi_bold_fg(th.accent(), &head), ansi_fg(text, &rest))
        } else if !ds.toast.is_empty() {
            ansi_fg(text, &truncate_text(&ds.toast, inner, "…"))
        } else if !ds.break_tip.is_empty() && !ds.idle && ds.phase_kind != SessionPhase::Work {
            ansi_fg(text, &truncate_text(&ds.break_tip, inner, "…"))
        } else {
            ansi_fg(muted, &truncate_text(&ds.next_up, inner, "…"))
        };
        lines.push(center_text(&context, width));
        lines.push(String::new());

        // Today: progress toward the goal, streak and water.
        let mut today = vec![format!(
            "{} {}",
            ansi_fg(muted, "today"),
            ansi_fg(text, &fmt_minutes(ds.today_focus))
        )];
        if ds.daily_goal > Duration::zero() {
            let frac = ds.today_focus.num_seconds() as f64 / ds.daily_goal.num_seconds() as f64;
            today[0].push_str(&ansi_fg(muted, &format!(" of {}  ", fmt_minutes(ds.daily_goal))));
            today[0].push_str(&meter(frac, if frac >= 1.0 { th.long_break() } else { accent }, th.subtle()));
        }
        if ds.streak_days > 1 {
            today.push(ansi_fg(muted, &format!("{}-day streak", ds.streak_days)));
        }
        if ds.water_today > 0 {
            let glasses = if ds.water_today == 1 { "glass" } else { "glasses" };
            today.push(ansi_fg(muted, &format!("{} {} of water", ds.water_today, glasses)));
        }
        let mut today_line = today.join(&ansi_fg(muted, "  ·  "));
        while visible_width(&today_line) > width && today.len() > 1 {
            today.pop();
            today_line = today.join(&ansi_fg(muted, "  ·  "));
        }
        lines.push(center_text(&today_line, width));
    }

    // Vertically centre the body and pin the hints to the last row.
    let hints = if ds.zen {
        String::new()
    } else {
        let items: Vec<&str> = ds.hints.split("  ·  ").collect();
        ansi_fg(muted, &fit_hints(&items, width.saturating_sub(2)))
    };
    let body_rows = f.height.saturating_sub(1);
    let top = body_rows.saturating_sub(lines.len()) / 2;
    let mut out = vec![String::new(); top];
    out.extend(lines);
    out.truncate(body_rows);
    while out.len() < body_rows {
        out.push(String::new());
    }
    out.push(center_text(&hints, width));
    out.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_heavy_width_matches_rendered_rows() {
        for time in ["25:00", "1:02:03", "00:00:00"] {
            let rows = heavy_clock_rows(time, "#ffffff");
            assert_eq!(visible_width(&rows[0]), heavy_width(time), "{}", time);
        }
    }

    #[test]
    fn test_fmt_minutes() {
        assert_eq!(fmt_minutes(Duration::minutes(45)), "45m");
        assert_eq!(fmt_minutes(Duration::minutes(120)), "2h");
        assert_eq!(fmt_minutes(Duration::minutes(95)), "1h 35m");
    }

    #[test]
    fn test_focus_fills_the_frame() {
        let th = crate::theme::get("tokyo-night");
        let ds = DisplayState {
            task: "write the release notes".into(),
            segment_remaining: Duration::minutes(24),
            progress: 0.3,
            running: true,
            today_focus: Duration::minutes(95),
            daily_goal: Duration::minutes(240),
            streak_days: 3,
            water_today: 2,
            next_up: "break in 24m".into(),
            hints: "enter pause · n skip · ? keys".into(),
            ..Default::default()
        };
        for (w, h) in [(30, 8), (60, 16), (120, 40)] {
            let out = focus(&ds, &th, &Frame { width: w, height: h });
            let lines: Vec<&str> = out.split('\n').collect();
            assert_eq!(lines.len(), h);
            assert!(lines.iter().all(|l| visible_width(l) <= w), "{}x{}", w, h);
        }
    }
}
