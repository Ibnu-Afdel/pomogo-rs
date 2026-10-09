# PomoGo

A calm focus companion for the Linux terminal. Set it up once, press enter,
and get on with your work: PomoGo runs your focus and break rhythm, reminds
you to rest your eyes, drink water and stretch, and keeps a record of every
session.

Built for [Omarchy](https://omarchy.org/) first and at home on any other
distro. Written in Rust; it reads and continues the history of the original
Go release.

## Highlights

- **Set up once.** The first launch asks five questions (rhythm, daily goal,
  autopilot, reminders, notifications). After that, `pomogo` just runs.
- **Autopilot.** Focus rolls into breaks and back on its own, Pomodoro style,
  or plan a 1–4 hour **deep focus** block.
- **Looks after you.** While you focus: rest your eyes every 20 minutes, a
  glass of water every 45, a stretch every hour. Breaks reset what they
  already cover, and every break suggests one concrete thing to do.
- **A daily goal.** Today's focus time, the goal, your streak and glasses of
  water, on screen and in your bar.
- **Remembers everything.** Sessions, tasks, projects, streaks, recaps,
  CSV/JSON export and Markdown weekly reports, in a local SQLite database.
- **Omarchy-native.** A bar widget for `omarchy-shell`, colors that follow
  `omarchy theme set` live, pause on lock, and Omarchy's TUI launcher.
- **Works anywhere.** Waybar, tmux and Starship modules, a desktop entry,
  freedesktop notifications and sounds.
- **Yours to style.** A clean default `focus` screen plus 11 other layouts,
  19 built-in themes (and your own), and ambient effects.

## Install

**Arch / Omarchy** (from the AUR; replaces the Go `pomogo-bin`):

```sh
yay -S pomogo
pomogo omarchy install    # on Omarchy: add the bar widget
```

**Any distro**: you need a Rust toolchain and a C compiler (for the bundled SQLite):

| Distro       | Command                                  |
|--------------|------------------------------------------|
| Omarchy/Arch | `sudo pacman -S --needed rust base-devel` |
| Fedora       | `sudo dnf install cargo gcc`              |
| Other        | [rustup.rs](https://rustup.rs) and your distro's C compiler |

Then:

```sh
git clone https://github.com/Ibnu-Afdel/pomogo-rs.git && cd pomogo-rs
./install.sh
```

`install.sh` builds a release binary into `~/.local/bin/pomogo`, installs
bash/zsh/fish completions and a desktop entry, and on Omarchy installs and
enables the bar widget. `./install.sh --uninstall` removes all of it and keeps
your session history. Set `PREFIX` to install somewhere other than
`~/.local`.

If the Go release is still installed (for example `pomogo-bin` from the AUR),
the installer tells you which `pomogo` wins on your `PATH`. On Omarchy
`~/.local/bin` comes first, so the Rust build is used. The AUR package
replaces `pomogo-bin` directly.

## Omarchy

`./install.sh` does all of this for you. To manage it by hand:

```sh
pomogo omarchy install       # add the bar widget to omarchy-shell and enable it
pomogo omarchy status        # check detection, palette and widget state
pomogo omarchy keybindings   # print suggested Hyprland bindings
pomogo omarchy uninstall     # remove the widget from the bar
```

The widget is also published on its own as
[omarchy-pomogo](https://github.com/Ibnu-Afdel/omarchy-pomogo), so it can
be managed by Omarchy's plugin manager instead:
`omarchy plugin add https://github.com/Ibnu-Afdel/omarchy-pomogo.git --enable`.

**Bar widget** (`pomogo.timer`). It shows the running segment, dims while
paused and hides when PomoGo is closed:

- click: open or focus PomoGo
- right click: start, pause or resume
- middle click: skip to the next segment

Move it like any other widget, for example
`omarchy bar move pomogo.timer --section right`. Its options (icon only or
icon and time, show the task name, show it while idle) are in Omarchy's bar
settings.

**Theme.** With the default `theme = "auto"`, PomoGo uses the active Omarchy
palette and repaints as soon as you switch themes.

**Keybindings.** Omarchy 4 configures Hyprland in Lua. Add these to
`~/.config/hypr/bindings.lua` (the keys are unbound by default):

```lua
o.bind("SUPER + ALT + P", "PomoGo", { tui = "pomogo", focus = true })
o.bind("SUPER + SHIFT + ALT + P", "PomoGo start/pause", "pomogo toggle")

-- Optional: float the PomoGo window instead of tiling it.
o.window("org.omarchy.pomogo", { tag = "+floating-window" })
```

**Lock screen.** With `pause_on_lock = true`, the timer pauses while
Omarchy's lock screen is up and resumes when you unlock.

## Other desktops

- **Waybar**: see [`contrib/waybar/config.jsonc`](contrib/waybar/config.jsonc).
- **tmux**: see [`contrib/tmux/pomogo.tmux`](contrib/tmux/pomogo.tmux).
- **Starship**: see [`contrib/starship/pomogo.toml`](contrib/starship/pomogo.toml).
- **Any compositor or WM**: bind `pomogo toggle` and `pomogo skip` to keys.
  They control the running TUI from anywhere.

Notifications go through the freedesktop notification service. Sounds use
`canberra-gtk-play` when present, otherwise `pw-play` or `paplay` with the
freedesktop sound theme. Pause-on-lock reads logind's `LockedHint` on
GNOME, KDE and others. Run `pomogo doctor` to see what's available.

## Keys

| Key | Action |
|---|---|
| `enter` | Start, pause or resume (`s` and `space` also work) |
| `n` | Skip segment |
| `w` | Log a glass of water |
| `esc` | Dismiss a reminder |
| `r` | Reset |
| `t` / `p` | Set task / project (with autocomplete) |
| `d` | Choose a Deep Focus duration |
| `tab` | Stats |
| `y` | Copy stats to the clipboard |
| `T` / `L` / `e` | Cycle theme / layout / ambient effect |
| `v` | Cycle the activity label |
| `a` | Sound picker |
| `S` | Zen mode (hide hints, for screenshots) |
| `?` | Help |
| `q`, `ctrl+c` | Quit (the session is saved and offered for restore next time) |

## Commands

```text
pomogo [--theme T] [--layout L] [--effects E] [--task T] [--project P] [--work MIN] [--break-time MIN] [--zen]
pomogo setup                     answer the setup questions again
pomogo start [profile|project]   start with a profile from config.toml or a project
pomogo toggle | skip             control the running TUI (bars, keybindings)
pomogo status [--format default|waybar|tmux|json]
pomogo stats [--week|--month] | history | recap
pomogo report [--start DATE --end DATE]
pomogo export [--format json|csv] [--start DATE --end DATE]
pomogo projects [list|add|archive]
pomogo themes | screenshot-preview
pomogo config init [--force]
pomogo omarchy [status|install|uninstall|keybindings|install-desktop]
pomogo doctor | completion <shell> | version
```

## Configuration

`pomogo setup` (or the first launch) writes a short, commented
`~/.config/pomogo/config.toml`. `pomogo config init` writes the same file with
defaults. The important settings:

```toml
work_duration = 25
short_break_duration = 5
long_break_duration = 15
sessions_before_long_break = 4

autopilot = true            # focus and breaks start on their own
daily_goal_minutes = 240    # 0 hides the goal

[wellness]                  # minutes of focus between reminders, 0 = off
eyes_minutes = 20
water_minutes = 45
stretch_minutes = 60

# theme = "auto"            # Omarchy palette on Omarchy, tokyo-night elsewhere
# layout = "focus"

[profiles.writing]          # pomogo start writing
work_duration = 50
project = "Writing"
```

Custom themes go in `~/.config/pomogo/themes/*.toml`.

## Files

| Path | Contents |
|---|---|
| `~/.config/pomogo/config.toml` | settings and profiles |
| `~/.local/share/pomogo/pomogo.db` | sessions, blocks, projects and water log (shared with the Go release) |
| `$XDG_RUNTIME_DIR/pomogo/state.json` | live session state, updated every second while the TUI runs |

## Development

```sh
cargo test
cargo run -- screenshot-preview --layout dashboard --theme auto
```

The bar widget lives in [`contrib/omarchy/plugin`](contrib/omarchy/plugin) and
is embedded into the binary at build time. Check it with
`omarchy plugin validate contrib/omarchy/plugin` and publish it to the
standalone repo with `scripts/publish-omarchy-plugin.sh`.

The AUR recipe is in [`contrib/aur`](contrib/aur). To release: bump the
version in `Cargo.toml` and the plugin manifest, tag `vX.Y.Z`, then update
`pkgver`, run `updpkgsums` and `makepkg --printsrcinfo > .SRCINFO`, and push
both files to `ssh://aur@aur.archlinux.org/pomogo.git`.

## License

MIT
