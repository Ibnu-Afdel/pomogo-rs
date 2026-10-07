# PomoGo (Rust Edition) 🦀

> Sleek, distraction-free Pomodoro timer TUI for Linux, developers, and rice enthusiasts with **first-party Omarchy Linux support**.

A ground-up, high-performance Rust rewrite of [Pomogo](../pomogo), preserving complete data compatibility with existing SQLite session stores while delivering instant startup, zero runtime dependencies, robust memory safety, and native Omarchy Linux desktop integration.

---

## Highlights

- 🎨 **First-Party Omarchy Linux Support**:
  - Live color palette extraction from `~/.local/state/omarchy/current/theme/colors.toml`.
  - Seamless desktop integration with native Quickshell top-bar widget (`PomogoBar.qml`).
  - Pre-configured Hyprland floating window rules and keybindings.
  - Dedicated CLI management via `pomogo omarchy status` & `pomogo omarchy install-quickshell`.
- 📐 **11 Responsive Terminal Layouts**:
  - `classic`, `minimal`, `centered`, `compact`, `retro`, `dashboard`, `monolith`, `tinybar`, `terminal-rice`, `focus-stack`, `command-center`.
  - Supports `--layout random` or `--layout daily`.
- ✨ **Ambient Particle Backgrounds**:
  - `stars`, `snow`, `rain`, `embers`, and `scanline`.
- 🗄️ **Zero-Migration SQLite Store**:
  - Automatically loads and continues sessions from existing Go PomoGo database (`~/.local/share/pomogo/pomogo.db`).
  - Schema migrations 1–7 preserved.
- ⚡ **IPC & Status Bar Ecosystem**:
  - Real-time atomic state written to `$XDG_RUNTIME_DIR/pomogo/state.json`.
  - Status formatters for Waybar (`pomogo status --format waybar`), Tmux, Starship, and raw JSON.
- 🔒 **Linux Desktop Awareness**:
  - D-Bus `LockedHint` system-lock auto-pausing.
  - Freedesktop notification action buttons via `notify-rust`.
  - Canberra GTK audio cues (`canberra-gtk-play`) with fallback terminal bell.
  - Active Git branch and Tmux session detection.

---

## Installation & Build

### Prerequisites
- Rust 1.75+ (via `rustup`)
- Linux packages: `libcanberra-gtk3-module` / `canberra-gtk-play`, `libdbus-1-dev` (optional for system lock detection)

### Building
```bash
cd pomogo-rust
cargo build --release
```
The optimized executable will be located at `target/release/pomogo`.

To install to `~/.local/bin`:
```bash
cargo install --path .
```

---

## Omarchy Linux Integration

