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

/// Cuts an ANSI-styled line down to `width` visible columns, keeping every
/// escape sequence so colors stay balanced.
pub fn clip_visible(s: &str, width: usize) -> String {
    if visible_width(s) <= width {
        return s.to_string();
    }
    let mut out = String::new();
    let mut used = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_escape = true;
            out.push(ch);
            continue;
        }
        if in_escape {
            out.push(ch);
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
            continue;
        }
        let w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + w > width {
            continue;
        }
        used += w;
        out.push(ch);
    }
    out
}

pub fn visible_width(s: &str) -> usize {
    strip_ansi(s).width()
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_visible_keeps_escapes() {
        let s = "\x1b[31mhello\x1b[0m world";
        let clipped = clip_visible(s, 3);
        assert_eq!(strip_ansi(&clipped), "hel");
        assert!(clipped.contains("\x1b[0m"));
        assert_eq!(clip_visible("abc", 10), "abc");
        assert_eq!(visible_width(&clip_visible("│ wide 🍅 text", 8)), 8);
    }
}
