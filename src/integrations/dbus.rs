// D-Bus and systemd-logind session lock detection.

use std::process::Command;

pub fn is_session_locked() -> bool {
    // 1. Try querying loginctl
    if let Ok(out) = Command::new("loginctl")
        .args(["show-session", "self", "-p", "LockedHint", "--value"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
            if s == "yes" || s == "true" {
                return true;
            }
            if s == "no" || s == "false" {
                return false;
            }
        }
    }

    // 2. Try with XDG_SESSION_ID if self didn't work
    if let Ok(sess_id) = std::env::var("XDG_SESSION_ID") {
        if let Ok(out) = Command::new("loginctl")
            .args(["show-session", &sess_id, "-p", "LockedHint", "--value"])
            .output()
        {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_lowercase();
                return s == "yes" || s == "true";
            }
        }
    }

    false
}

