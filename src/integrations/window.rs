// Finding and focusing the terminal window a PomoGo TUI runs in.

use std::fs;
use std::process::Command;

/// Brings the PomoGo running as `pid` to the front: focuses the Hyprland
/// window of the terminal it runs in, and inside tmux first switches an
/// attached client to its pane. Returns false outside Hyprland or when no
/// window shows it (e.g. a detached tmux session).
pub fn focus_window_of(pid: u32) -> bool {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
        return false;
    }
    match tmux_client_showing(pid) {
        Some(client_pid) => focus_window_owning(client_pid),
        None => focus_window_owning(pid),
    }
}

/// When `pid` runs inside tmux, switches an attached client to its pane
/// and returns that client's pid (a child of the terminal window).
fn tmux_client_showing(pid: u32) -> Option<u32> {
    let env = fs::read(format!("/proc/{}/environ", pid)).ok()?;
    let var = |name: &str| {
        env.split(|b| *b == 0).find_map(|kv| {
            let kv = std::str::from_utf8(kv).ok()?;
            kv.strip_prefix(name)?.strip_prefix('=').map(str::to_string)
        })
    };
    // TMUX is "<socket>,<server pid>,<session>"; TMUX_PANE is "%<id>".
    let socket = var("TMUX")?.split(',').next()?.to_string();
    let pane = var("TMUX_PANE")?;
    let tmux = |args: &[&str]| -> Option<String> {
        let out = Command::new("tmux").arg("-S").arg(&socket).args(args).output().ok()?;
        out.status.success().then(|| String::from_utf8_lossy(&out.stdout).into_owned())
    };

    let session = tmux(&["display-message", "-p", "-t", &pane, "#{session_id}"])?;
    let session = session.trim();
    let clients = tmux(&["list-clients", "-F", "#{client_pid} #{client_tty} #{session_id}"])?;
    let mut clients: Vec<(&str, &str, &str)> = clients
        .lines()
        .filter_map(|l| {
            let mut f = l.split(' ');
            Some((f.next()?, f.next()?, f.next()?))
        })
        .collect();
    // Prefer a client already on PomoGo's session so other sessions stay put.
    clients.sort_by_key(|(_, _, s)| *s != session);
    let (client_pid, tty, _) = *clients.first()?;

    tmux(&["switch-client", "-c", tty, "-t", &pane])?;
    tmux(&["select-window", "-t", &pane])?;
    tmux(&["select-pane", "-t", &pane])?;
    client_pid.parse().ok()
}

/// Focuses the Hyprland window whose process is `pid` or one of its
/// ancestors.
fn focus_window_owning(pid: u32) -> bool {
    let out = match Command::new("hyprctl").args(["clients", "-j"]).output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => return false,
    };
    let clients: Vec<serde_json::Value> = serde_json::from_slice(&out).unwrap_or_default();
    let ancestors = ancestors(pid);
    let Some(address) = clients.iter().find_map(|c| {
        let owner = c.get("pid")?.as_u64()? as u32;
        ancestors.contains(&owner).then(|| c.get("address")?.as_str().map(str::to_string))?
    }) else {
        return false;
    };

    // Hyprland's Lua dispatcher first, then the classic one (same as
    // omarchy-launch-or-focus).
    let lua = format!("hl.dsp.focus({{ window = \"address:{}\" }})", address);
    let ok = |args: &[&str]| {
        Command::new("hyprctl")
            .args(args)
            .output()
            .map(|o| o.status.success() && !String::from_utf8_lossy(&o.stdout).contains("error"))
            .unwrap_or(false)
    };
    ok(&["dispatch", &lua]) || ok(&["dispatch", "focuswindow", &format!("address:{}", address)])
}

/// `pid` and its parent chain up to init.
fn ancestors(pid: u32) -> Vec<u32> {
    let mut chain = Vec::new();
    let mut cur = pid;
    while cur > 1 && chain.len() < 64 {
        chain.push(cur);
        cur = match parent_of(cur) {
            Some(p) => p,
            None => break,
        };
    }
    chain
}

fn parent_of(pid: u32) -> Option<u32> {
    let stat = fs::read_to_string(format!("/proc/{}/stat", pid)).ok()?;
    // The command name may contain spaces or parens; fields resume after the last ')'.
    let rest = &stat[stat.rfind(')')? + 2..];
    rest.split_whitespace().nth(1)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestors_start_with_self_and_reach_a_parent() {
        let me = std::process::id();
        let chain = ancestors(me);
        assert_eq!(chain[0], me);
        assert_eq!(chain.get(1).copied(), Some(std::os::unix::process::parent_id()));
    }
}
