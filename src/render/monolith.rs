// Monolith layout: timer as a large, bold standalone object.

use crate::render::bigclock::{ansi_fg, big_clock_rows};
use crate::render::text::truncate_text;
use crate::render::widgets::{center_text, place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn monolith(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt = th.text();

    let mut width = if f.width > 8 { f.width - 8 } else { 44 };
    if width > 78 {
        width = 78;
    }
    if width < 44 {
        width = 44;
    }

    let mut lines = Vec::new();
    if !ds.zen {
        lines.push(center_text(
            &ansi_fg(muted, &ds.mode_label.to_uppercase()),
            width,
        ));
        lines.push(String::new());
    }

    let clock_str = format_clock(ds);
    for row in big_clock_rows(&clock_str, color) {
        lines.push(center_text(&row, width));
    }
    lines.push(String::new());

    if !ds.project.is_empty() || !ds.task.is_empty() {
        let mut parts = Vec::new();
        if !ds.project.is_empty() {
            parts.push(ds.project.as_str());
        }
        if !ds.task.is_empty() {
            parts.push(ds.task.as_str());
        }
        let ctx = parts.join("  ");
        let trunc_ctx = truncate_text(&ctx, width, "…");
        lines.push(center_text(&ansi_fg(txt, &trunc_ctx), width));
    }

    if !ds.zen {
        lines.push(center_text(&ansi_fg(muted, &ds.status_message), width));
    }
    lines.push(String::new());
    lines.push(progress_bar(ds.progress, width, th.progress_fill(), th.progress_track()));

    place_center(f.width, f.height, &lines.join("\n"))
}

