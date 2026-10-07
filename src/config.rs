// TOML-based configuration for PomoGo.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use chrono::Duration;
use serde::{Deserialize, Serialize};

use crate::render::LAYOUT_NAMES;
use crate::wellness::WellnessConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    // Durations (in minutes)
    #[serde(default = "default_work_duration")]
    pub work_duration: usize,
    #[serde(default = "default_short_break_duration")]
    pub short_break_duration: usize,
    #[serde(default = "default_long_break_duration")]
    pub long_break_duration: usize,
    #[serde(default = "default_sessions_before_long_break")]
    pub sessions_before_long_break: usize,

    // Display
    #[serde(default = "default_theme")]
    pub theme: String,
    #[serde(default = "default_layout")]
    pub layout: String,
    #[serde(default = "default_effects")]
    pub effects: String,

    // Notifications
    #[serde(default = "default_true")]
    pub notifications_enabled: bool,
    #[serde(default = "default_true")]
    pub sound_enabled: bool,
    #[serde(default = "default_sound_start")]
    pub sound_start_event: String,
    #[serde(default = "default_sound_end")]
    pub sound_end_event: String,

    /// Roll from focus into breaks and back without waiting for a key.
    #[serde(default = "default_true")]
    pub autopilot: bool,

    /// Focus minutes to aim for each day; 0 hides the goal.
    #[serde(default = "default_daily_goal")]
    pub daily_goal_minutes: u32,

    /// Eyes, water and stretch reminders while focusing.
    #[serde(default)]
    pub wellness: WellnessConfig,

    // Notes
    #[serde(default = "default_true")]
    pub prompt_for_notes: bool,

    // Lock & Suspend
    #[serde(default = "default_true")]
    pub pause_on_lock: bool,
    #[serde(default = "default_true")]
    pub pause_on_suspend: bool,
    #[serde(default = "default_true")]
    pub terminal_title_enabled: bool,
    #[serde(default = "default_true")]
    pub show_git: bool,
    #[serde(default = "default_false")]
    pub show_tmux: bool,

    // Profiles
    #[serde(default)]
    pub profiles: HashMap<String, Profile>,

    // Mode configurations
    #[serde(default)]
    pub quick_focus: QuickFocusConfig,
    #[serde(default)]
    pub deep_focus: DeepFocusConfig,

    // Hooks
    #[serde(default)]
    pub on_work_start: Option<String>,
    #[serde(default)]
    pub on_work_end: Option<String>,
    #[serde(default)]
    pub on_break_start: Option<String>,
    #[serde(default)]
    pub on_break_end: Option<String>,
}

fn default_work_duration() -> usize { 25 }
fn default_short_break_duration() -> usize { 5 }
fn default_long_break_duration() -> usize { 15 }
fn default_sessions_before_long_break() -> usize { 4 }
fn default_theme() -> String { "auto".to_string() }
fn default_layout() -> String { "focus".to_string() }
fn default_daily_goal() -> u32 { 240 }
fn default_effects() -> String { "none".to_string() }
fn default_sound_start() -> String { "message-new-instant".to_string() }
fn default_sound_end() -> String { "complete".to_string() }
fn default_true() -> bool { true }
fn default_false() -> bool { false }

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuickFocusConfig {
    pub work_duration: Option<usize>,
    pub short_break_duration: Option<usize>,
    pub long_break_duration: Option<usize>,
    pub sessions_before_long_break: Option<usize>,
    pub auto_advance: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeepFocusConfig {
    pub work_duration: Option<usize>,
    pub short_break_duration: Option<usize>,
    pub long_break_duration: Option<usize>,
    pub sessions_before_long_break: Option<usize>,
    pub default_duration: Option<usize>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Profile {
    pub work_duration: Option<usize>,
    pub short_break_duration: Option<usize>,
    pub long_break_duration: Option<usize>,
    pub sessions_before_long_break: Option<usize>,
    pub theme: Option<String>,
    pub layout: Option<String>,
    pub project: Option<String>,
    pub sound_event: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            work_duration: 25,
            short_break_duration: 5,
            long_break_duration: 15,
            sessions_before_long_break: 4,
            theme: default_theme(),
            layout: default_layout(),
            effects: "none".to_string(),
            notifications_enabled: true,
            sound_enabled: true,
            sound_start_event: "message-new-instant".to_string(),
            sound_end_event: "complete".to_string(),
            autopilot: true,
            daily_goal_minutes: default_daily_goal(),
            wellness: WellnessConfig::default(),
            prompt_for_notes: true,
            pause_on_lock: true,
            pause_on_suspend: true,
            terminal_title_enabled: true,
            show_git: true,
            show_tmux: false,
            profiles: HashMap::new(),
            quick_focus: QuickFocusConfig::default(),
            deep_focus: DeepFocusConfig::default(),
            on_work_start: None,
            on_work_end: None,
            on_break_start: None,
            on_break_end: None,
        }
    }
}

