// Focus-stack layout: compact stacked layout for repeated work sessions.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::text::{text_width, truncate_text};
use crate::render::widgets::{place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn focus_stack(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt = th.text();
    let border = th.border();

    let mut width = if f.width > 8 { f.width - 8 } else { 44 };
    if width > 72 {
        width = 72;
    }
    if width < 44 {
        width = 44;
    }

    let clock_str = format_clock(ds);
    let left = ansi_bold_fg(color, &clock_str);
    let right = ansi_fg(muted, &ds.mode_label.to_uppercase());

    let header_gap = if width > text_width(&clock_str) + text_width(&ds.mode_label) {
        width - text_width(&clock_str) - text_width(&ds.mode_label)
    } else {
        1
    };
    let header = format!("{}{}{}", left, " ".repeat(header_gap), right);

    let mut ctx_parts = Vec::new();
    if !ds.project.is_empty() {
        ctx_parts.push(ds.project.as_str());
    }
    if !ds.task.is_empty() {
        ctx_parts.push(ds.task.as_str());
    }
    let mut context = ctx_parts.join("  /  ");
    if context.is_empty() {
        context = ds.status_message.clone();
    }
    let trunc_ctx = truncate_text(&context, width, "…");

    let bar = progress_bar(ds.progress, width, th.progress_fill(), th.progress_track());
    let rail = ansi_fg(border, &"─".repeat(width));

    let lines = if ds.zen {
        vec![
            header,
            rail,
            ansi_fg(txt, &trunc_ctx),
            String::new(),
            bar,
        ]
    } else {
        vec![
            header,
            rail,
            ansi_fg(txt, &trunc_ctx),
            ansi_fg(muted, &truncate_text(&ds.status_message, width, "…")),
            String::new(),
            bar,
        ]
    };

    let content = lines.join("\n");
    let boxed = render_box(&content, border, BorderStyle::Normal, 2, 1);
    place_center(f.width, f.height, &boxed)
}

