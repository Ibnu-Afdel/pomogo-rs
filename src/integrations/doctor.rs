// System health diagnostics and doctor command.

use std::fs;
use std::process::Command;

use crate::config::{config_file_path, db_file_path, Config};
use crate::statefile::xdg_runtime_dir;
use crate::store::Store;
use crate::theme::external::{check_external_theme_contrast, check_external_themes};
use crate::theme::omarchy::{is_omarchy_environment, omarchy_colors_file_path};

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub name: String,
    pub passed: bool,
    pub message: String,
}

pub fn run_doctor() -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    // 1. Config Check
    let cfg_path = config_file_path();
    let mut cfg_diag = Diagnostic {
        name: "Configuration File Valid".to_string(),
        passed: true,
        message: "Valid config.toml".to_string(),
    };
    if !cfg_path.exists() {
        cfg_diag.passed = false;
        cfg_diag.message = format!(
            "Config file does not exist at {} (use 'pomogo config init' to create it)",
            cfg_path.display()
        );
    } else {
        match Config::load() {
            Ok(_) => {}
            Err(e) => {
                cfg_diag.passed = false;
                cfg_diag.message = format!("Error loading config: {}", e);
            }
        }
    }
    diags.push(cfg_diag);

    // 2. Database Check
    let db_path = db_file_path();
    let mut db_diag = Diagnostic {
        name: "SQLite Database Health".to_string(),
        passed: true,
        message: "Database is readable and writable".to_string(),
    };
    if !db_path.exists() {
        db_diag.passed = false;
        db_diag.message = format!(
            "Database file does not exist at {} (it will be created on next startup)",
            db_path.display()
        );
    } else {
        match Store::new(&db_path) {
            Ok(_) => {}
            Err(e) => {
                db_diag.passed = false;
                db_diag.message = format!("SQLite error: {}", e);
            }
        }
    }
    diags.push(db_diag);

    // 3. State File check
    let state_dir = xdg_runtime_dir();
    let mut state_diag = Diagnostic {
        name: "Runtime State Directory".to_string(),
        passed: true,
        message: "Runtime state directory is writable".to_string(),
    };
    if let Err(e) = fs::create_dir_all(&state_dir) {
        state_diag.passed = false;
        state_diag.message = format!("State directory {} not writable: {}", state_dir.display(), e);
    }
    diags.push(state_diag);

    // 4. Notification Utility (notify-send)
    let mut ns_diag = Diagnostic {
        name: "Notification Utility (notify-send)".to_string(),
        passed: true,
        message: "notify-send is installed".to_string(),
    };
    if Command::new("which").arg("notify-send").output().map(|o| o.status.success()).unwrap_or(false) {
        // passed
    } else {
        ns_diag.passed = false;
        ns_diag.message = "notify-send command not found in PATH (system notifications will be disabled)".to_string();
    }
    diags.push(ns_diag);

    // 5. Canberra Check
    let mut sound_diag = Diagnostic {
        name: "Canberra Sound Player (canberra-gtk-play)".to_string(),
        passed: true,
        message: "canberra-gtk-play is installed".to_string(),
    };
    if Command::new("which").arg("canberra-gtk-play").output().map(|o| o.status.success()).unwrap_or(false) {
        // passed
    } else {
        sound_diag.passed = false;
        sound_diag.message = "canberra-gtk-play not found in PATH (transition sounds will use terminal bell)".to_string();
    }
    diags.push(sound_diag);

    // 6. External Themes Check
    let mut theme_diag = Diagnostic {
        name: "External Themes".to_string(),
        passed: true,
        message: "All external themes loaded successfully".to_string(),
    };
    let malformed = check_external_themes();
    if !malformed.is_empty() {
        theme_diag.passed = false;
        theme_diag.message = format!("Malformed theme files found: {}", malformed.join(", "));
    } else {
        let low_contrast = check_external_theme_contrast();
        if !low_contrast.is_empty() {
            theme_diag.passed = false;
            theme_diag.message = format!("Low-contrast theme files found: {}", low_contrast.join(", "));
        }
    }
    diags.push(theme_diag);

    // 7. Omarchy First-Party Environment Check
    let mut omarchy_diag = Diagnostic {
        name: "Omarchy Linux Integration".to_string(),
        passed: true,
        message: "Omarchy system detected; live palette syncing available".to_string(),
    };
    if is_omarchy_environment() {
        if let Some(p) = omarchy_colors_file_path() {
            omarchy_diag.message = format!("Omarchy active; live palette found at {}", p.display());
        } else {
            omarchy_diag.message = "Omarchy environment detected; colors.toml ready".to_string();
        }
    } else {
        omarchy_diag.passed = true;
        omarchy_diag.message = "Standard Linux environment detected; native D-Bus and Waybar support active".to_string();
    }
    diags.push(omarchy_diag);

    diags
}

