// First-party Omarchy Linux integration for dynamic system-wide theming.
// Reads the live palette from ~/.local/state/omarchy/current/theme/colors.toml.
//
// Omarchy 4 themes name their colors (red, blue, muted, ...); older themes
// only carry the terminal palette (color0..color15). Both are understood,
// with the named keys taking precedence.

use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OmarchyColors {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub accent: Option<String>,
    pub selection: Option<String>,
    pub muted: Option<String>,
    pub dark_background: Option<String>,
    pub darker_background: Option<String>,
    pub lighter_background: Option<String>,
    pub bright_foreground: Option<String>,

    pub red: Option<String>,
    pub green: Option<String>,
    pub yellow: Option<String>,
    pub blue: Option<String>,
    pub magenta: Option<String>,
    pub cyan: Option<String>,

    pub color0: Option<String>,
    pub color1: Option<String>,
    pub color2: Option<String>,
    pub color3: Option<String>,
    pub color4: Option<String>,
    pub color5: Option<String>,
    pub color6: Option<String>,
    pub color7: Option<String>,
    pub color8: Option<String>,
    pub color9: Option<String>,
    pub color10: Option<String>,
    pub color11: Option<String>,
    pub color12: Option<String>,
    pub color13: Option<String>,
    pub color14: Option<String>,
    pub color15: Option<String>,
}

pub fn is_omarchy_environment() -> bool {
    if std::env::var("OMARCHY").is_ok() {
        return true;
    }

    let home = std::env::var("HOME").unwrap_or_default();
    if !home.is_empty() {
        let state_dir = PathBuf::from(&home).join(".local/state/omarchy");
        if state_dir.exists() {
            return true;
        }
        let config_dir = PathBuf::from(&home).join(".config/omarchy");
        if config_dir.exists() {
            return true;
        }
    }

    PathBuf::from("/etc/omarchy").exists()
}

pub fn omarchy_colors_file_path() -> Option<PathBuf> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return None;
    }

    // Canonical active theme path in Omarchy
    let canonical = PathBuf::from(&home).join(".local/state/omarchy/current/theme/colors.toml");
    if canonical.exists() {
        return Some(canonical);
    }

    // Check alternate config symlink
    let alt = PathBuf::from(&home).join(".config/omarchy/current/theme/colors.toml");
    if alt.exists() {
        return Some(alt);
    }

    None
}

fn pick(candidates: &[&Option<String>], fallback: &str) -> String {
    candidates
        .iter()
        .find_map(|c| c.as_ref().filter(|v| !v.trim().is_empty()).cloned())
        .unwrap_or_else(|| fallback.to_string())
}

pub fn parse_omarchy_colors(content: &str) -> Option<Theme> {
    let c: OmarchyColors = toml::from_str(content).ok()?;

    let bg = pick(&[&c.background], "#1a1b26");
    let fg = pick(&[&c.foreground], "#c0caf5");
    let accent = pick(&[&c.accent, &c.blue, &c.color4], "#7aa2f7");
    let selection = pick(&[&c.selection, &c.lighter_background, &c.color0], "#283457");

    let work = pick(&[&c.red, &c.color1], "#f7768e");
    let brk = pick(&[&c.blue, &c.color4], "#7aa2f7");
    let long_brk = pick(&[&c.green, &c.color2], "#9ece6a");
    let muted = pick(&[&c.muted, &c.color8], "#565f89");
    let subtle = pick(&[&c.lighter_background, &c.selection, &c.color0], &selection);
    let ambient = pick(&[&c.dark_background, &c.selection, &c.color0], &selection);

    Some(Theme {
        name: "omarchy".to_string(),
        work,
        brk,
        long_break: long_brk,
        idle: muted.clone(),
        accent: accent.clone(),
        background: bg,
        text: fg,
        muted,
        subtle,
        border: accent.clone(),
        progress_fill: accent,
        progress_track: selection,
        ambient,
        description: "Follows the active Omarchy theme".to_string(),
    })
}

/// Modification time of the active Omarchy palette, used to notice
/// `omarchy theme set` while the TUI is running.
pub fn omarchy_colors_mtime() -> Option<SystemTime> {
    let path = omarchy_colors_file_path()?;
    fs::metadata(path).and_then(|m| m.modified()).ok()
}

pub fn load_omarchy_theme() -> Option<Theme> {
    let path = omarchy_colors_file_path()?;
    let content = fs::read_to_string(&path).ok()?;
    parse_omarchy_colors(&content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_omarchy_colors() {
        let sample = r##"
background = "#181825"
foreground = "#cdd6f4"
accent = "#cba6f7"
selection = "#313244"
color1 = "#f38ba8"
color2 = "#a6e3a1"
color4 = "#89b4fa"
color8 = "#585b70"
"##;
        let theme = parse_omarchy_colors(sample).expect("parse omarchy colors");
        assert_eq!(theme.name, "omarchy");
        assert_eq!(theme.background, "#181825");
        assert_eq!(theme.text, "#cdd6f4");
        assert_eq!(theme.accent, "#cba6f7");
        assert_eq!(theme.work, "#f38ba8");
        assert_eq!(theme.brk, "#89b4fa");
        assert_eq!(theme.long_break, "#a6e3a1");
        assert_eq!(theme.muted, "#585b70");
    }

    #[test]
    fn test_parse_omarchy4_named_colors() {
        let sample = r##"
mode = "dark"
accent = "#509475"
selection = "#32473B"
muted = "#53685B"
background = "#111c18"
dark_background = "#0c1512"
lighter_background = "#23372B"
foreground = "#C1C497"
red = "#FF5345"
green = "#549e6a"
blue = "#509475"
"##;
        let theme = parse_omarchy_colors(sample).expect("parse omarchy 4 colors");
        assert_eq!(theme.work, "#FF5345");
        assert_eq!(theme.brk, "#509475");
        assert_eq!(theme.long_break, "#549e6a");
        assert_eq!(theme.muted, "#53685B");
        assert_eq!(theme.accent, "#509475");
        assert_eq!(theme.progress_track, "#32473B");
        assert_eq!(theme.ambient, "#0c1512");
    }
}


