// Session lock detection: Hyprland's session lock (Omarchy 4's lock screen
// never sets logind's LockedHint) and systemd-logind for everything else.

use std::process::Command;

pub fn is_session_locked() -> bool {
    hyprland_session_locked() || logind_locked_hint()
}

/// Hyprland has no direct lock query, but an active ext-session-lock shows up
/// as LOCK in a monitor's solitaryBlockedBy. Same check as Omarchy's
/// omarchy-hyprland-session-locked.
fn hyprland_session_locked() -> bool {
    if std::env::var_os("HYPRLAND_INSTANCE_SIGNATURE").is_none() {
        return false;
    }
    let out = match Command::new("hyprctl").args(["-j", "monitors"]).output() {
        Ok(o) if o.status.success() => o.stdout,
        _ => return false,
    };
    monitors_report_lock(&String::from_utf8_lossy(&out))
}

fn monitors_report_lock(json: &str) -> bool {
    let monitors: Vec<serde_json::Value> = serde_json::from_str(json).unwrap_or_default();
    monitors.iter().any(|m| {
        m.get("solitaryBlockedBy")
            .and_then(|v| v.as_array())
            .map(|reasons| reasons.iter().any(|r| r.as_str() == Some("LOCK")))
            .unwrap_or(false)
    })
}

fn logind_locked_hint() -> bool {
    let mut sessions = vec!["self".to_string()];
    if let Ok(id) = std::env::var("XDG_SESSION_ID") {
        sessions.push(id);
    }
    for session in sessions {
        if let Ok(out) = Command::new("loginctl")
            .args(["show-session", &session, "-p", "LockedHint", "--value"])
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_monitors_report_lock() {
        assert!(monitors_report_lock(r#"[{"solitaryBlockedBy":["WINDOWED"]},{"solitaryBlockedBy":["LOCK"]}]"#));
        assert!(!monitors_report_lock(r#"[{"solitaryBlockedBy":["WINDOWED","CANDIDATE"]}]"#));
        assert!(!monitors_report_lock(r#"[{"name":"eDP-1"}]"#));
        assert!(!monitors_report_lock("not json"));
    }
}
