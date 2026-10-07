// Command-center layout: split view with visible task and project context.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::text::{truncate_text, visible_width};
use crate::render::widgets::{place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn command_center(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt = th.text();
    let border = th.border();

    let mut width = if f.width > 8 { f.width - 8 } else { 62 };
    if width > 92 {
        width = 92;
    }
    if width < 62 {
        width = 62;
    }

    let left_w = width / 2;
    let right_w = width - left_w - 3;

    let timer_block = vec![
        ansi_fg(muted, &ds.mode_label.to_uppercase()),
        ansi_bold_fg(color, &format_clock(ds)),
        progress_bar(ds.progress, left_w, th.progress_fill(), th.progress_track()),
    ];

    let project = if ds.project.is_empty() {
        "No project".to_string()
    } else {
        ds.project.clone()
    };
    let task = if ds.task.is_empty() {
        "No task".to_string()
    } else {
        ds.task.clone()
    };
    let status = if ds.zen {
        String::new()
    } else {
        ds.status_message.clone()
    };

    let right_lines = vec![
        ansi_fg(muted, "PROJECT"),
        ansi_bold_fg(txt, &truncate_text(&project, right_w, "…")),
        String::new(),
        ansi_fg(muted, "TASK"),
        ansi_fg(txt, &truncate_text(&task, right_w, "…")),
        String::new(),
        ansi_fg(muted, &truncate_text(&status, right_w, "…")),
    ];

    let max_lines = timer_block.len().max(right_lines.len());
    let mut combined = Vec::new();

    for i in 0..max_lines {
        let left_part = if i < timer_block.len() { &timer_block[i] } else { "" };
        let right_part = if i < right_lines.len() { &right_lines[i] } else { "" };

        let vis_l = visible_width(left_part);
        let pad_l = if left_w >= vis_l { left_w - vis_l } else { 0 };

        let vis_r = visible_width(right_part);
        let pad_r = if right_w >= vis_r { right_w - vis_r } else { 0 };

        let row = format!(
            "{}{}{}{}{}{}",
            left_part,
            " ".repeat(pad_l),
            ansi_fg(border, " │ "),
            right_part,
            " ".repeat(pad_r),
            ""
        );
        combined.push(row);
    }

    let content = combined.join("\n");
    let boxed = render_box(&content, border, BorderStyle::Rounded, 2, 1);
    place_center(f.width, f.height, &boxed)
}

