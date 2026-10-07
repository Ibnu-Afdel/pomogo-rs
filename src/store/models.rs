use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbSession {
    pub id: i64,
    #[serde(rename = "type")]
    pub session_type: String, // "work", "short_break", "long_break"
    pub task: Option<String>,
    pub note: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub completed: bool,
    pub duration_secs: i64,
    pub project_id: Option<i64>,
    pub project_name: Option<String>,
    pub mode: Option<String>,
    pub block_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub archived: bool,
    pub icon: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStore {
    pub id: i64,
    pub mode: String,
    pub planned_secs: i64,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub completed: bool,
    pub pauses: i64,
}

