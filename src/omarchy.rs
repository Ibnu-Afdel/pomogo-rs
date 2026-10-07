// First-party Omarchy Linux system integration and tools.

use std::fs;
use std::path::PathBuf;
use crate::theme::omarchy::{is_omarchy_environment, omarchy_colors_file_path};

pub fn print_omarchy_status() {
    println!("Omarchy Linux First-Party Support");
    println!("---------------------------------");
    let is_om = is_omarchy_environment();
    if is_om {
        println!("  Omarchy Environment:  ✔ Detected");
    } else {
        println!("  Omarchy Environment:  (Standard Linux system detected)");
    }

    if let Some(palette) = omarchy_colors_file_path() {
        println!("  System Theme Source:  ✔ Found ({})", palette.display());
    } else {
        println!("  System Theme Source:  Not yet initialized");
    }

    let runtime = crate::statefile::xdg_runtime_dir();
    println!("  IPC State Channel:    {}/state.json", runtime.display());

    let home = std::env::var("HOME").unwrap_or_default();
    let qs_plugin_path = PathBuf::from(&home).join(".config/omarchy/plugins/pomogo");
    if qs_plugin_path.exists() {
        println!("  Quickshell Plugin:    ✔ Installed in {}", qs_plugin_path.display());
    } else {
        println!("  Quickshell Plugin:    Available (run 'pomogo omarchy install-quickshell')");
    }
}

pub fn install_quickshell_plugin() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return Err("HOME environment variable is not set".to_string());
    }

    let plugin_dir = PathBuf::from(&home).join(".config/omarchy/plugins/pomogo");
    fs::create_dir_all(&plugin_dir)
        .map_err(|e| format!("failed to create plugin directory: {}", e))?;

    let qml_file = plugin_dir.join("PomogoBar.qml");
    let qml_content = include_str!("../contrib/omarchy/quickshell-plugin/PomogoBar.qml");
    fs::write(&qml_file, qml_content)
        .map_err(|e| format!("failed to write PomogoBar.qml: {}", e))?;

    let meta_file = plugin_dir.join("plugin.json");
    let meta_content = include_str!("../contrib/omarchy/quickshell-plugin/plugin.json");
    fs::write(&meta_file, meta_content)
        .map_err(|e| format!("failed to write plugin.json: {}", e))?;

    Ok(plugin_dir)
}

pub fn install_desktop_entry() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return Err("HOME environment variable is not set".to_string());
    }

    let app_dir = PathBuf::from(&home).join(".local/share/applications");
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("failed to create applications directory: {}", e))?;

    let desktop_file = app_dir.join("pomogo.desktop");
    let desktop_content = include_str!("../contrib/pomogo.desktop");
    fs::write(&desktop_file, desktop_content)
        .map_err(|e| format!("failed to write pomogo.desktop: {}", e))?;

    Ok(desktop_file)
}

