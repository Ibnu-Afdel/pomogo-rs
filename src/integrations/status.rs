// Status bar formatters for Waybar, Tmux, JSON, and scripts.

use chrono::{DateTime, Local};
use serde::Serialize;
use crate::statefile::{is_expired, is_stale, live_remaining_secs, State};

#[derive(Debug, Serialize)]
pub struct WaybarOutput {
    pub text: String,
    pub class: String,
    pub tooltip: String,
}

pub fn format_status(state: Option<&State>, format: &str) -> Result<String, String> {
    if is_idle(state) {
        return match format {
            "waybar" => {
                let out = WaybarOutput {
                    text: "".to_string(),
                    class: "pomogo-idle".to_string(),
                    tooltip: "Idle".to_string(),
                };
                serde_json::to_string(&out).map_err(|e| e.to_string())
            }
            "tmux" => Ok("".to_string()),
            "json" => Ok(r#"{"state":"idle"}"#.to_string()),
            _ => Ok("Idle".to_string()),
        };
    }

    let st = state.unwrap();

    let time_str = if st.mode == "deep" && st.block_remaining_secs > 0 {
        let hrs = st.block_remaining_secs / 3600;
        let mins = (st.block_remaining_secs % 3600) / 60;
        let secs = st.block_remaining_secs % 60;
        if hrs > 0 {
            format!("{}:{:02}:{:02}", hrs, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        }
    } else {
        let remaining = live_remaining_secs(st);
        format!("{:02}:{:02}", remaining / 60, remaining % 60)
    };

    let (icon, class) = if st.paused {
        ("⏸️", "pomogo-paused")
    } else if st.session_type == "work" {
        ("🍅", "pomogo-work")
    } else {
        ("☕", "pomogo-break")
    };

    let display_str = format!("{} {}", icon, time_str);

    let end_time_str = if st.ends_at > 0 {
        DateTime::from_timestamp(st.ends_at, 0)
            .map(|dt| dt.with_timezone(&Local).format("%H:%M").to_string())
            .unwrap_or_else(|| "--:--".to_string())
    } else {
        "--:--".to_string()
    };

    let mut phase_name = st.session_type.as_str();
    if phase_name == "short_break" {
        phase_name = "break";
    } else if phase_name == "long_break" {
        phase_name = "long break";
    }

    let capitalized_phase = capitalize(phase_name);
    let mut tooltip = format!("{} session ends at {}", capitalized_phase, end_time_str);
    if let Some(t) = &st.task {
        if !t.is_empty() {
            tooltip = format!("Task: {}\n{}", t, tooltip);
        }
    }
    if st.daily_goal_secs > 0 {
        tooltip.push_str(&format!(
            "\nToday: {} of {}",
            fmt_hm(st.today_focus_secs),
            fmt_hm(st.daily_goal_secs)
        ));
    }

    match format {
        "waybar" => {
            let out = WaybarOutput {
                text: display_str,
                class: class.to_string(),
                tooltip,
            };
            serde_json::to_string(&out).map_err(|e| e.to_string())
        }
        "tmux" => Ok(display_str),
        "json" => serde_json::to_string(st).map_err(|e| e.to_string()),
        _ => {
            let mut res = format!("{} · {}", display_str, st.session_type);
            if st.paused {
                res.push_str(" (paused)");
            }
            if let Some(t) = &st.task {
                if !t.is_empty() {
                    res.push_str(&format!(" [{}]", t));
                }
            }
            Ok(res)
        }
    }
}

fn is_idle(state: Option<&State>) -> bool {
    match state {
        None => true,
        Some(st) => {
            if st.state == "idle" || st.state.is_empty() {
                return true;
            }
            if is_stale(st) {
                return true;
            }
            if is_expired(st) {
                return true;
            }
            false
        }
    }
}

fn fmt_hm(secs: i64) -> String {
    let mins = secs.max(0) / 60;
    match (mins / 60, mins % 60) {
        (0, m) => format!("{}m", m),
        (h, 0) => format!("{}h", h),
        (h, m) => format!("{}h {}m", h, m),
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_status_idle() {
        assert_eq!(format_status(None, "tmux").unwrap(), "");
        assert_eq!(format_status(None, "json").unwrap(), r#"{"state":"idle"}"#);
        assert_eq!(format_status(None, "default").unwrap(), "Idle");

        let waybar_str = format_status(None, "waybar").unwrap();
        let val: serde_json::Value = serde_json::from_str(&waybar_str).unwrap();
        assert_eq!(val["class"], "pomogo-idle");
    }

    #[test]
    fn test_format_status_running() {
        let state = State {
            version: 2,
            mode: "quick".to_string(),
            state: "work".to_string(),
            session_type: "work".to_string(),
            ends_at: chrono::Utc::now().timestamp() + 1500,
            paused: false,
            remaining_secs: 1500,
            pid: std::process::id(),
            session_count: 1,
            updated_at: chrono::Utc::now().timestamp(),
            started_at: chrono::Utc::now().timestamp(),
            total_secs: 1500,
            task: Some("Refactor".to_string()),
            project_id: None,
            project_name: None,
            block_ends_at: 0,
            block_remaining_secs: 0,
            segment_index: 0,
            segment_count: 1,
            block_id: None,
            planned_total_secs: 1500,
            ..Default::default()
        };

        let tmux = format_status(Some(&state), "tmux").unwrap();
        assert_eq!(tmux, "🍅 25:00");

        let waybar = format_status(Some(&state), "waybar").unwrap();
        let val: serde_json::Value = serde_json::from_str(&waybar).unwrap();
        assert_eq!(val["text"], "🍅 25:00");
        assert_eq!(val["class"], "pomogo-work");
        assert!(val["tooltip"].as_str().unwrap().contains("Task: Refactor"));

        let with_goal = State { today_focus_secs: 95 * 60, daily_goal_secs: 4 * 3600, ..state };
        let waybar = format_status(Some(&with_goal), "waybar").unwrap();
        assert!(waybar.contains("Today: 1h 35m of 4h"));
    }
}