pub fn xdg_config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("pomogo");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("pomogo")
}

pub fn config_file_path() -> PathBuf {
    xdg_config_dir().join("config.toml")
}

pub fn xdg_data_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("pomogo");
        }
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".local").join("share").join("pomogo")
}

pub fn db_file_path() -> PathBuf {
    xdg_data_dir().join("pomogo.db")
}

impl Config {
    pub fn load() -> Result<Self, String> {
        let path = config_file_path();
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| format!("failed to read config file {}: {}", path.display(), e))?;

        let cfg: Config = toml::from_str(&content)
            .map_err(|e| format!("failed to parse config file: {}", e))?;

        cfg.validate()?;
        Ok(cfg)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.work_duration == 0 {
            return Err("work_duration must be positive".to_string());
        }
        if self.short_break_duration == 0 {
            return Err("short_break_duration must be positive".to_string());
        }
        if self.long_break_duration == 0 {
            return Err("long_break_duration must be positive".to_string());
        }
        if self.sessions_before_long_break == 0 {
            return Err("sessions_before_long_break must be positive".to_string());
        }
        if self.theme.is_empty() {
            return Err("theme must be set".to_string());
        }

        if !LAYOUT_NAMES.contains(&self.layout.as_str())
            && !["random", "daily", ""].contains(&self.layout.as_str())
        {
            return Err(format!("unknown layout: {:?}", self.layout));
        }

        let valid_effects = ["none", "stars", "snow", "rain", "embers", "scanline", "random", ""];
        if !valid_effects.contains(&self.effects.as_str()) {
            return Err(format!("unknown effects: {:?}", self.effects));
        }

