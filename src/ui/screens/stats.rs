// Statistics overlay screen.

use chrono::Local;
use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::{center_text, place_center};
use crate::stats::{DayStats, Stats};
use crate::store::models::DbSession;
use crate::theme::Theme;

pub fn render_stats(
    width: usize,
    height: usize,
    th: &Theme,
    s: &Stats,
    status_msg: &str,
    recent: &[DbSession],
) -> String {
    let accent_col = th.accent();
    let muted_col = th.muted();
    let border_col = th.border();
    let prog_fill = th.progress_fill();
    let prog_track = th.progress_track();
    let txt_col = th.text();

    let mut text_w = if width > 12 { width - 12 } else { 34 };
    if text_w > 68 {
        text_w = 68;
    }
    if text_w < 34 {
        text_w = 34;
    }

    let today_str = format!("Today: {} sessions ({} mins focused)", s.today_count, s.today_minutes);
    let streak_str = format!("Streak: {} days (Best: {} days)", s.current_streak, s.best_streak);
    let month_str = format!(
        "This Month: {} sessions (Rate: {:.0}%)",
        s.month_count,
        s.completion_rate * 100.0
    );
    let life_hrs = s.lifetime_minutes / 60;
    let life_mins = s.lifetime_minutes % 60;
    let life_str = format!(
        "Lifetime: {} sessions ({}h {}m focused)",
        s.lifetime_sessions, life_hrs, life_mins
    );

    let graph_title = ansi_bold_fg(accent_col, "Weekly Focus Activity (mins)");
    let graph = week_bar_graph(&s.week_days, prog_fill, prog_track, muted_col);

    let recent_title = ansi_bold_fg(accent_col, "Recent Work Sessions");
    let mut recent_lines = Vec::new();

    if recent.is_empty() {
        recent_lines.push(ansi_fg(muted_col, "No sessions recorded yet"));
    } else {
        for r in recent.iter().rev().take(5) {
            let time_str = r.started_at.with_timezone(&Local).format("%H:%M").to_string();
            let task_str = r.task.as_deref().unwrap_or("[no task]");
            let (status, status_col) = if r.completed {
                ("completed", prog_fill)
            } else {
                ("skipped", muted_col)
            };

            let mut line = format!(
                "{}  {} ({})",
                ansi_fg(muted_col, &time_str),
                ansi_bold_fg(txt_col, task_str),
                ansi_fg(status_col, status)
            );
            if let Some(note) = &r.note {
                if !note.is_empty() {
                    line.push_str(&format!(" - {}", ansi_fg(muted_col, note)));
                }
            }
            recent_lines.push(line);
        }
    }

    let hints = if !status_msg.is_empty() {
        ansi_fg(accent_col, status_msg)
    } else {
        ansi_fg(muted_col, "Tab timer  ·  y yank stats  ·  ? help  ·  q quit")
    };

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(center_text(&ansi_bold_fg(accent_col, "Focus Statistics"), text_w));
    lines.push(String::new());
    lines.push(center_text(&today_str, text_w));
    lines.push(center_text(&streak_str, text_w));
    lines.push(center_text(&month_str, text_w));
    lines.push(center_text(&life_str, text_w));
    lines.push(String::new());
    lines.push(center_text(&graph_title, text_w));

    for g_line in graph {
        lines.push(center_text(&g_line, text_w));
    }
    lines.push(String::new());

    lines.push(center_text(&recent_title, text_w));
    for r_line in recent_lines {
        lines.push(center_text(&r_line, text_w));
    }
    lines.push(String::new());
    lines.push(center_text(&hints, text_w));
    lines.push(String::new());

    let content = lines.join("\n");
    let boxed = render_box(&content, border_col, BorderStyle::Rounded, 3, 0);
    place_center(width, height, &boxed)
}

fn week_bar_graph(
    days: &[DayStats; 7],
    fill_col: &str,
    _track_col: &str,
    muted_col: &str,
) -> Vec<String> {
    let mut out = Vec::new();
    for d in days {
        let day_label = d.date.format("%a").to_string();
        let bar_len = (d.minutes / 10).min(15);
        let bar_str = if bar_len > 0 {
            "█".repeat(bar_len)
        } else {
            "░".to_string()
        };

        let colored_bar = if bar_len > 0 {
            ansi_fg(fill_col, &bar_str)
        } else {
            ansi_fg(muted_col, &bar_str)
        };

        let _pad_bar = format!("{:<15}", bar_str);
        let line = format!("{}  {} {}m", day_label, colored_bar, d.minutes);
        out.push(line);
    }
    out
}

