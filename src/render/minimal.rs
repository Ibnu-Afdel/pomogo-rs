// Minimal layout: clean borderless compact timer.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::widgets::{center_text, place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn minimal(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let txt_col = th.text();

    let text_w = (if f.width > 4 { f.width - 4 } else { 30 }).clamp(30, 60);

    let clock_str = format_clock(ds);
    let timer_disp = ansi_bold_fg(color, &clock_str);
    let label_disp = ansi_fg(muted, &format!(" · {}", ds.mode_label));
    let header = center_text(&format!("{}{}", timer_disp, label_disp), text_w);

    let details = if !ds.project.is_empty() && !ds.task.is_empty() {
        center_text(&ansi_fg(txt_col, &format!("{} → {}", ds.project, ds.task)), text_w)
    } else if !ds.project.is_empty() {
        center_text(&ansi_fg(txt_col, &ds.project), text_w)
    } else if !ds.task.is_empty() {
        center_text(&ansi_fg(txt_col, &ds.task), text_w)
    } else {
        String::new()
    };

    let bar = progress_bar(ds.progress, text_w, th.progress_fill(), th.progress_track());

    let mut lines = Vec::new();
    lines.push(String::new());
    lines.push(header);
    if !details.is_empty() {
        lines.push(details);
    }
    if !ds.zen {
        let status = center_text(&ansi_fg(muted, &ds.status_message), text_w);
        lines.push(status);
    }
    lines.push(String::new());
    lines.push(bar);
    lines.push(String::new());

    place_center(f.width, f.height, &lines.join("\n"))
}

