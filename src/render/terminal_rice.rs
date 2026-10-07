// Terminal-rice layout: styled frame with decorative headers and thick borders.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg, big_clock_rows};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::text::truncate_text;
use crate::render::widgets::{center_text, place_center, progress_bar};
use crate::render::{format_clock, phase_color, DisplayState, Frame};
use crate::theme::Theme;

pub fn terminal_rice(ds: &DisplayState, th: &Theme, f: &Frame) -> String {
    let color = phase_color(ds.phase_kind, th);
    let muted = th.muted();
    let border = th.border();
    let txt = th.text();

    let width = (if f.width > 10 { f.width - 10 } else { 46 }).clamp(46, 72);

    let rail = ansi_fg(border, &"─".repeat(width));
    let header = center_text(
        &ansi_fg(muted, &format!("✦ {} ✦", ds.mode_label.to_uppercase())),
        width,
    );

    let mut body = vec![rail.clone(), header, String::new()];

    let clock_str = format_clock(ds);
    for row in big_clock_rows(&clock_str, color) {
        body.push(center_text(&row, width));
    }
    body.push(String::new());

    if !ds.project.is_empty() {
        let p = truncate_text(&ds.project, width, "…");
        body.push(center_text(&ansi_bold_fg(txt, &p), width));
    }

    if !ds.task.is_empty() && !ds.zen {
        let t = truncate_text(&ds.task, width, "…");
        body.push(center_text(&ansi_fg(muted, &t), width));
    }

    body.push(String::new());
    body.push(progress_bar(ds.progress, width, th.progress_fill(), th.progress_track()));
    body.push(rail);

    let content = body.join("\n");
    let boxed = render_box(&content, border, BorderStyle::Thick, 2, 0);
    place_center(f.width, f.height, &boxed)
}

