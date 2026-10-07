// Ambient background particle effects: stars, snow, rain, embers, scanlines.

use crate::render::bigclock::ansi_fg;
use crate::render::text::visible_width;
use crate::theme::Theme;

pub fn hash(x: i32, y: i32, seed: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(37)
        .wrapping_add((y as u32).wrapping_mul(17))
        .wrapping_add(seed as u32);
    h = (h ^ 61) ^ (h >> 16);
    h = h.wrapping_mul(9);
    h = (h ^ (h >> 4)).wrapping_mul(0x27d4eb2d);
    h = h ^ (h >> 15);
    h
}

pub fn render_ambient(
    effect: &str,
    tick_count: usize,
    width: usize,
    height: usize,
    th: &Theme,
    layout_content: &str,
) -> String {
    if effect == "none" || effect.is_empty() || width == 0 || height == 0 {
        return layout_content.to_string();
    }

    let star_dim = ansi_fg(th.ambient(), ".");
    let star_med = ansi_fg(th.muted(), "+");
    let star_bright = format!("\x1b[1m{}\x1b[0m", ansi_fg(th.muted(), "*"));

    let snow1 = ansi_fg(th.ambient(), ".");
    let snow2 = ansi_fg(th.muted(), "*");

    let rain1 = ansi_fg(th.ambient(), "│");
    let rain2 = ansi_fg(th.ambient(), "/");

    let ember1 = ansi_fg(th.accent(), ".");
    let ember2 = ansi_fg(th.work(), "·");

    let scan = ansi_fg(th.ambient(), "─");

    let mut bg_grid = vec![vec![" ".to_string(); width]; height];

    for (y, row) in bg_grid.iter_mut().enumerate() {
        for (x, cell) in row.iter_mut().enumerate() {
            let mut char_str = " ".to_string();
            match effect {
                "stars" => {
                    if hash(x as i32, y as i32, 42) % 100 < 3 {
                        let blink = (hash(x as i32, y as i32, 12) + tick_count as u32) % 4;
                        char_str = match blink {
                            0 => star_dim.clone(),
                            1 => star_med.clone(),
                            2 => star_bright.clone(),
                            _ => " ".to_string(),
                        };
                    }
                }
                "snow" => {
                    let offset = hash(x as i32, 0, 77);
                    let row = (offset + tick_count as u32) % (height as u32);
                    if y as u32 == row && hash(x as i32, 0, 88).is_multiple_of(4) {
                        char_str = if hash(x as i32, y as i32, 99).is_multiple_of(2) {
                            snow2.clone()
                        } else {
                            snow1.clone()
                        };
                    }
                }
                "rain" => {
                    let offset = hash(x as i32, 0, 11);
                    let row = (offset + (tick_count * 2) as u32) % (height as u32);
                    if y as u32 == row && hash(x as i32, 0, 22).is_multiple_of(3) {
                        char_str = if hash(x as i32, y as i32, 33).is_multiple_of(2) {
                            rain2.clone()
                        } else {
                            rain1.clone()
                        };
                    }
                }
                "embers" => {
                    let offset = hash(x as i32, 0, 121);
                    let h_safe = height.max(1) as u32;
                    let row = (h_safe + offset - (tick_count as u32 % h_safe)) % h_safe;
                    if y as u32 == row && hash(x as i32, y as i32, 131) % 100 < 7 {
                        char_str = if hash(x as i32, y as i32, 141).is_multiple_of(2) {
                            ember1.clone()
                        } else {
                            ember2.clone()
                        };
                    }
                }
                "scanline" => {
                    let h_safe = height.max(1);
                    let row = tick_count % h_safe;
                    if (y == row || y == (row + height / 2) % h_safe)
                        && hash(x as i32, y as i32, 151) % 100 < 65 {
                            char_str = scan.clone();
                        }
                }
                _ => {}
            }
            *cell = char_str;
        }
    }

    let content_lines: Vec<&str> = layout_content.lines().collect();
    let content_h = content_lines.len();
    let mut content_w = 0;
    for line in &content_lines {
        let w = visible_width(line);
        if w > content_w {
            content_w = w;
        }
    }

    let start_y = if height > content_h { (height - content_h) / 2 } else { 0 };
    let start_x = if width > content_w { (width - content_w) / 2 } else { 0 };

    let mut merged = Vec::with_capacity(height);
    for (y, bg_row) in bg_grid.iter().enumerate() {
        if y >= start_y && y < start_y + content_h {
            let line_idx = y - start_y;
            let line = content_lines[line_idx];

            let left_bg = bg_row[..start_x].concat();
            let overlay = overlay_line(&bg_row[start_x..start_x + content_w.min(width - start_x)], line);
            let right_start = (start_x + content_w).min(width);
            let right_bg = bg_row[right_start..].concat();

            merged.push(format!("{}{}{}", left_bg, overlay, right_bg));
        } else {
            merged.push(bg_row.concat());
        }
    }

    merged.join("\n")
}

fn overlay_line(bg: &[String], content: &str) -> String {
    // If content has spaces, show bg particle through them where appropriate
    let mut out = String::new();
    let mut bg_idx = 0;
    let mut in_escape = false;

    for ch in content.chars() {
        if ch == '\x1b' {
            in_escape = true;
            out.push(ch);
            continue;
        }
        if in_escape {
            out.push(ch);
            if ch == 'm' {
                in_escape = false;
            }
            continue;
        }

        if ch == ' ' {
            if bg_idx < bg.len() && !bg[bg_idx].trim().is_empty() {
                out.push_str(&bg[bg_idx]);
            } else {
                out.push(' ');
            }
        } else {
            out.push(ch);
        }
        bg_idx += 1;
    }

    out
}

