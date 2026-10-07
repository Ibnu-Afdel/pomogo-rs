// Text input overlay screen (for task, note, project, and custom duration inputs).

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::place_center;
use crate::theme::Theme;

pub fn render_input(
    width: usize,
    height: usize,
    th: &Theme,
    mode: &str,
    input_text: &str,
    suggestions: &[String],
    suggestion_index: i32,
) -> String {
    let (color, title) = match mode {
        "note" => (th.work(), "Session Note"),
        "project" => (th.accent(), "Set Project Name"),
        "custom_duration" => (th.accent(), "Set Custom Duration (e.g. 1h30m, 90m)"),
        _ => (th.accent(), "Set Current Task"),
    };

    let mut rows = Vec::new();
    rows.push(ansi_bold_fg(color, title));
    rows.push(String::new());

    // Input cursor field
    rows.push(format!("> {}█", input_text));
    rows.push(String::new());

    if (mode == "task" || mode == "project") && !suggestions.is_empty() {
        rows.push(ansi_bold_fg(color, "Suggestions:"));
        for (i, s) in suggestions.iter().take(5).enumerate() {
            let indicator = if i as i32 == suggestion_index { "> " } else { "  " };
            let item = if i as i32 == suggestion_index {
                ansi_bold_fg(color, s)
            } else {
                ansi_fg(th.muted(), s)
            };
            rows.push(format!("{}{}", indicator, item));
        }
        rows.push(String::new());
    }

    let footer = if (mode == "task" || mode == "project") && !suggestions.is_empty() {
        "↓/↑ navigate  ·  tab select  ·  ctrl+d delete  ·  enter save"
    } else {
        "enter save  ·  esc cancel"
    };
    rows.push(ansi_fg(th.muted(), footer));

    let content = rows.join("\n");
    let boxed = render_box(&content, color, BorderStyle::Rounded, 4, 1);
    place_center(width, height, &boxed)
}

