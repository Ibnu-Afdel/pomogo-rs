// First-party Omarchy integration: the omarchy-shell bar widget, launcher
// and keybinding suggestions.
//
// Omarchy 4 runs its bar inside omarchy-shell (Quickshell). Third-party bar
// widgets live in ~/.config/omarchy/plugins/<id>/ with a manifest.json, and
// are switched on through `omarchy plugin enable`, which owns shell.json.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::config::xdg_config_dir;
use crate::theme::omarchy::{is_omarchy_environment, omarchy_colors_file_path};

pub const PLUGIN_ID: &str = "pomogo.timer";

const PLUGIN_FILES: [(&str, &str); 2] = [
    ("manifest.json", include_str!("../contrib/omarchy/plugin/manifest.json")),
    ("BarWidget.qml", include_str!("../contrib/omarchy/plugin/BarWidget.qml")),
];

/// Lua for ~/.config/hypr/bindings.lua (Omarchy 4 configures Hyprland in Lua).
pub const HYPRLAND_SNIPPET: &str = r#"-- PomoGo: open/focus the timer, and start/pause it from anywhere.
o.bind("SUPER + ALT + P", "PomoGo", { tui = "pomogo", focus = true })
o.bind("SUPER + SHIFT + ALT + P", "PomoGo start/pause", "pomogo toggle")

-- Optional: float the PomoGo window instead of tiling it.
o.window("org.omarchy.pomogo", { tag = "+floating-window" })
"#;

fn omarchy_config_dir() -> PathBuf {
    xdg_config_dir()
        .parent()
        .map(|p| p.join("omarchy"))
        .unwrap_or_else(|| PathBuf::from(".config/omarchy"))
}

pub fn plugin_dir() -> PathBuf {
    omarchy_config_dir().join("plugins").join(PLUGIN_ID)
}

fn command_exists(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}

/// The bar widget is enabled when its id appears in shell.json.
fn plugin_enabled() -> bool {
    fs::read_to_string(omarchy_config_dir().join("shell.json"))
        .map(|s| s.contains(&format!("\"{}\"", PLUGIN_ID)))
        .unwrap_or(false)
}

fn shell_knows_plugin() -> bool {
    Command::new("omarchy-shell")
        .args(["shell", "listPlugins"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).contains(&format!("\"{}\"", PLUGIN_ID)))
        .unwrap_or(false)
}

fn run_quiet(program: &str, args: &[&str]) -> Result<(), String> {
    let out = Command::new(program)
        .args(args)
        .output()
        .map_err(|e| format!("failed to run {}: {}", program, e))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{} {} failed: {}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

pub fn print_omarchy_status() {
    let check = |ok: bool| if ok { "✔" } else { "·" };

    println!("PomoGo on Omarchy");
    println!("-----------------");
    let on_omarchy = is_omarchy_environment();
    println!(
        "  [{}] Omarchy          {}",
        check(on_omarchy),
        if on_omarchy { "detected" } else { "not detected (generic Linux mode)" }
    );

    match omarchy_colors_file_path() {
        Some(p) => println!("  [✔] Theme palette    {} (use theme \"auto\" to follow it)", p.display()),
        None => println!("  [·] Theme palette    not found"),
    }

    let installed = plugin_dir().join("manifest.json").exists();
    println!(
        "  [{}] Bar widget       {}",
        check(installed),
        if installed {
            plugin_dir().display().to_string()
        } else {
            "not installed (run `pomogo omarchy install`)".to_string()
        }
    );
    if installed {
        let enabled = plugin_enabled();
        println!(
            "  [{}] Widget enabled   {}",
            check(enabled),
            if enabled { "yes" } else { "no (run `omarchy plugin enable pomogo.timer`)" }
        );
    }

    let state = crate::statefile::xdg_runtime_dir().join("state.json");
    println!("  [·] State file       {}", state.display());
}

/// Copies the bar widget into the Omarchy plugin directory and, unless told
/// otherwise, asks the running shell to load and enable it.
pub fn install_plugin(enable: bool) -> Result<PathBuf, String> {
    let dir = plugin_dir();
    if dir.join(".git").exists() {
        // Added with `omarchy plugin add`; writing into the checkout would
        // block `omarchy plugin update` from fast-forwarding it.
        println!("✔ Bar widget is a git checkout at {} (update it with `omarchy plugin update {}`)", dir.display(), PLUGIN_ID);
    } else {
        fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create {}: {}", dir.display(), e))?;
        for (name, content) in PLUGIN_FILES {
            let path = dir.join(name);
            fs::write(&path, content).map_err(|e| format!("failed to write {}: {}", path.display(), e))?;
        }
        println!("✔ Bar widget files written to {}", dir.display());
    }

    if !command_exists("omarchy") {
        println!("· `omarchy` is not on PATH; enable the widget from Omarchy's plugin settings.");
        return Ok(dir);
    }

    // The shell must know the plugin before it can be enabled. The rescan
    // runs asynchronously, so wait briefly for the id to show up.
    if command_exists("omarchy-shell") {
        let _ = run_quiet("omarchy-shell", &["shell", "rescanPlugins"]);
        for _ in 0..20 {
            if shell_knows_plugin() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    }

    if !enable {
        println!("· Enable it later with: omarchy plugin enable {}", PLUGIN_ID);
    } else if plugin_enabled() {
        println!("✔ Already enabled in the bar");
    } else {
        run_quiet("omarchy", &["plugin", "enable", PLUGIN_ID])?;
        println!("✔ Enabled in the bar (move it with `omarchy bar move {} --section right`)", PLUGIN_ID);
    }

    Ok(dir)
}

pub fn uninstall_plugin() -> Result<(), String> {
    if plugin_enabled() && command_exists("omarchy") {
        run_quiet("omarchy", &["plugin", "disable", PLUGIN_ID])?;
        println!("✔ Removed from the bar");
    }
    let dir = plugin_dir();
    if dir.join(".git").exists() && command_exists("omarchy") {
        run_quiet("omarchy", &["plugin", "remove", PLUGIN_ID, "--yes"])?;
        println!("✔ Removed the git-managed widget with `omarchy plugin remove`");
    } else if dir.exists() {
        fs::remove_dir_all(&dir).map_err(|e| format!("failed to remove {}: {}", dir.display(), e))?;
        println!("✔ Deleted {}", dir.display());
    } else {
        println!("· Bar widget was not installed");
    }
    Ok(())
}

pub fn install_desktop_entry() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    if home.is_empty() {
        return Err("HOME environment variable is not set".to_string());
    }

    let app_dir = std::env::var("XDG_DATA_HOME")
        .ok()
        .filter(|d| !d.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(&home).join(".local/share"))
        .join("applications");
    fs::create_dir_all(&app_dir)
        .map_err(|e| format!("failed to create applications directory: {}", e))?;

    let desktop_file = app_dir.join("pomogo.desktop");
    let desktop_content = include_str!("../contrib/pomogo.desktop");
    fs::write(&desktop_file, desktop_content)
        .map_err(|e| format!("failed to write pomogo.desktop: {}", e))?;

    Ok(desktop_file)
}
