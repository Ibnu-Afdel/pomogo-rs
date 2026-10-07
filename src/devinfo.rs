// Development environment introspection (Git branch, Tmux session).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn find_git_branch(start_path: &Path) -> Option<String> {
    let mut current = start_path.to_path_buf();

    loop {
        let git_path = current.join(".git");
        if git_path.exists() {
            let head_path = if git_path.is_dir() {
                Some(git_path.join("HEAD"))
            } else {
                // Worktree or submodule: .git contains "gitdir: <path>"
                if let Ok(content) = fs::read_to_string(&git_path) {
                    let trimmed = content.trim();
                    if let Some(rest) = trimmed.strip_prefix("gitdir:") {
                        let rel = rest.trim();
                        let real_dir = if Path::new(rel).is_absolute() {
                            PathBuf::from(rel)
                        } else {
                            current.join(rel)
                        };
                        Some(real_dir.join("HEAD"))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            if let Some(h) = head_path {
                if let Ok(content) = fs::read_to_string(&h) {
                    let line = content.trim();
                    if let Some(ref_str) = line.strip_prefix("ref:") {
                        let target = ref_str.trim();
                        if let Some(branch) = target.strip_prefix("refs/heads/") {
                            return Some(branch.to_string());
                        }
                        return Some(target.to_string());
                    }
                    // Detached HEAD -> short SHA
                    if line.len() >= 7 {
                        return Some(line[..7].to_string());
                    }
                    return Some(line.to_string());
                }
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

pub fn get_tmux_session() -> Option<String> {
    if std::env::var("TMUX").is_err() {
        return None;
    }

    let out = Command::new("tmux")
        .args(["display-message", "-p", "#S"])
        .output()
        .ok()?;

    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }

    None
}

