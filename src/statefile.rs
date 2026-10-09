// Persistent session state storage via JSON in $XDG_RUNTIME_DIR/pomogo/state.json.

use std::fs;
use std::path::PathBuf;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::session::{Mode, Runner};
use crate::timer::{SessionPhase, SessionState};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct State {
    pub version: usize,
    pub mode: String,
    pub state: String,
    pub session_type: String,
    pub ends_at: i64,
    pub paused: bool,
    pub remaining_secs: i64,
    pub pid: u32,
    pub session_count: usize,
    pub updated_at: i64,
    pub started_at: i64,
    pub total_secs: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<i64>,
    #[serde(default)]
    pub project_name: Option<String>,
    #[serde(default)]
    pub block_ends_at: i64,
    #[serde(default)]
    pub block_remaining_secs: i64,
    #[serde(default)]
    pub segment_index: usize,
    #[serde(default)]
    pub segment_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<i64>,
    #[serde(default)]
    pub planned_total_secs: i64,

    /// Focus seconds logged today, including the running segment.
    #[serde(default)]
    pub today_focus_secs: i64,
    /// Daily focus goal in seconds; 0 when no goal is set.
    #[serde(default)]
    pub daily_goal_secs: i64,
    #[serde(default)]
    pub water_today: usize,
    /// Body reminder currently on screen ("eyes", "water" or "stretch").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<String>,
}

/// Day-level details from the TUI that bars and scripts can show.
#[derive(Debug, Clone, Default)]
pub struct Companion {
    pub today_focus_secs: i64,
    pub daily_goal_secs: i64,
    pub water_today: usize,
    pub nudge: Option<String>,
}

pub fn xdg_runtime_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_RUNTIME_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("pomogo");
        }
    }
    PathBuf::from("/tmp").join("pomogo")
}

pub struct StateManager {
    pub state_path: PathBuf,
}

impl StateManager {
    pub fn new() -> Result<Self, String> {
        let dir = xdg_runtime_dir();
        fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create runtime dir {}: {}", dir.display(), e))?;

        Ok(Self {
            state_path: dir.join("state.json"),
        })
    }

    pub fn write(
        &self,
        runner: &Runner,
        task: Option<&str>,
        project_id: Option<i64>,
        project_name: Option<&str>,
        block_id: Option<i64>,
        companion: &Companion,
    ) -> Result<(), String> {
        let now = Utc::now();
        let remaining = if runner.timer.is_running && !runner.timer.is_paused {
            if let Some(ends) = runner.timer.ends_at {
                let r = ends - now;
                if r < chrono::Duration::zero() {
                    chrono::Duration::zero()
                } else {
                    r
                }
            } else {
                runner.timer.remaining_time
            }
        } else {
            runner.timer.remaining_time
        };

        let mut block_ends_at = 0;
        let mut block_remaining_secs = 0;
        if runner.block.mode == Mode::Deep {
            let block_rem = runner.block.remaining(remaining);
            block_remaining_secs = block_rem.num_seconds();
            block_ends_at = (now + block_rem).timestamp();
        }

        let state_val = State {
            version: 2,
            mode: runner.block.mode.as_str().to_string(),
            state: match runner.timer.state {
                SessionState::Idle => "idle",
                SessionState::Work => "work",
                SessionState::ShortBreak => "short_break",
                SessionState::LongBreak => "long_break",
            }
            .to_string(),
            session_type: match runner.timer.phase {
                SessionPhase::Work => "work",
                SessionPhase::ShortBreak => "short_break",
                SessionPhase::LongBreak => "long_break",
            }
            .to_string(),
            ends_at: runner.timer.ends_at.map(|t| t.timestamp()).unwrap_or(0),
            paused: runner.timer.is_paused,
            remaining_secs: remaining.num_seconds(),
            pid: std::process::id(),
            session_count: runner.timer.session_count,
            updated_at: now.timestamp(),
            started_at: runner.timer.started_at.map(|t| t.timestamp()).unwrap_or(0),
            total_secs: runner.block.current_segment.duration.num_seconds(),
            task: task.filter(|s| !s.is_empty()).map(|s| s.to_string()),
            project_id,
            project_name: project_name.filter(|s| !s.is_empty()).map(|s| s.to_string()),
            block_ends_at,
            block_remaining_secs,
            segment_index: runner.block.index,
            segment_count: runner.block.segments.len(),
            block_id,
            planned_total_secs: runner.block.planned_total.num_seconds(),
            today_focus_secs: companion.today_focus_secs,
            daily_goal_secs: companion.daily_goal_secs,
            water_today: companion.water_today,
            nudge: companion.nudge.clone(),
        };

        let json_str = serde_json::to_string_pretty(&state_val)
            .map_err(|e| format!("failed to serialize state: {}", e))?;

        let temp_path = self.state_path.with_extension("json.tmp");
        fs::write(&temp_path, json_str)
            .map_err(|e| format!("failed to write temp state: {}", e))?;

        fs::rename(&temp_path, &self.state_path)
            .map_err(|e| format!("failed to rename state file: {}", e))?;

        Ok(())
    }

    pub fn read(&self) -> Result<Option<State>, String> {
        if !self.state_path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.state_path)
            .map_err(|e| format!("failed to read state file: {}", e))?;

        let state: State = serde_json::from_str(&content)
            .map_err(|e| format!("failed to deserialize state: {}", e))?;

