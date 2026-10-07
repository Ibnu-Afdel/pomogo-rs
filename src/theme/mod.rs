pub mod contrast;
pub mod external;
pub mod omarchy;

use std::collections::HashMap;
use chrono::Local;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub name: String,
    pub work: String,
    pub brk: String,
    pub long_break: String,
    pub idle: String,
    pub accent: String,
    pub background: String,
    pub text: String,
    pub muted: String,
    pub subtle: String,
    pub border: String,
    pub progress_fill: String,
    pub progress_track: String,
    pub ambient: String,
    pub description: String,
}

impl Theme {
    pub fn work(&self) -> &str { &self.work }
    pub fn brk(&self) -> &str { &self.brk }
    pub fn long_break(&self) -> &str { &self.long_break }
    pub fn idle(&self) -> &str { &self.idle }
    pub fn accent(&self) -> &str { &self.accent }
    pub fn background(&self) -> &str { &self.background }
    pub fn text(&self) -> &str { &self.text }
    pub fn muted(&self) -> &str { &self.muted }
    pub fn subtle(&self) -> &str { &self.subtle }
    pub fn border(&self) -> &str { &self.border }
    pub fn progress_fill(&self) -> &str { &self.progress_fill }
    pub fn progress_track(&self) -> &str { &self.progress_track }
    pub fn ambient(&self) -> &str { &self.ambient }
}

