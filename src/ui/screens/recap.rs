// End-of-cycle recap screen summarizing metrics and achievements.

use chrono::Duration;
use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::{center_text, place_center};
use crate::theme::Theme;

#[derive(Debug, Clone, Default)]
pub struct RecapInfo {
    pub total_focused: Duration,
    pub segments: usize,
    pub breaks: usize,
    pub pauses: usize,
    pub streak: usize,
    pub is_deep: bool,
    pub focus_score: usize, // 0 - 10
}

pub fn render_recap(width: usize, height: usize, th: &Theme, info: &RecapInfo) -> String {
    let accent_col = th.accent();
    let muted_col = th.muted();
    let txt_col = th.text();
    let border_col = th.border();

    let mut text_w = if width > 12 { width - 12 } else { 34 };
    if text_w > 50 {
        text_w = 50;
    }
    if text_w < 34 {
        text_w = 34;
    }

    let title = ansi_bold_fg(accent_col, "✦ FOCUS CYCLE COMPLETE ✦");
    let tone = ansi_fg(muted_col, recap_tone(info));

    let mode_str = if info.is_deep { "Deep Focus" } else { "Quick Focus" };

    let hrs = info.total_focused.num_hours();
    let mins = info.total_focused.num_minutes() % 60;
    let secs = info.total_focused.num_seconds() % 60;
    let time_str = if hrs > 0 {
        format!("{}h {}m", hrs, mins)
    } else if mins > 0 {
        format!("{}m {}s", mins, secs)
    } else {
        format!("{}s", secs)
    };

    let row = |label: &str, val: &str| {
        let l = ansi_fg(muted_col, &format!("{:>18}", label.to_uppercase()));
        let v = ansi_bold_fg(txt_col, &format!("{:<18}", val));
        format!("{}  {}", l, v)
    };

    let streak_str = format!("{} Days", info.streak);

    let fill_blocks = info.focus_score.min(10);
    let empty_blocks = 10 - fill_blocks;
    let score_val = format!(
        "{}{} {}/10",
        "■".repeat(fill_blocks),
        "□".repeat(empty_blocks),
        info.focus_score
    );

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(center_text(&title, text_w));
    lines.push(center_text(&tone, text_w));
    lines.push(String::new());
    lines.push(center_text(&row("Mode", mode_str), text_w));
    lines.push(center_text(&row("Time Focused", &time_str), text_w));
    lines.push(center_text(
        &row("Segments Done", &format!("{} Completed", info.segments)),
        text_w,
    ));
    lines.push(center_text(&row("Breaks", &info.breaks.to_string()), text_w));
    lines.push(center_text(&row("Pauses Taken", &info.pauses.to_string()), text_w));
    lines.push(center_text(&row("Daily Streak", &streak_str), text_w));
    lines.push(center_text(&row("Focus Score", &score_val), text_w));
    lines.push(String::new());
    lines.push(center_text(
        &ansi_fg(muted_col, "Press [Enter] to dismiss"),
        text_w,
    ));
    lines.push(String::new());

    let content = lines.join("\n");
    let boxed = render_box(&content, border_col, BorderStyle::Rounded, 4, 1);
    place_center(width, height, &boxed)
}

fn recap_tone(info: &RecapInfo) -> &'static str {
    if info.focus_score >= 9 {
        "Flawless focus. Deep work mastery unlocked."
    } else if info.focus_score >= 7 {
        "Solid, steady flow. Great progress made."
    } else if info.focus_score >= 5 {
        "Good effort. Rest up before your next sprint."
    } else {
        "Session closed. Reflect, recharge, and return fresh."
    }
}

