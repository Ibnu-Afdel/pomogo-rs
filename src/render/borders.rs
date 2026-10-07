// Terminal box framing with Rounded, Normal, Thick, and Double border styles.

use crate::render::bigclock::ansi_fg;
use crate::render::text::visible_width;

#[derive(Debug, Clone, Copy)]
pub enum BorderStyle {
    Rounded,
    Normal,
    Thick,
    Double,
}

pub struct BorderChars {
    pub top_left: &'static str,
    pub top_right: &'static str,
    pub bottom_left: &'static str,
    pub bottom_right: &'static str,
    pub horizontal: &'static str,
    pub vertical: &'static str,
}

impl BorderStyle {
    pub fn chars(&self) -> BorderChars {
        match self {
            BorderStyle::Rounded => BorderChars {
                top_left: "╭",
                top_right: "╮",
                bottom_left: "╰",
                bottom_right: "╯",
                horizontal: "─",
                vertical: "│",
            },
            BorderStyle::Normal => BorderChars {
                top_left: "┌",
                top_right: "┐",
                bottom_left: "└",
                bottom_right: "┘",
                horizontal: "─",
                vertical: "│",
            },
            BorderStyle::Thick => BorderChars {
                top_left: "┏",
                top_right: "┓",
                bottom_left: "┗",
                bottom_right: "┛",
                horizontal: "━",
                vertical: "┃",
            },
            BorderStyle::Double => BorderChars {
                top_left: "╔",
                top_right: "╗",
                bottom_left: "╚",
                bottom_right: "╝",
                horizontal: "═",
                vertical: "║",
            },
        }
    }
}

pub fn render_box(
    content: &str,
    border_color: &str,
    style: BorderStyle,
    padding_h: usize,
    padding_v: usize,
) -> String {
    let chars = style.chars();
    let lines: Vec<&str> = content.lines().collect();

    let mut max_w = 0;
    for l in &lines {
        let w = visible_width(l);
        if w > max_w {
            max_w = w;
        }
    }

    let inner_w = max_w + 2 * padding_h;
    let top_border = format!(
        "{}{}{}",
        chars.top_left,
        chars.horizontal.repeat(inner_w),
        chars.top_right
    );
    let bottom_border = format!(
        "{}{}{}",
        chars.bottom_left,
        chars.horizontal.repeat(inner_w),
        chars.bottom_right
    );

    let v_border = ansi_fg(border_color, chars.vertical);

    let mut out = Vec::new();
    out.push(ansi_fg(border_color, &top_border));

    for _ in 0..padding_v {
        out.push(format!("{}{}{}", v_border, " ".repeat(inner_w), v_border));
    }

    for line in lines {
        let vis_w = visible_width(line);
        let right_pad = if max_w >= vis_w { max_w - vis_w } else { 0 };
        let padded_line = format!(
            "{}{}{}{}{}",
            v_border,
            " ".repeat(padding_h),
            line,
            " ".repeat(right_pad + padding_h),
            v_border
        );
        out.push(padded_line);
    }

    for _ in 0..padding_v {
        out.push(format!("{}{}{}", v_border, " ".repeat(inner_w), v_border));
    }

    out.push(ansi_fg(border_color, &bottom_border));
    out.join("\n")
}

