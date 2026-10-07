// Big clock ASCII art digits (5 rows tall).

pub const BIG_DIGITS: [[&str; 5]; 10] = [
    [" ██ ", "█  █", "█  █", "█  █", " ██ "], // 0
    ["  █ ", "  █ ", "  █ ", "  █ ", "  █ "], // 1
    [" ██ ", "   █", " ██ ", "█   ", "████"], // 2
    [" ██ ", "   █", " ██ ", "   █", " ██ "], // 3
    ["█  █", "█  █", "████", "   █", "   █"], // 4
    ["████", "█   ", "███ ", "   █", "███ "], // 5
    [" ██ ", "█   ", "███ ", "█  █", " ██ "], // 6
    ["████", "  █ ", " █  ", " █  ", " █  "], // 7
    [" ██ ", "█  █", " ██ ", "█  █", " ██ "], // 8
    [" ██ ", "█  █", " ███", "   █", " ██ "], // 9
];

pub const BIG_COLON: [&str; 5] = ["  ", " ●", "  ", " ●", "  "];

pub fn parse_hex_color(hex: &str) -> (u8, u8, u8) {
    let clean = hex.trim().trim_start_matches('#');
    if clean.len() == 6 {
        if let Ok(val) = u32::from_str_radix(clean, 16) {
            return (
                ((val >> 16) & 0xff) as u8,
                ((val >> 8) & 0xff) as u8,
                (val & 0xff) as u8,
            );
        }
    }
    (200, 200, 200)
}

pub fn ansi_fg(hex: &str, text: &str) -> String {
    let (r, g, b) = parse_hex_color(hex);
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

pub fn ansi_bold_fg(hex: &str, text: &str) -> String {
    let (r, g, b) = parse_hex_color(hex);
    format!("\x1b[1;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

pub fn ansi_italic_fg(hex: &str, text: &str) -> String {
    let (r, g, b) = parse_hex_color(hex);
    format!("\x1b[3;38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

pub fn big_clock_rows(time_str: &str, color_hex: &str) -> Vec<String> {
    let mut rows = vec![String::new(); 5];
    let mut first = true;

    for ch in time_str.chars() {
        if !first {
            for row in &mut rows {
                row.push(' ');
            }
        }
        first = false;

        if ch.is_ascii_digit() {
            let d = (ch as u8 - b'0') as usize;
            for (i, row) in rows.iter_mut().enumerate() {
                row.push_str(BIG_DIGITS[d][i]);
            }
        } else if ch == ':' {
            for (i, row) in rows.iter_mut().enumerate() {
                row.push_str(BIG_COLON[i]);
            }
        }
    }

    rows.into_iter()
        .map(|r| ansi_bold_fg(color_hex, &r))
        .collect()
}

