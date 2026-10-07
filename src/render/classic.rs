// Classic layout: rounded border box, big clock digits, dots, progress bar, hints.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg, ansi_italic_fg, big_clock_rows};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::{center_text, place_center, progress_bar, session_dots};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn classic(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt_col = th.text();

    let mut text_w = if f.width > 12 { f.width - 12 } else { 34 };
    if text_w > 68 {
        text_w = 68;
    }
    if text_w < 34 {
        text_w = 34;
    }

    let clock_str = format_clock(ds);
    let clock_rows = big_clock_rows(&clock_str, color);

    let label = ansi_bold_fg(color, &ds.mode_label);
    let dots = session_dots(ds.segment_index, ds.segment_count, color, muted);
    let status = ansi_fg(muted, &ds.status_message);

    let bar = progress_bar(ds.progress, text_w, th.progress_fill(), th.progress_track());

    let hints = ansi_fg(
        muted,
        "s start  ·  space pause  ·  n skip  ·  t task  ·  p project  ·  r reset  ·  ? help  ·  q quit",
    );

    let mut lines = Vec::new();
    lines.push(String::new());
    for row in clock_rows {
        lines.push(center_text(&row, text_w));
    }
    lines.push(String::new());
    lines.push(center_text(&label, text_w));

    if ds.segment_count > 0 && !ds.zen {
        lines.push(center_text(&dots, text_w));
    }

    if !ds.project.is_empty() {
        let p_line = ansi_bold_fg(txt_col, &format!("Project: {}", ds.project));
        lines.push(center_text(&p_line, text_w));
    }

    if !ds.task.is_empty() {
        let t_line = ansi_italic_fg(txt_col, &format!("Task: {}", ds.task));
        lines.push(center_text(&t_line, text_w));
    }

    if !ds.git_branch.is_empty() && !ds.zen {
        let g_line = ansi_fg(muted, &format!(" {}", ds.git_branch));
        lines.push(center_text(&g_line, text_w));
    }

    if !ds.tmux_session.is_empty() && !ds.zen {
        let tm_line = ansi_fg(muted, &format!("tmux:{}", ds.tmux_session));
        lines.push(center_text(&tm_line, text_w));
    }

    if !ds.zen {
        lines.push(center_text(&status, text_w));
    }

    lines.push(String::new());
    lines.push(bar);

    if !ds.zen {
        lines.push(String::new());
        lines.push(center_text(&hints, text_w));
    }
    lines.push(String::new());

    let content = lines.join("\n");
    let boxed = render_box(&content, th.border(), BorderStyle::Rounded, 4, 0);
    place_center(f.width, f.height, &boxed)
}

