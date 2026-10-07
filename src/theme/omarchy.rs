// First-party Omarchy Linux integration for dynamic system-wide theming.
// Reads live color palette from Omarchy: ~/.local/state/omarchy/current/theme/colors.toml

use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

use crate::theme::Theme;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct OmarchyColors {
    pub background: Option<String>,
    pub foreground: Option<String>,
    pub accent: Option<String>,
    pub selection: Option<String>,
    pub dark_background: Option<String>,
    pub darker_background: Option<String>,
    pub bright_foreground: Option<String>,

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

pub fn parse_omarchy_colors(content: &str) -> Option<Theme> {
    let colors: OmarchyColors = toml::from_str(content).ok()?;

    let bg = colors.background.unwrap_or_else(|| "#1a1b26".to_string());
    let fg = colors.foreground.unwrap_or_else(|| "#c0caf5".to_string());
    let accent = colors.accent.clone().or(colors.color5.clone()).unwrap_or_else(|| "#bb9af7".to_string());
    let selection = colors.selection.unwrap_or_else(|| "#283457".to_string());

    // ANSI colors mapping
    let work = colors.color1.unwrap_or_else(|| "#f7768e".to_string());
    let brk = colors.color4.unwrap_or_else(|| "#7aa2f7".to_string());
    let long_brk = colors.color2.unwrap_or_else(|| "#9ece6a".to_string());
    let idle = colors.color8.clone().unwrap_or_else(|| "#565f89".to_string());
    let muted = colors.color8.unwrap_or_else(|| "#565f89".to_string());
    let subtle = colors.color0.unwrap_or_else(|| selection.clone());
    let border = accent.clone();
    let ambient = colors.dark_background.unwrap_or_else(|| selection.clone());

    Some(Theme {
        name: "omarchy".to_string(),
        work,
        brk,
        long_break: long_brk,
        idle,
        accent: accent.clone(),
        background: bg,
        text: fg,
        muted,
        subtle,
        border,
        progress_fill: accent,
        progress_track: selection,
        ambient,
        description: "Dynamic system theme loaded from Omarchy Linux".to_string(),
    })
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
}