pub fn builtin_themes() -> HashMap<&'static str, Theme> {
    let mut m = HashMap::new();

    m.insert("tokyo-night", Theme {
        name: "tokyo-night".to_string(),
        work: "#f7768e".to_string(),
        brk: "#7aa2f7".to_string(),
        long_break: "#9ece6a".to_string(),
        idle: "#565f89".to_string(),
        accent: "#bb9af7".to_string(),
        background: "#1a1b26".to_string(),
        text: "#c0caf5".to_string(),
        muted: "#565f89".to_string(),
        subtle: "#414868".to_string(),
        border: "#3b4261".to_string(),
        progress_fill: "#bb9af7".to_string(),
        progress_track: "#24283b".to_string(),
        ambient: "#1f2335".to_string(),
        description: "Dark theme inspired by Tokyo's neon-lit nights.".to_string(),
    });

    m.insert("catppuccin", Theme {
        name: "catppuccin".to_string(),
        work: "#f38ba8".to_string(),
        brk: "#89b4fa".to_string(),
        long_break: "#a6e3a1".to_string(),
        idle: "#585b70".to_string(),
        accent: "#cba6f7".to_string(),
        background: "#1e1e2e".to_string(),
        text: "#cdd6f4".to_string(),
        muted: "#7f849c".to_string(),
        subtle: "#313244".to_string(),
        border: "#45475a".to_string(),
        progress_fill: "#f38ba8".to_string(),
        progress_track: "#181825".to_string(),
        ambient: "#11111b".to_string(),
        description: "Dark, warm pastel theme from the Catppuccin palette.".to_string(),
    });

    m.insert("catppuccin-latte", Theme {
        name: "catppuccin-latte".to_string(),
        work: "#d20f39".to_string(),
        brk: "#1e66f5".to_string(),
        long_break: "#40a02b".to_string(),
        idle: "#9ca0b0".to_string(),
        accent: "#8839ef".to_string(),
        background: "#eff1f5".to_string(),
        text: "#4c4f69".to_string(),
        muted: "#7c7f93".to_string(),
        subtle: "#ccd0da".to_string(),
        border: "#acb0be".to_string(),
        progress_fill: "#d20f39".to_string(),
        progress_track: "#e6e9ef".to_string(),
        ambient: "#ccd0da".to_string(),
        description: "Light pastel theme from the Catppuccin palette.".to_string(),
    });

    m.insert("gruvbox", Theme {
        name: "gruvbox".to_string(),
        work: "#fb4934".to_string(),
        brk: "#83a598".to_string(),
        long_break: "#b8bb26".to_string(),
        idle: "#928374".to_string(),
        accent: "#fe8019".to_string(),
        background: "#282828".to_string(),
        text: "#ebdbb2".to_string(),
        muted: "#a89984".to_string(),
        subtle: "#3c3836".to_string(),
        border: "#504945".to_string(),
        progress_fill: "#fb4934".to_string(),
        progress_track: "#1d2021".to_string(),
        ambient: "#1d2021".to_string(),
        description: "Warm, earthy retro theme with strong contrast.".to_string(),
    });

    m.insert("rose-pine", Theme {
        name: "rose-pine".to_string(),
        work: "#ebbcba".to_string(),
        brk: "#31748f".to_string(),
        long_break: "#9ccfd8".to_string(),
        idle: "#6e6a86".to_string(),
        accent: "#c4a7e7".to_string(),
        background: "#191724".to_string(),
        text: "#e0def4".to_string(),
        muted: "#908caa".to_string(),
        subtle: "#26233a".to_string(),
        border: "#403d52".to_string(),
        progress_fill: "#ebbcba".to_string(),
        progress_track: "#212030".to_string(),
        ambient: "#2a2837".to_string(),
        description: "All natural pine, foam, and rose colors.".to_string(),
    });

    m.insert("everforest", Theme {
        name: "everforest".to_string(),
        work: "#e67e80".to_string(),
        brk: "#7fbbb3".to_string(),
        long_break: "#a7c080".to_string(),
        idle: "#859289".to_string(),
        accent: "#dbbc7f".to_string(),
        background: "#2d353b".to_string(),
        text: "#d3c6aa".to_string(),
        muted: "#9da9a0".to_string(),
        subtle: "#343f44".to_string(),
        border: "#475258".to_string(),
        progress_fill: "#a7c080".to_string(),
        progress_track: "#232a2e".to_string(),
        ambient: "#232a2e".to_string(),
        description: "Warm, comforting forest green palette.".to_string(),
    });

    m.insert("nord", Theme {
        name: "nord".to_string(),
        work: "#bf616a".to_string(),
        brk: "#88c0d0".to_string(),
        long_break: "#a3be8c".to_string(),
        idle: "#4c566a".to_string(),
        accent: "#81a1c1".to_string(),
        background: "#2e3440".to_string(),
        text: "#d8dee9".to_string(),
        muted: "#81a1c1".to_string(),
        subtle: "#3b4252".to_string(),
        border: "#434c5e".to_string(),
        progress_fill: "#88c0d0".to_string(),
        progress_track: "#2e3440".to_string(),
        ambient: "#3b4252".to_string(),
        description: "Arctic, ice-cold color scheme.".to_string(),
    });

    m.insert("dracula", Theme {
        name: "dracula".to_string(),
        work: "#ff5555".to_string(),
        brk: "#8be9fd".to_string(),
        long_break: "#50fa7b".to_string(),
        idle: "#6272a4".to_string(),
        accent: "#bd93f9".to_string(),
        background: "#282a36".to_string(),
        text: "#f8f8f2".to_string(),
        muted: "#6272a4".to_string(),
        subtle: "#343746".to_string(),
        border: "#44475a".to_string(),
        progress_fill: "#bd93f9".to_string(),
        progress_track: "#191a21".to_string(),
        ambient: "#191a21".to_string(),
        description: "Classic, high-contrast dark theme for vampires.".to_string(),
    });

    m.insert("kanagawa", Theme {
        name: "kanagawa".to_string(),
        work: "#c3404b".to_string(),
        brk: "#7e9cd8".to_string(),
        long_break: "#76946a".to_string(),
        idle: "#727169".to_string(),
        accent: "#957fb8".to_string(),
        background: "#1f1f28".to_string(),
        text: "#dcd7ba".to_string(),
        muted: "#727169".to_string(),
        subtle: "#2a2a37".to_string(),
        border: "#363646".to_string(),
        progress_fill: "#957fb8".to_string(),
        progress_track: "#16161d".to_string(),
        ambient: "#16161d".to_string(),
        description: "Edo-era artistic theme inspired by the Great Wave.".to_string(),
    });

    m.insert("carbon", Theme {
        name: "carbon".to_string(),
        work: "#fa4d56".to_string(),
        brk: "#4589ff".to_string(),
        long_break: "#24a148".to_string(),
        idle: "#525252".to_string(),
        accent: "#a861ea".to_string(),
        background: "#161616".to_string(),
        text: "#f4f4f4".to_string(),
        muted: "#707070".to_string(),
        subtle: "#262626".to_string(),
        border: "#393939".to_string(),
        progress_fill: "#f4f4f4".to_string(),
        progress_track: "#161616".to_string(),
        ambient: "#262626".to_string(),
        description: "Monochrome, high-contrast gray scheme.".to_string(),
    });

    m.insert("night-owl", Theme {
        name: "night-owl".to_string(),
        work: "#ef5350".to_string(),
        brk: "#82aaff".to_string(),
        long_break: "#22da6e".to_string(),
        idle: "#637777".to_string(),
        accent: "#c792ea".to_string(),
        background: "#011627".to_string(),
        text: "#d6deeb".to_string(),
        muted: "#637777".to_string(),
        subtle: "#0b2942".to_string(),
        border: "#1d3b53".to_string(),
        progress_fill: "#82aaff".to_string(),
        progress_track: "#01111d".to_string(),
        ambient: "#0b253a".to_string(),
        description: "High-contrast blue night palette for late coding.".to_string(),
    });

    m.insert("one-dark", Theme {
        name: "one-dark".to_string(),
        work: "#e06c75".to_string(),
        brk: "#61afef".to_string(),
        long_break: "#98c379".to_string(),
        idle: "#5c6370".to_string(),
        accent: "#c678dd".to_string(),
        background: "#282c34".to_string(),
        text: "#abb2bf".to_string(),
        muted: "#5c6370".to_string(),
        subtle: "#2c313c".to_string(),
        border: "#3e4451".to_string(),
        progress_fill: "#61afef".to_string(),
        progress_track: "#21252b".to_string(),
        ambient: "#21252b".to_string(),
        description: "Atom's classic balanced dark editor palette.".to_string(),
    });

    m.insert("ayu-mirage", Theme {
        name: "ayu-mirage".to_string(),
        work: "#f07178".to_string(),
        brk: "#59c2ff".to_string(),
        long_break: "#bbe67e".to_string(),
        idle: "#607080".to_string(),
        accent: "#ffcc66".to_string(),
        background: "#1f2430".to_string(),
        text: "#cbccc6".to_string(),
        muted: "#707a8c".to_string(),
        subtle: "#242936".to_string(),
        border: "#343f4c".to_string(),
        progress_fill: "#ffcc66".to_string(),
        progress_track: "#1b1f29".to_string(),
        ambient: "#191e2a".to_string(),
        description: "Warm, low-glare Ayu palette for all-day focus.".to_string(),
    });

    m.insert("solarized-dark", Theme {
        name: "solarized-dark".to_string(),
        work: "#dc322f".to_string(),
        brk: "#268bd2".to_string(),
        long_break: "#859900".to_string(),
        idle: "#586e75".to_string(),
        accent: "#b58900".to_string(),
        background: "#002b36".to_string(),
        text: "#839496".to_string(),
        muted: "#586e75".to_string(),
        subtle: "#073642".to_string(),
        border: "#657b83".to_string(),
        progress_fill: "#2aa198".to_string(),
        progress_track: "#073642".to_string(),
        ambient: "#073642".to_string(),
        description: "Precision low-contrast palette built for terminals.".to_string(),
    });

    m.insert("oxocarbon", Theme {
        name: "oxocarbon".to_string(),
        work: "#ee5396".to_string(),
        brk: "#33b1ff".to_string(),
        long_break: "#42be65".to_string(),
        idle: "#525252".to_string(),
        accent: "#be95ff".to_string(),
        background: "#161616".to_string(),
        text: "#f2f4f8".to_string(),
        muted: "#78a9ff".to_string(),
        subtle: "#262626".to_string(),
        border: "#393939".to_string(),
        progress_fill: "#08bdba".to_string(),
        progress_track: "#262626".to_string(),
        ambient: "#0f0f0f".to_string(),
        description: "IBM-inspired cyberpunk palette with crisp accents.".to_string(),
    });

    m.insert("high-contrast", Theme {
        name: "high-contrast".to_string(),
        work: "#ff5f5f".to_string(),
        brk: "#00d7ff".to_string(),
        long_break: "#5fff87".to_string(),
        idle: "#bcbcbc".to_string(),
        accent: "#ffd700".to_string(),
        background: "#000000".to_string(),
        text: "#ffffff".to_string(),
        muted: "#bcbcbc".to_string(),
        subtle: "#1c1c1c".to_string(),
        border: "#ffffff".to_string(),
        progress_fill: "#ffd700".to_string(),
        progress_track: "#3a3a3a".to_string(),
        ambient: "#262626".to_string(),
        description: "Maximum contrast dark theme for readability.".to_string(),
    });

    m.insert("github-dark", Theme {
        name: "github-dark".to_string(),
        work: "#ff7b72".to_string(),
        brk: "#79c0ff".to_string(),
        long_break: "#a5d6ff".to_string(),
        idle: "#8b949e".to_string(),
        accent: "#d2a8ff".to_string(),
        background: "#0d1117".to_string(),
        text: "#e6edf3".to_string(),
        muted: "#8b949e".to_string(),
        subtle: "#161b22".to_string(),
        border: "#30363d".to_string(),
        progress_fill: "#58a6ff".to_string(),
        progress_track: "#30363d".to_string(),
        ambient: "#21262d".to_string(),
        description: "GitHub-inspired dark theme with crisp code-review contrast.".to_string(),
    });

    m.insert("material-ocean", Theme {
        name: "material-ocean".to_string(),
        work: "#ff5370".to_string(),
        brk: "#82aaff".to_string(),
        long_break: "#c3e88d".to_string(),
        idle: "#b2ccd6".to_string(),
        accent: "#ffcb6b".to_string(),
        background: "#0f111a".to_string(),
        text: "#eeffff".to_string(),
        muted: "#b2ccd6".to_string(),
        subtle: "#1f2233".to_string(),
        border: "#3b4252".to_string(),
        progress_fill: "#89ddff".to_string(),
        progress_track: "#2f334d".to_string(),
        ambient: "#252a3a".to_string(),
        description: "Material-style ocean palette with bright but readable accents.".to_string(),
    });

    m.insert("forest-dawn", Theme {
        name: "forest-dawn".to_string(),
        work: "#f26d6d".to_string(),
        brk: "#6fb3d2".to_string(),
        long_break: "#9fd356".to_string(),
        idle: "#c6d3b8".to_string(),
        accent: "#f7c65f".to_string(),
        background: "#10140f".to_string(),
        text: "#edf4e4".to_string(),
        muted: "#c6d3b8".to_string(),
        subtle: "#1d2519".to_string(),
        border: "#46543f".to_string(),
        progress_fill: "#9fd356".to_string(),
        progress_track: "#2b3327".to_string(),
        ambient: "#283224".to_string(),
        description: "Earthy dark theme with balanced green, blue, and gold signals.".to_string(),
    });

    m
}

