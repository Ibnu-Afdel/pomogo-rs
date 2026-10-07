// Compact layout: short timer widget designed for split-screen layouts.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::text::{text_width, truncate_text};
use crate::render::widgets::{center_text, place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn compact(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt_col = th.text();

    let text_w = (if f.width > 4 { f.width - 4 } else { 30 }).clamp(30, 60);

    let clock_str = format_clock(ds);
    let timer_disp = ansi_bold_fg(color, &clock_str);

    let mut detail_parts = Vec::new();
    if !ds.project.is_empty() {
        detail_parts.push(ds.project.clone());
    }
    if !ds.task.is_empty() {
        detail_parts.push(ds.task.clone());
    }
    let details_str = detail_parts.join(" · ");
    let sep = "  |  ";

    // Drop the status first, then shorten the details, so the row never
    // spills past the widget width on narrow splits.
    let mut show_status = !ds.zen;
    let fixed = |status: bool| {
        text_width(&clock_str)
            + if status { sep.len() + text_width(&ds.status_message) } else { 0 }
    };
    let detail_budget = |status: bool| text_w.saturating_sub(fixed(status) + sep.len());
    if show_status && !details_str.is_empty() && text_width(&details_str) > detail_budget(true) {
        show_status = false;
    }
    if show_status && fixed(true) > text_w {
        show_status = false;
    }
    let details_str = truncate_text(&details_str, detail_budget(show_status), "…");

    let mut status_parts = vec![timer_disp];
    if !details_str.is_empty() {
        status_parts.push(ansi_fg(txt_col, &details_str));
    }
    if show_status {
        status_parts.push(ansi_fg(muted, &ds.status_message));
    }

    let bar = progress_bar(ds.progress, text_w, th.progress_fill(), th.progress_track());

    let lines = [
        String::new(),
        center_text(&status_parts.join(sep), text_w),
        String::new(),
        bar,
        String::new(),
    ];

    place_center(f.width, f.height, &lines.join("\n"))
}