PomoGo Rust provides first-party integration with [Omarchy Linux](https://omarchy.org/):

1. **Verify Environment**:
   ```bash
   pomogo omarchy status
   ```
2. **Install Native Quickshell Widget**:
   ```bash
   pomogo omarchy install-quickshell
   ```
3. **Automated Setup Script**:
   ```bash
   bash contrib/omarchy/omarchy-setup.sh
   ```

### Dynamic Palette
When running under Omarchy, PomoGo automatically detects your active theme:
```bash
pomogo --theme omarchy
```
If you switch your theme in Omarchy, PomoGo adapts seamlessly.

---

## Keyboard Controls

| Key | Action |
|:---:|:---|
| <kbd>s</kbd> / <kbd>Space</kbd> | Start / Pause / Resume timer |
| <kbd>n</kbd> | Skip to next phase |
| <kbd>r</kbd> | Reset current session |
| <kbd>t</kbd> | Set active task name |
| <kbd>p</kbd> | Select or manage active project |
| <kbd>d</kbd> | Quick duration picker |
| <kbd>z</kbd> | Toggle Zen mode (hide UI chrome) |
| <kbd>m</kbd> | Toggle sound mute |
| <kbd>S</kbd> | Open sound profile picker |
| <kbd>i</kbd> | Open focus stats dashboard |
| <kbd>c</kbd> | Copy timer status to clipboard (OSC 52) |
| <kbd>?</kbd> | Open help dialog |
| <kbd>q</kbd> | Exit PomoGo |

---

## CLI Reference

```
pomogo [COMMAND]

Commands:
  start               Start a focus session with optional profile or flags
  status              Print current session status (for Waybar, Tmux, polybar)
  stats               Display today's metrics, streaks, and focus history
  history             View recent completed sessions
  projects            List, add, or archive focus projects
  themes              List all 19 built-in themes and Omarchy palette
  screenshot-preview  Render a terminal preview of any layout and theme
  recap               Show summary of today's focus sessions
  export              Export session data as JSON or CSV
  report              Generate Markdown weekly focus report
  doctor              Run system diagnostics and verify dependencies
  omarchy             Manage Omarchy Linux first-party integration
  completion          Generate shell completions (bash, zsh, fish)
  config              Inspect or initialize ~/.config/pomogo/config.toml
  version             Print version information
```

### Examples
- **Preview a rice layout with Omarchy theme**:
  ```bash
  pomogo screenshot-preview --layout terminal-rice --theme omarchy
  ```
- **Run system health check**:
  ```bash
  pomogo doctor
  ```
- **Waybar configuration**:
  ```json
  "custom/pomogo": {
      "exec": "pomogo status --format waybar",
      "return-type": "json",
      "interval": 1,
      "on-click": "pomogo",
      "signal": 10
  }
  ```

---

## Architecture & Code Layout

```
pomogo-rust/
├── Cargo.toml
├── src/
│   ├── main.rs            # Clap CLI router & argument parsing
│   ├── lib.rs             # Public library exports
│   ├── timer.rs           # Pure Pomodoro state machine with Clock trait
│   ├── session.rs         # Quick / Deep block planner and runner
│   ├── config.rs          # TOML configuration loader and profile resolver
│   ├── statefile.rs       # Atomic JSON state writing to XDG_RUNTIME_DIR
│   ├── restore.rs         # Recovery and prompt for interrupted sessions
│   ├── stats.rs           # Streaks, 7-day activity, and metrics calculation
│   ├── notify.rs          # D-Bus freedesktop notifications & Canberra audio
│   ├── devinfo.rs         # Git worktree/branch traversal & Tmux detection
│   ├── omarchy.rs         # Omarchy CLI actions & integration helpers
│   ├── theme/
│   │   ├── mod.rs         # 19 built-in palettes & theme resolver
│   │   ├── omarchy.rs     # Live colors.toml loader from Omarchy
│   │   ├── contrast.rs    # WCAG 2.1 relative luminance calculator
│   │   └── external.rs    # ~/.config/pomogo/themes/*.toml loader
│   ├── store/
│   │   ├── mod.rs         # SQLite connection & 7 schema migrations
│   │   ├── models.rs      # DbSession, Project, BlockStore models
│   │   └── export.rs      # JSON, CSV, and Markdown report generators
│   ├── integrations/
│   │   ├── status.rs      # Waybar, Tmux, JSON status line formatters
│   │   ├── dbus.rs        # systemd-logind LockedHint monitor
│   │   └── doctor.rs      # System health and environment check
│   ├── render/
│   │   ├── mod.rs         # Frame, DisplayState, and layout resolver
│   │   ├── bigclock.rs    # 5-row tall ANSI truecolor digit renderer
│   │   ├── borders.rs     # Box border styles (rounded, thick, double, etc.)
│   │   ├── widgets.rs     # Progress bars, dots, centering
│   │   ├── ambient.rs     # Particle simulation (stars, snow, rain, embers)
│   │   ├── text.rs        # Unicode width measuring & ANSI stripping
│   │   └── *.rs           # 11 layout modules
│   └── ui/
│       ├── mod.rs         # Crossterm terminal loop & application state
│       ├── keymap.rs      # Key bindings
│       └── screens/       # Dialogs (help, stats, picker, sound, recap, etc.)
└── contrib/
    ├── omarchy/           # Quickshell QML widget & setup script
    ├── waybar/            # Waybar modules
    ├── tmux/              # Tmux statusbar snippet
    └── starship/          # Starship prompt snippet
```

---

## License
MIT / Apache-2.0