        Ok(())
    }

    pub fn work_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.work_duration as i64)
    }

    pub fn short_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.short_break_duration as i64)
    }

    pub fn long_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.long_break_duration as i64)
    }

    // Quick focus helpers
    pub fn quick_focus_work_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.quick_focus.work_duration.unwrap_or(self.work_duration) as i64)
    }

    pub fn quick_focus_short_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.quick_focus.short_break_duration.unwrap_or(self.short_break_duration) as i64)
    }

    pub fn quick_focus_long_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.quick_focus.long_break_duration.unwrap_or(self.long_break_duration) as i64)
    }

    pub fn quick_focus_sessions_before_long_break(&self) -> usize {
        self.quick_focus.sessions_before_long_break.unwrap_or(self.sessions_before_long_break)
    }

    pub fn quick_focus_auto_advance(&self) -> bool {
        self.quick_focus.auto_advance.unwrap_or(self.autopilot)
    }

    // Deep focus helpers
    pub fn deep_focus_work_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.deep_focus.work_duration.unwrap_or(self.work_duration) as i64)
    }

    pub fn deep_focus_short_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.deep_focus.short_break_duration.unwrap_or(self.short_break_duration) as i64)
    }

    pub fn deep_focus_long_break_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.deep_focus.long_break_duration.unwrap_or(self.long_break_duration) as i64)
    }

    pub fn deep_focus_sessions_before_long_break(&self) -> usize {
        self.deep_focus.sessions_before_long_break.unwrap_or(self.sessions_before_long_break)
    }

    pub fn deep_focus_default_duration_as_duration(&self) -> Duration {
        Duration::minutes(self.deep_focus.default_duration.unwrap_or(120) as i64)
    }

    pub fn resolve_profile(&self, name: &str) -> (Config, String, String) {
        let mut cfg = self.clone();
        let mut project = String::new();
        let mut sound_event = String::new();

        if let Some(prof) = self.profiles.get(name) {
            if let Some(w) = prof.work_duration {
                cfg.work_duration = w;
            }
            if let Some(s) = prof.short_break_duration {
                cfg.short_break_duration = s;
            }
            if let Some(l) = prof.long_break_duration {
                cfg.long_break_duration = l;
            }
            if let Some(n) = prof.sessions_before_long_break {
                cfg.sessions_before_long_break = n;
            }
            if let Some(th) = &prof.theme {
                cfg.theme = th.clone();
            }
            if let Some(ly) = &prof.layout {
                cfg.layout = ly.clone();
            }
            if let Some(p) = &prof.project {
                project = p.clone();
            }
            if let Some(se) = &prof.sound_event {
                sound_event = se.clone();
            }
        }

        (cfg, project, sound_event)
    }

    pub fn write_default(force: bool) -> Result<PathBuf, String> {
        let dir = xdg_config_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create config directory {}: {}", dir.display(), e))?;

        let file = config_file_path();
        if file.exists() && !force {
            return Err(format!(
                "{} already exists (use --force to overwrite it)",
                file.display()
            ));
        }
        let default_content = r#"# PomoGo Configuration File
# Location: ~/.config/pomogo/config.toml

# Durations in minutes
work_duration = 25
short_break_duration = 5
long_break_duration = 15
sessions_before_long_break = 4

# Theming: "auto", "omarchy", "tokyo-night", "catppuccin", "gruvbox", "rose-pine",
# "nord", "everforest", "dracula", "kanagawa", "random", "daily"
# "auto" follows the active Omarchy theme (and its live changes) on Omarchy,
# and uses tokyo-night everywhere else. Run `pomogo themes` for the full list.
theme = "auto"

# Layout: "classic", "minimal", "centered", "compact", "retro", "dashboard",
# "monolith", "tinybar", "terminal-rice", "focus-stack", "command-center", "random", "daily"
layout = "classic"

# Ambient Background Effects: "none", "stars", "snow", "rain", "embers", "scanline", "random"
effects = "none"

# Notifications & Audio
notifications_enabled = true
sound_enabled = true
sound_start_event = "message-new-instant"
sound_end_event = "complete"

# Options
prompt_for_notes = true
pause_on_lock = true
pause_on_suspend = true
terminal_title_enabled = true
show_git = true
show_tmux = false

# Quick Focus overrides
[quick_focus]
auto_advance = false

# Deep Focus overrides
[deep_focus]
default_duration = 120

# Optional Profiles: e.g. run with 'pomogo start coding'
[profiles.coding]
work_duration = 50
short_break_duration = 10
layout = "dashboard"
project = "Dev"

[profiles.review]
work_duration = 20
short_break_duration = 5
layout = "compact"
project = "Code Review"
"#;

        fs::write(&file, default_content)
            .map_err(|e| format!("failed to write config file {}: {}", file.display(), e))?;

        Ok(file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults_and_validation() {
        let mut cfg = Config::default();
        assert!(cfg.validate().is_ok());
        assert_eq!(cfg.work_duration, 25);
        assert_eq!(cfg.short_break_duration, 5);
        assert_eq!(cfg.long_break_duration, 15);
        assert_eq!(cfg.sessions_before_long_break, 4);

        // Invalid work duration
        cfg.work_duration = 0;
        assert!(cfg.validate().is_err());
        cfg.work_duration = 25;

        // Invalid layout
        cfg.layout = "nonexistent_layout".to_string();
        assert!(cfg.validate().is_err());
        cfg.layout = "classic".to_string();

        // Invalid effects
        cfg.effects = "fireworks".to_string();
        assert!(cfg.validate().is_err());
        cfg.effects = "none".to_string();
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_profile_override() {
        let mut cfg = Config::default();
        let profile = Profile {
            work_duration: Some(50),
            short_break_duration: Some(10),
            long_break_duration: None,
            sessions_before_long_break: None,
            theme: Some("nord".to_string()),
            layout: Some("monolith".to_string()),
            project: Some("Backend".to_string()),
            sound_event: None,
        };
        cfg.profiles.insert("deep".to_string(), profile);

        let (new_cfg, proj, _sound) = cfg.resolve_profile("deep");
        assert_eq!(new_cfg.work_duration, 50);
        assert_eq!(new_cfg.short_break_duration, 10);
        assert_eq!(new_cfg.theme, "nord");
        assert_eq!(new_cfg.layout, "monolith");
        assert_eq!(proj, "Backend");
    }
}