        Ok(Some(state))
    }

    pub fn remove(&self) -> Result<(), String> {
        if self.state_path.exists() {
            fs::remove_file(&self.state_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }
}

pub fn is_stale(state: &State) -> bool {
    let pid = state.pid;
    if pid == 0 {
        return true;
    }
    // Check if process exists in /proc/{pid}
    let proc_path = format!("/proc/{}", pid);
    !PathBuf::from(proc_path).exists()
}

pub fn is_expired(state: &State) -> bool {
    if state.paused {
        return false;
    }
    if state.ends_at <= 0 {
        return false;
    }
    let now = Utc::now().timestamp();
    now >= state.ends_at
}

/// Seconds left in the current segment, counted from `ends_at` while the
/// timer runs so readers don't depend on the last heartbeat.
pub fn live_remaining_secs(state: &State) -> i64 {
    if !state.paused && state.ends_at > 0 {
        (state.ends_at - Utc::now().timestamp()).max(0)
    } else {
        state.remaining_secs.max(0)
    }
}

/// Remote actions understood by a running TUI.
#[derive(Debug, Clone, Copy)]
pub enum RemoteAction {
    /// Start when idle, otherwise pause or resume.
    Toggle,
    /// Skip to the next segment.
    Skip,
}

/// Sends `action` to the PomoGo TUI recorded in the state file.
pub fn signal_running(action: RemoteAction) -> Result<u32, String> {
    let state = StateManager::new()?
        .read()?
        .ok_or_else(|| "PomoGo is not running".to_string())?;
    if is_stale(&state) || !is_pomogo_process(state.pid) {
        return Err("PomoGo is not running".to_string());
    }
    let sig = match action {
        RemoteAction::Toggle => libc::SIGUSR1,
        RemoteAction::Skip => libc::SIGUSR2,
    };
    // SAFETY: kill(2) only reads its two integer arguments.
    if unsafe { libc::kill(state.pid as libc::pid_t, sig) } != 0 {
        return Err(format!("failed to signal PomoGo: {}", std::io::Error::last_os_error()));
    }
    Ok(state.pid)
}

/// PID of another PomoGo TUI that is alive and still writing its heartbeat.
pub fn running_pid() -> Option<u32> {
    let state = StateManager::new().ok()?.read().ok()??;
    let fresh = Utc::now().timestamp() - state.updated_at <= 5;
    (fresh && state.pid != std::process::id() && !is_stale(&state) && is_pomogo_process(state.pid))
        .then_some(state.pid)
}

/// Guards against PID reuse: only signal a process that is actually PomoGo.
fn is_pomogo_process(pid: u32) -> bool {
    fs::read_to_string(format!("/proc/{}/comm", pid))
        .map(|comm| comm.trim().starts_with("pomogo"))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_statefile_read_write_remove() {
        let dir = tempdir().unwrap();
        let state_path = dir.path().join("state.json");
        let manager = StateManager {
            state_path: state_path.clone(),
        };

        assert!(manager.read().unwrap().is_none());

        let block = crate::session::Block::new_quick(
            chrono::Duration::minutes(25),
            chrono::Duration::minutes(5),
            chrono::Duration::minutes(15),
            4,
            false,
        );
        let runner = Runner::new(block);
        manager
            .write(&runner, Some("Test Task"), Some(42), Some("Project X"), None, &Companion::default())
            .unwrap();

        let loaded = manager.read().unwrap().expect("state loaded");
        assert_eq!(loaded.task.as_deref(), Some("Test Task"));
        assert_eq!(loaded.project_id, Some(42));
        assert_eq!(loaded.project_name.as_deref(), Some("Project X"));
        assert_eq!(loaded.mode, "quick");
        assert_eq!(loaded.state, "idle");

        manager.remove().unwrap();
        assert!(manager.read().unwrap().is_none());
    }

    #[test]
    fn test_is_stale_and_expired() {
        let mut state = State {
            version: 2,
            mode: "quick".to_string(),
            state: "work".to_string(),
            session_type: "work".to_string(),
            ends_at: Utc::now().timestamp() - 10,
            paused: false,
            remaining_secs: 0,
            pid: std::process::id(),
            session_count: 1,
            updated_at: Utc::now().timestamp(),
            started_at: Utc::now().timestamp() - 1500,
            total_secs: 1500,
            task: None,
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

        // Self process is not stale
        assert!(!is_stale(&state));
        // Dead PID is stale
        state.pid = 999_999_999;
        assert!(is_stale(&state));

        // Expired because ends_at was in past
        assert!(is_expired(&state));

        // Paused is not expired
        state.paused = true;
        assert!(!is_expired(&state));
    }

    #[test]
    fn test_live_remaining_secs() {
        let now = Utc::now().timestamp();
        let mut state: State = serde_json::from_value(serde_json::json!({
            "version": 2, "mode": "quick", "state": "work", "session_type": "work",
            "ends_at": now + 90, "paused": false, "remaining_secs": 300, "pid": 1,
            "session_count": 0, "updated_at": now - 30, "started_at": now - 60,
            "total_secs": 1500
        }))
        .unwrap();
        let live = live_remaining_secs(&state);
        assert!((89..=90).contains(&live), "running timers count down from ends_at");
        state.paused = true;
        assert_eq!(live_remaining_secs(&state), 300);
        state.paused = false;
        state.ends_at = now - 5;
        assert_eq!(live_remaining_secs(&state), 0);
    }
}


