// Session restoration prompt screen.

use crate::render::bigclock::ansi_bold_fg;
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::place_center;
use crate::theme::Theme;

pub fn render_restore_prompt(width: usize, height: usize, th: &Theme) -> String {
    let color = th.accent();
    let _muted = th.muted();

    let rows = [ansi_bold_fg(color, "Restore previous session?"),
        String::new(),
        format!("\x1b[38;2;{};{};{}my resume  ·  n discard  ·  q quit\x1b[0m", 140, 140, 140)];

    let content = rows.join("\n");
    let boxed = render_box(&content, color, BorderStyle::Rounded, 4, 1);
    place_center(width, height, &boxed)
}

