// Help overlay modal screen.

use crate::render::bigclock::ansi_bold_fg;
use crate::render::borders::{render_box, BorderStyle};
use crate::render::text::visible_width;
use crate::render::widgets::place_center;
use crate::theme::Theme;

#[derive(Debug, Clone)]
pub struct HelpBinding {
    pub keys: &'static str,
    pub description: &'static str,
}

pub fn render_help(width: usize, height: usize, th: &Theme, bindings: &[HelpBinding]) -> String {
    let accent = th.accent();
    let _muted = th.muted();

    let mut rows = Vec::new();
    for b in bindings {
        let key_str = ansi_bold_fg(accent, b.keys);
        let pad_len = if visible_width(b.keys) < 14 {
            14 - visible_width(b.keys)
        } else {
            1
        };
        rows.push(format!("{}{}{}", key_str, " ".repeat(pad_len), b.description));
    }

    rows.push(String::new());
    rows.push(format!(
        "\x1b[38;2;{};{};{}mSessions auto-restore after an unexpected close.\x1b[0m",
        140, 140, 140
    ));
    rows.push(format!(
        "\x1b[38;2;{};{};{}mPress ? or Esc to close this overlay.\x1b[0m",
        140, 140, 140
    ));

    let content = rows.join("\n");
    let boxed = render_box(&content, accent, BorderStyle::Rounded, 3, 1);
    place_center(width, height, &boxed)
}

