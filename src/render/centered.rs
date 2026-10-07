// Centered layout: huge central clock digits, whisper-quiet contextual headers.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg, big_clock_rows};
use crate::render::widgets::{center_text, place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn centered(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt_col = th.text();

    let mut text_w = if f.width > 6 { f.width - 6 } else { 40 };
    if text_w > 70 {
        text_w = 70;
    }
    if text_w < 40 {
        text_w = 40;
    }

    let clock_str = format_clock(ds);
    let clock_rows = big_clock_rows(&clock_str, color);

    let mut context_parts = vec![ds.mode_label.to_uppercase()];
    if !ds.project.is_empty() {
        context_parts.push(ds.project.to_uppercase());
    }
    let header_text = center_text(
        &ansi_bold_fg(muted, &context_parts.join("  ·  ")),
        text_w,
    );

    let task_line = if !ds.task.is_empty() {
        center_text(&ansi_fg(txt_col, &ds.task), text_w)
    } else {
        String::new()
    };

    let status = center_text(&ansi_fg(muted, &ds.status_message), text_w);
    let bar = progress_bar(ds.progress, text_w, th.progress_fill(), th.progress_track());

    let mut lines = Vec::new();
    lines.push(String::new());
    if !ds.zen {
        lines.push(header_text);
    } else if !ds.project.is_empty() {
        lines.push(center_text(
            &ansi_bold_fg(muted, &ds.project.to_uppercase()),
            text_w,
        ));
    }
    lines.push(String::new());

    for row in clock_rows {
        lines.push(center_text(&row, text_w));
    }
    lines.push(String::new());

    if !task_line.is_empty() {
        lines.push(task_line);
    }
    if !ds.zen {
        lines.push(status);
    }
    lines.push(String::new());
    lines.push(bar);
    lines.push(String::new());

    place_center(f.width, f.height, &lines.join("\n"))
}

