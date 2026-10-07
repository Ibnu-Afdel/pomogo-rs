// Tinybar layout: narrow 2-row strip for terminal edges and small tmux panes.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::text::{text_width, truncate_text};
use crate::render::widgets::{place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn tinybar(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt = th.text();

    let width = (if f.width > 4 { f.width - 4 } else { 36 }).clamp(36, 90);

    let clock_str = format_clock(ds);
    let left = ansi_bold_fg(color, &clock_str);
    let label = ansi_fg(muted, &ds.mode_label.to_uppercase());

    let mut context = ds.project.clone();
    if !ds.task.is_empty() {
        if !context.is_empty() {
            context.push_str(" · ");
        }
        context.push_str(&ds.task);
    }

    let overhead = text_width(&clock_str) + text_width(&ds.mode_label) + 8;
    let avail = if width > overhead { width - overhead } else { 10 };
    let trunc_ctx = truncate_text(&context, avail, "…");
    let right = ansi_fg(txt, &trunc_ctx);

    let mut line = format!("{}  {}", left, label);
    if !context.is_empty() {
        line.push_str(&format!("  {}  {}", ansi_fg(muted, "│"), right));
    }

    let bar = progress_bar(ds.progress, width, th.progress_fill(), th.progress_track());
    let content = format!("{}\n{}", line, bar);
    place_center(f.width, f.height, &content)
}

