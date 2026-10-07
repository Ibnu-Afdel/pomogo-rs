// Retro layout: double-border frame with textured character progress bar (▓▒).

use crate::render::bigclock::{ansi_bold_fg, ansi_fg, ansi_italic_fg, big_clock_rows};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::{center_text, fit_hints, place_center, retro_progress_bar, session_dots};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn retro(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt_col = th.text();

    let text_w = (if f.width > 12 { f.width - 12 } else { 34 }).clamp(34, 68);

    let clock_str = format_clock(ds);
    let clock_rows = big_clock_rows(&clock_str, color);

    let label = ansi_bold_fg(color, &ds.mode_label.to_uppercase());
    let dots = session_dots(ds.segment_index, ds.segment_count, color, muted);
    let status = ansi_fg(muted, &ds.status_message.to_uppercase());

    let bar = retro_progress_bar(ds.progress, text_w, th.progress_fill(), th.progress_track());

    let hints = ansi_fg(
        muted,
        &fit_hints(
            &["S START", "SPACE PAUSE", "N SKIP", "T TASK", "P PROJECT", "R RESET", "? HELP", "Q QUIT"],
            text_w,
        ),
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
        let p_line = ansi_bold_fg(txt_col, &format!("PROJECT: {}", ds.project.to_uppercase()));
        lines.push(center_text(&p_line, text_w));
    }

    if !ds.task.is_empty() {
        let t_line = ansi_italic_fg(txt_col, &format!("TASK: {}", ds.task.to_uppercase()));
        lines.push(center_text(&t_line, text_w));
    }

    if !ds.git_branch.is_empty() && !ds.zen {
        let g_line = ansi_fg(muted, &format!(" {}", ds.git_branch.to_uppercase()));
        lines.push(center_text(&g_line, text_w));
    }

    if !ds.tmux_session.is_empty() && !ds.zen {
        let tm_line = ansi_fg(muted, &format!("TMUX:{}", ds.tmux_session.to_uppercase()));
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
    let boxed = render_box(&content, th.border(), BorderStyle::Double, 4, 0);
    place_center(f.width, f.height, &boxed)
}

