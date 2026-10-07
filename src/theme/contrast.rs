// WCAG 2.1 relative luminance and contrast calculations.

use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl FromStr for Rgb {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let clean = s.trim().trim_start_matches('#');
        if clean.len() != 6 {
            return Err(format!("expected 6 hex digits, got {:?}", s));
        }
        let val = u32::from_str_radix(clean, 16)
            .map_err(|e| format!("parse error for {:?}: {}", s, e))?;

        Ok(Rgb {
            r: ((val >> 16) & 0xff) as u8,
            g: ((val >> 8) & 0xff) as u8,
            b: (val & 0xff) as u8,
        })
    }
}

pub fn relative_luminance(rgb: &Rgb) -> f64 {
    fn linear(c: u8) -> f64 {
        let v = (c as f64) / 255.0;
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    }

    0.2126 * linear(rgb.r) + 0.7152 * linear(rgb.g) + 0.0722 * linear(rgb.b)
}

pub fn contrast_ratio(fg: &str, bg: &str) -> Result<f64, String> {
    let fg_rgb: Rgb = fg.parse()?;
    let bg_rgb: Rgb = bg.parse()?;

    let mut l1 = relative_luminance(&fg_rgb);
    let mut l2 = relative_luminance(&bg_rgb);
    if l1 < l2 {
        std::mem::swap(&mut l1, &mut l2);
    }

    Ok((l1 + 0.05) / (l2 + 0.05))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_ratio_black_and_white() {
        let ratio = contrast_ratio("#ffffff", "#000000").unwrap();
        assert!((ratio - 21.0).abs() < 0.1);

        let same = contrast_ratio("#123456", "#123456").unwrap();
        assert!((same - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_rgb_parse_invalid() {
        assert!(contrast_ratio("#abc", "#000000").is_err());
        assert!(contrast_ratio("not_hex", "#ffffff").is_err());
    }
}


