// Reusable terminal widgets: progress bar, session dots, alignment.

use crate::render::bigclock::ansi_fg;
use crate::render::text::visible_width;

pub fn session_dots(
    completed: usize,
    total: usize,
    filled_color: &str,
    empty_color: &str,
) -> String {
    let mut out = String::new();
    for i in 0..total {
        if i > 0 {
            out.push(' ');
        }
        if i < completed {
            out.push_str(&ansi_fg(filled_color, "●"));
        } else {
            out.push_str(&ansi_fg(empty_color, "○"));
        }
    }
    out
}

pub fn progress_bar(
    progress: f64,
    width: usize,
    filled_color: &str,
    empty_color: &str,
) -> String {
    if width == 0 {
        return String::new();
    }
    let n = (width as f64 * progress.clamp(0.0, 1.0)).round() as usize;
    let n = n.min(width);

    let filled = "━".repeat(n);
    let empty = "─".repeat(width - n);

    format!("{}{}", ansi_fg(filled_color, &filled), ansi_fg(empty_color, &empty))
}

pub fn retro_progress_bar(
    progress: f64,
    width: usize,
    filled_color: &str,
    empty_color: &str,
) -> String {
    if width == 0 {
        return String::new();
    }
    let n = (width as f64 * progress.clamp(0.0, 1.0)).round() as usize;
    let n = n.min(width);

    let filled = "▓".repeat(n);
    let empty = "▒".repeat(width - n);

    format!("{}{}", ansi_fg(filled_color, &filled), ansi_fg(empty_color, &empty))
}

/// Joins hint items with a separator, dropping trailing items (and finally
/// tightening the separator) until the line fits in `width` columns.
pub fn fit_hints(items: &[&str], width: usize) -> String {
    for sep in ["  ·  ", " · "] {
        for n in (1..=items.len()).rev() {
            let line = items[..n].join(sep);
            if visible_width(&line) <= width {
                return line;
            }
        }
    }
    String::new()
}

pub fn center_text(s: &str, width: usize) -> String {
    let vis_w = visible_width(s);
    if vis_w >= width {
        return s.to_string();
    }
    let pad_left = (width - vis_w) / 2;
    let pad_right = width - vis_w - pad_left;
    format!("{}{}{}", " ".repeat(pad_left), s, " ".repeat(pad_right))
}

pub fn pad_line(s: &str, width: usize) -> String {
    let vis_w = visible_width(s);
    if vis_w >= width {
        return s.to_string();
    }
    format!("{}{}", s, " ".repeat(width - vis_w))
}

pub fn place_center(width: usize, height: usize, content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let content_height = lines.len();
    if content_height == 0 {
        return String::new();
    }

    let top_pad = if height > content_height {
        (height - content_height) / 2
    } else {
        0
    };

    let bottom_pad = if height > content_height + top_pad {
        height - content_height - top_pad
    } else {
        0
    };

    let mut out = Vec::new();
    for _ in 0..top_pad {
        out.push(" ".repeat(width));
    }

    for line in lines {
        let vis_w = visible_width(line);
        let left_pad = if width > vis_w { (width - vis_w) / 2 } else { 0 };
        let right_pad = if width > vis_w + left_pad {
            width - vis_w - left_pad
        } else {
            0
        };
        out.push(format!("{}{}{}", " ".repeat(left_pad), line, " ".repeat(right_pad)));
    }

    for _ in 0..bottom_pad {
        out.push(" ".repeat(width));
    }

    out.join("\n")
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fit_hints_drops_items_to_fit() {
        let items = ["s start", "space pause", "q quit"];
        assert_eq!(fit_hints(&items, 80), "s start  ·  space pause  ·  q quit");
        assert_eq!(fit_hints(&items, 30), "s start  ·  space pause");
        assert_eq!(fit_hints(&items, 7), "s start");
        assert_eq!(fit_hints(&items, 3), "");
    }
}
