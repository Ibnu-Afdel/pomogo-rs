// Text measurement, padding, and truncation using unicode-width.

use unicode_width::UnicodeWidthStr;

pub fn text_width(s: &str) -> usize {
    s.width()
}

pub fn pad_right(s: &str, width: usize) -> String {
    let w = text_width(s);
    if w >= width {
        return s.to_string();
    }
    let spaces = " ".repeat(width - w);
    format!("{}{}", s, spaces)
}

pub fn truncate_text(s: &str, width: usize, tail: &str) -> String {
    if width == 0 {
        return String::new();
    }
    if text_width(s) <= width {
        return s.to_string();
    }

    let tail_w = text_width(tail);
    if tail_w >= width {
        let mut cur = String::new();
        for ch in tail.chars() {
            if text_width(&cur) + ch.len_utf8() <= width {
                cur.push(ch);
            } else {
                break;
            }
        }
        return cur;
    }

    let target_w = width - tail_w;
    let mut cur = String::new();
    for ch in s.chars() {
        let next_w = text_width(&cur) + unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if next_w <= target_w {
            cur.push(ch);
        } else {
            break;
        }
    }
    format!("{}{}", cur, tail)
}

pub fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut in_escape = false;

    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            if ch == 'm' || ch == 'K' || ch == 'H' || ch == 'J' {
                in_escape = false;
            }
            continue;
        }
        out.push(ch);
    }
    out
}

pub fn visible_width(s: &str) -> usize {
    strip_ansi(s).width()
}

