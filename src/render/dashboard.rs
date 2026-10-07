// Dashboard layout: denser operational view with split columns.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::text::{pad_right, truncate_text, visible_width};
use crate::render::widgets::{place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn dashboard(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt = th.text();
    let border = th.border();

    let mut width = if f.width > 10 { f.width - 10 } else { 50 };
    if width > 76 {
        width = 76;
    }
    if width < 50 {
        width = 50;
    }

    let left_w = width / 2 - 2;
    let right_w = width - left_w - 3;

    let timer_text = format_clock(ds);
    let title = ansi_bold_fg(color, &ds.mode_label.to_uppercase());
    let clock = ansi_bold_fg(color, &timer_text);
    let bar = progress_bar(ds.progress, left_w, th.progress_fill(), th.progress_track());

    let mut meta = vec![
        meta_row("project", &ds.project, left_w, muted, txt),
        meta_row("task", &ds.task, left_w, muted, txt),
        meta_row("theme", &ds.theme_name, left_w, muted, txt),
    ];
    if !ds.zen {
        meta.push(meta_row("status", &ds.status_message, left_w, muted, txt));
    }

    let block_lines = vec![
        ansi_fg(muted, "FOCUS"),
        title,
        clock,
        String::new(),
        bar,
    ];

    // Combine left and right columns with divider
    let max_lines = block_lines.len().max(meta.len());
    let mut combined_lines = Vec::new();

    for i in 0..max_lines {
        let left_part = if i < block_lines.len() {
            &block_lines[i]
        } else {
            ""
        };
        let right_part = if i < meta.len() { &meta[i] } else { "" };

        let vis_left = visible_width(left_part);
        let pad_l = if left_w >= vis_left { left_w - vis_left } else { 0 };

        let vis_right = visible_width(right_part);
        let pad_r = if right_w >= vis_right { right_w - vis_right } else { 0 };

        let row = format!(
            "{}{}{}{}{}{}",
            left_part,
            " ".repeat(pad_l),
            ansi_fg(border, " │ "),
            right_part,
            " ".repeat(pad_r),
            ""
        );
        combined_lines.push(row);
    }

    let content = combined_lines.join("\n");
    let boxed = render_box(&content, border, BorderStyle::Normal, 2, 1);
    place_center(f.width, f.height, &boxed)
}

fn meta_row(label: &str, val: &str, width: usize, muted: &str, txt: &str) -> String {
    let clean_val = if val.is_empty() { "none" } else { val };
    let avail_w = if width > 11 { width - 11 } else { 8 };
    let trunc_val = truncate_text(clean_val, avail_w, "…");

    let l = ansi_fg(muted, &pad_right(&label.to_uppercase(), 9));
    let v = ansi_fg(txt, &trunc_val);
    format!("{} {}", l, v)
}