pub fn list() -> Vec<String> {
    let mut names = Vec::new();
    // Built-in theme names
    let builtins = [
        "tokyo-night", "catppuccin", "catppuccin-latte", "gruvbox", "rose-pine",
        "everforest", "nord", "dracula", "kanagawa", "carbon", "night-owl",
        "one-dark", "ayu-mirage", "solarized-dark", "oxocarbon", "high-contrast",
        "github-dark", "material-ocean", "forest-dawn",
    ];
    for b in builtins {
        names.push(b.to_string());
    }

    // Add Omarchy dynamic theme if environment or file is present
    if omarchy::is_omarchy_environment() || omarchy::omarchy_colors_file_path().is_some() {
        names.push("omarchy".to_string());
    }

    // External themes
    let ext = external::load_external_themes();
    for (k, _) in ext {
        if !names.contains(&k) {
            names.push(k);
        }
    }

    names
}

pub fn get(name: &str) -> Theme {
    // 1. Check omarchy first if requested
    if name == "omarchy" {
        if let Some(om) = omarchy::load_omarchy_theme() {
            return om;
        }
    }

    // 2. Check external user themes
    let ext = external::load_external_themes();
    if let Some(t) = ext.get(name) {
        return t.clone();
    }

    // 3. Check builtins
    let builtins = builtin_themes();
    if let Some(t) = builtins.get(name) {
        return t.clone();
    }

    // Fallback: if running in Omarchy, try Omarchy theme
    if omarchy::is_omarchy_environment() {
        if let Some(om) = omarchy::load_omarchy_theme() {
            return om;
        }
    }

    // Fallback default: Tokyo Night
    builtins.get("tokyo-night").cloned().unwrap()
}

pub fn resolve_theme_name(configured: &str) -> String {
    // "auto" follows the desktop theme on Omarchy and falls back to Tokyo Night elsewhere.
    if configured.is_empty() || configured == "auto" {
        if omarchy::omarchy_colors_file_path().is_some() {
            return "omarchy".to_string();
        }
        return "tokyo-night".to_string();
    }

    if configured == "random" {
        let all = list();
        if all.is_empty() {
            return "tokyo-night".to_string();
        }
        let now_seed = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0) + std::process::id() as i64;
        let idx = (now_seed.unsigned_abs() as usize) % all.len();
        return all[idx].clone();
    }

    if configured == "daily" {
        let date_str = Local::now().format("%Y-%m-%d").to_string();
        let mut hash: i64 = 0;
        for b in date_str.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(b as i64);
        }
        let all = list();
        if all.is_empty() {
            return "tokyo-night".to_string();
        }
        let idx = (hash.unsigned_abs() as usize) % all.len();
        return all[idx].clone();
    }

    configured.to_string()
}

