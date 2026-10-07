// Help overlay: key groups in two columns, or one when the terminal is narrow.

use crate::render::bigclock::{ansi_bold_fg, ansi_fg};
use crate::render::borders::{render_box, BorderStyle};
use crate::render::widgets::{pad_line, place_center};
use crate::theme::Theme;
use crate::ui::keymap::{KeyGroup, KEY_GROUPS};

const KEY_COL: usize = 7;
const COLUMN_WIDTH: usize = 34;

fn group_lines(group: &KeyGroup, th: &Theme) -> Vec<String> {
    let mut lines = vec![ansi_bold_fg(th.text(), group.title)];
    for (key, what) in group.keys {
        lines.push(format!(
            "{}{}",
            pad_line(&ansi_bold_fg(th.accent(), key), KEY_COL),
            ansi_fg(th.muted(), what)
        ));
    }
    lines.push(String::new());
    lines
}

pub fn render_help(width: usize, height: usize, th: &Theme) -> String {
    let (left, right) = KEY_GROUPS.split_at(2);
    let left: Vec<String> = left.iter().flat_map(|g| group_lines(g, th)).collect();
    let right: Vec<String> = right.iter().flat_map(|g| group_lines(g, th)).collect();

    let mut rows: Vec<String> = if width >= COLUMN_WIDTH * 2 + 10 {
        (0..left.len().max(right.len()))
            .map(|i| {
                let l = left.get(i).cloned().unwrap_or_default();
                let r = right.get(i).cloned().unwrap_or_default();
                format!("{}{}", pad_line(&l, COLUMN_WIDTH), r)
            })
            .collect()
    } else {
        left.into_iter().chain(right).collect()
    };
    while rows.last().is_some_and(|r| r.trim().is_empty()) {
        rows.pop();
    }
    rows.push(String::new());
    rows.push(ansi_fg(th.muted(), "Press ? or esc to close."));

    let boxed = render_box(&rows.join("\n"), th.border(), BorderStyle::Rounded, 3, 1);
    place_center(width, height, &boxed)
}
