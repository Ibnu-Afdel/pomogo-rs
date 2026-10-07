// Loading and validating user themes from ~/.config/pomogo/themes/*.toml

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::Deserialize;

use crate::config::xdg_config_dir;
use crate::theme::contrast::contrast_ratio;
use crate::theme::Theme;

#[derive(Debug, Clone, Deserialize)]
pub struct ExternalTheme {
    pub name: Option<String>,
    pub work: String,
    #[serde(rename = "break")]
    pub brk: String,
    #[serde(rename = "long-break")]
    pub long_break: String,
    pub idle: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub muted: String,
    pub subtle: String,
    pub border: String,
    #[serde(rename = "progress-fill")]
    pub progress_fill: String,
    #[serde(rename = "progress-track")]
    pub progress_track: String,
    pub ambient: String,
    pub description: Option<String>,
}

impl ExternalTheme {
    pub fn to_theme(&self, default_name: &str) -> Theme {
        Theme {
            name: self.name.clone().unwrap_or_else(|| default_name.to_string()),
            work: self.work.clone(),
            brk: self.brk.clone(),
            long_break: self.long_break.clone(),
            idle: self.idle.clone(),
            accent: self.accent.clone(),
            background: self.background.clone(),
            text: self.text.clone(),
            muted: self.muted.clone(),
            subtle: self.subtle.clone(),
            border: self.border.clone(),
            progress_fill: self.progress_fill.clone(),
            progress_track: self.progress_track.clone(),
            ambient: self.ambient.clone(),
            description: self.description.clone().unwrap_or_default(),
        }
    }
}

pub fn themes_dir() -> PathBuf {
    xdg_config_dir().join("themes")
}

pub fn load_external_themes() -> HashMap<String, Theme> {
    let mut themes = HashMap::new();
    let dir = themes_dir();
    if !dir.exists() {
        return themes;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(ext) = toml::from_str::<ExternalTheme>(&content) {
                        let stem = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("custom");
                        let th = ext.to_theme(stem);
                        themes.insert(th.name.clone(), th);
                    }
                }
            }
        }
    }

    themes
}

pub fn check_external_themes() -> Vec<String> {
    let mut malformed = Vec::new();
    let dir = themes_dir();
    if !dir.exists() {
        return malformed;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if toml::from_str::<ExternalTheme>(&content).is_err() {
                        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                            malformed.push(name.to_string());
                        }
                    }
                }
            }
        }
    }

    malformed
}

pub fn check_external_theme_contrast() -> Vec<String> {
    let mut low_contrast = Vec::new();
    let dir = themes_dir();
    if !dir.exists() {
        return low_contrast;
    }

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(ext) = toml::from_str::<ExternalTheme>(&content) {
                        let mut issues = Vec::new();
                        if let Ok(ratio) = contrast_ratio(&ext.text, &ext.background) {
                            if ratio < 4.5 {
                                issues.push(format!("text/background ratio {:.2} < 4.5", ratio));
                            }
                        }
                        if let Ok(ratio) = contrast_ratio(&ext.muted, &ext.background) {
                            if ratio < 2.0 {
                                issues.push(format!("muted/background ratio {:.2} < 2.0", ratio));
                            }
                        }
                        if let Ok(ratio) = contrast_ratio(&ext.accent, &ext.background) {
                            if ratio < 3.0 {
                                issues.push(format!("accent/background ratio {:.2} < 3.0", ratio));
                            }
                        }

                        if !issues.is_empty() {
                            let filename = path
                                .file_name()
                                .and_then(|s| s.to_str())
                                .unwrap_or("theme.toml");
                            low_contrast.push(format!("{} ({})", filename, issues.join("; ")));
                        }
                    }
                }
            }
        }
    }

    low_contrast
}

