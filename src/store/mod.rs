pub mod export;
pub mod models;

use std::fs;
use std::path::Path;
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

pub use models::{BlockStore, DbSession, Project};

/// Parses a stored timestamp. Rows written by this version are RFC 3339;
/// the Go release stored `time.Time.String()`, e.g.
/// `2026-07-14 10:21:57.208608329 +0300 EAT m=+128.698439020`, and
/// go-sqlite3 style `2026-07-14 10:21:57.2086+03:00` also turns up.
pub fn parse_db_time(raw: &str) -> Option<DateTime<Utc>> {
    let s = raw.trim();
    if let Ok(dt) = DateTime::parse_from_rfc3339(s) {
        return Some(dt.with_timezone(&Utc));
    }

    // Go's String(): "<date> <time> <offset> <zone abbr> [m=±monotonic]".
    let mut parts = s.split_whitespace();
    if let (Some(date), Some(time), Some(offset)) = (parts.next(), parts.next(), parts.next()) {
        let candidate = format!("{} {} {}", date, time, offset);
        if let Ok(dt) = DateTime::parse_from_str(&candidate, "%Y-%m-%d %H:%M:%S%.f %z") {
            return Some(dt.with_timezone(&Utc));
        }
    }

    for fmt in ["%Y-%m-%d %H:%M:%S%.f%:z", "%Y-%m-%dT%H:%M:%S%.f%:z"] {
        if let Ok(dt) = DateTime::parse_from_str(s, fmt) {
            return Some(dt.with_timezone(&Utc));
        }
    }
    None
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn new(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("failed to create db directory: {}", e))?;
        }

        let conn = Connection::open(path)
            .map_err(|e| format!("failed to open sqlite database {}: {}", path.display(), e))?;

        let mut store = Store { conn };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory()
            .map_err(|e| format!("failed to open in-memory sqlite db: {}", e))?;
        let mut store = Store { conn };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&mut self) -> Result<(), String> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS schema_migrations (version INTEGER PRIMARY KEY);",
                [],
            )
            .map_err(|e| format!("failed to create schema_migrations: {}", e))?;

        let migrations = [
            "CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                type TEXT NOT NULL,
                task TEXT,
                note TEXT,
                started_at DATETIME NOT NULL,
                ended_at DATETIME,
                completed INTEGER NOT NULL,
                duration_secs INTEGER NOT NULL
            );",
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                archived INTEGER NOT NULL DEFAULT 0
            );",
            "ALTER TABLE sessions ADD COLUMN project_id INTEGER REFERENCES projects(id);",
            "CREATE TABLE IF NOT EXISTS blocks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                mode TEXT NOT NULL,
                planned_secs INTEGER NOT NULL,
                started_at DATETIME NOT NULL,
                ended_at DATETIME,
                completed INTEGER NOT NULL,
                pauses INTEGER DEFAULT 0
            );",
            "ALTER TABLE sessions ADD COLUMN mode TEXT;",
            "ALTER TABLE sessions ADD COLUMN block_id INTEGER REFERENCES blocks(id);",
            "ALTER TABLE projects ADD COLUMN icon TEXT DEFAULT '';",
        ];

        for (i, sql) in migrations.iter().enumerate() {
            let version = (i + 1) as i64;
            let count: i64 = self
                .conn
                .query_row(
                    "SELECT COUNT(*) FROM schema_migrations WHERE version = ?",
                    params![version],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if count > 0 {
                continue;
            }

            let tx = self
                .conn
                .transaction()
                .map_err(|e| format!("failed to begin tx for migration {}: {}", version, e))?;

            tx.execute(sql, [])
                .map_err(|e| format!("failed to run migration {}: {}", version, e))?;

            tx.execute(
                "INSERT INTO schema_migrations (version) VALUES (?)",
                params![version],
            )
            .map_err(|e| format!("failed to record migration {}: {}", version, e))?;

            tx.commit()
                .map_err(|e| format!("failed to commit migration {}: {}", version, e))?;
        }

        Ok(())
    }

    pub fn get_current_schema_version(&self) -> Result<i64, String> {
        let version: i64 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
                [],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        Ok(version)
    }

    pub fn save_session(&mut self, sess: &mut DbSession) -> Result<(), String> {
        let started_str = sess.started_at.to_rfc3339();
        let ended_str = sess.ended_at.map(|t| t.to_rfc3339());
        let completed_int = if sess.completed { 1 } else { 0 };

        self.conn
            .execute(
                "INSERT INTO sessions (type, task, note, started_at, ended_at, completed, duration_secs, project_id, mode, block_id)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![
                    sess.session_type,
                    sess.task,
                    sess.note,
                    started_str,
                    ended_str,
                    completed_int,
                    sess.duration_secs,
                    sess.project_id,
                    sess.mode,
                    sess.block_id,
                ],
            )
            .map_err(|e| format!("failed to save session: {}", e))?;

        sess.id = self.conn.last_insert_rowid();
        Ok(())
    }

    pub fn get_sessions(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<DbSession>, String> {
        // Timestamps are stored as text in more than one format (the Go
        // release wrote time.Time.String()), so string comparison in SQL
        // cannot select a time range. Parse every row and filter here.
        let mut stmt = self
            .conn
            .prepare(
                "SELECT s.id, s.type, s.task, s.note, s.started_at, s.ended_at, s.completed, s.duration_secs, s.project_id, p.name, s.mode, s.block_id
                 FROM sessions s
                 LEFT JOIN projects p ON s.project_id = p.id",
            )
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let started_str: String = row.get(4)?;
                let Some(started_at) = parse_db_time(&started_str) else {
                    return Ok(None);
                };

                let ended_str: Option<String> = row.get(5)?;
                let ended_at = ended_str.as_deref().and_then(parse_db_time);

                let completed_int: i32 = row.get(6)?;

                Ok(Some(DbSession {
                    id: row.get(0)?,
                    session_type: row.get(1)?,
                    task: row.get(2)?,
                    note: row.get(3)?,
                    started_at,
                    ended_at,
                    completed: completed_int == 1,
                    duration_secs: row.get(7)?,
                    project_id: row.get(8)?,
                    project_name: row.get(9)?,
                    mode: row.get(10)?,
                    block_id: row.get(11)?,
                }))
            })
            .map_err(|e| e.to_string())?;

        let mut sessions = Vec::new();
        for r in rows {
            if let Some(sess) = r.map_err(|e| e.to_string())? {
                if sess.started_at >= start && sess.started_at <= end {
                    sessions.push(sess);
                }
            }
        }
        sessions.sort_by_key(|s| (s.started_at, s.id));
        Ok(sessions)
    }

    pub fn create_project(&mut self, p: &mut Project) -> Result<(), String> {
        let archived_int = if p.archived { 1 } else { 0 };
        self.conn
            .execute(
                "INSERT INTO projects (name, color, archived, icon) VALUES (?, ?, ?, ?)",
                params![p.name, p.color, archived_int, p.icon],
            )
            .map_err(|e| format!("failed to insert project: {}", e))?;

        p.id = self.conn.last_insert_rowid();
        Ok(())
    }

    pub fn get_projects(&self) -> Result<Vec<Project>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color, archived, icon FROM projects ORDER BY name ASC")
            .map_err(|e| e.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let archived_int: i32 = row.get(3)?;
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    archived: archived_int == 1,
                    icon: row.get(4).unwrap_or_default(),
                })
            })
            .map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    pub fn get_project_by_name(&self, name: &str) -> Result<Option<Project>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, color, archived, icon FROM projects WHERE name = ?")
            .map_err(|e| e.to_string())?;

        let res = stmt
            .query_row(params![name], |row| {
                let archived_int: i32 = row.get(3)?;
                Ok(Project {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    color: row.get(2)?,
                    archived: archived_int == 1,
                    icon: row.get(4).unwrap_or_default(),
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(res)
    }

    pub fn archive_project(&self, name: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE projects SET archived = 1 WHERE name = ?",
                params![name],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn unarchive_project(&self, name: &str) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE projects SET archived = 0 WHERE name = ?",
                params![name],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_unique_tasks(&self, project_id: Option<i64>) -> Result<Vec<String>, String> {
        let (sql, params_vec): (&str, Vec<rusqlite::types::Value>) = match project_id {
            Some(pid) => (
                "SELECT DISTINCT task FROM sessions WHERE task IS NOT NULL AND task != '' AND project_id = ? ORDER BY started_at DESC LIMIT 50",
                vec![pid.into()],
            ),
            None => (
                "SELECT DISTINCT task FROM sessions WHERE task IS NOT NULL AND task != '' AND project_id IS NULL ORDER BY started_at DESC LIMIT 50",
                vec![],
            ),
        };

        let mut stmt = self.conn.prepare(sql).map_err(|e| e.to_string())?;
        let rows = stmt
            .query_map(rusqlite::params_from_iter(params_vec), |row| {
                row.get::<_, String>(0)
            })
            .map_err(|e| e.to_string())?;

        let mut tasks = Vec::new();
        for r in rows {
            tasks.push(r.map_err(|e| e.to_string())?);
        }
        Ok(tasks)
    }

    pub fn delete_task_name(&self, task: &str, project_id: Option<i64>) -> Result<(), String> {
        match project_id {
            Some(pid) => {
                self.conn.execute(
                    "UPDATE sessions SET task = NULL WHERE task = ? AND project_id = ?",
                    params![task, pid],
                ).map_err(|e| e.to_string())?;
            }
            None => {
                self.conn.execute(
                    "UPDATE sessions SET task = NULL WHERE task = ? AND project_id IS NULL",
                    params![task],
                ).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    pub fn create_block(&mut self, b: &mut BlockStore) -> Result<(), String> {
        let started_str = b.started_at.to_rfc3339();
        let completed_int = if b.completed { 1 } else { 0 };

        self.conn
            .execute(
                "INSERT INTO blocks (mode, planned_secs, started_at, completed, pauses) VALUES (?, ?, ?, ?, ?)",
                params![b.mode, b.planned_secs, started_str, completed_int, b.pauses],
            )
            .map_err(|e| format!("failed to insert block: {}", e))?;

        b.id = self.conn.last_insert_rowid();
        Ok(())
    }

    pub fn finish_block(&self, id: i64, completed: bool, ended_at: DateTime<Utc>) -> Result<(), String> {
        let completed_int = if completed { 1 } else { 0 };
        let ended_str = ended_at.to_rfc3339();

        self.conn
            .execute(
                "UPDATE blocks SET completed = ?, ended_at = ? WHERE id = ?",
                params![completed_int, ended_str, id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn increment_block_pauses(&self, id: i64) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE blocks SET pauses = pauses + 1 WHERE id = ?",
                params![id],
            )
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_last_block(&self) -> Result<Option<BlockStore>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, mode, planned_secs, started_at, ended_at, completed, pauses FROM blocks ORDER BY id DESC LIMIT 1")
            .map_err(|e| e.to_string())?;

        let res = stmt
            .query_row([], |row| {
                let started_str: String = row.get(3)?;
                let started_at = parse_db_time(&started_str).unwrap_or(DateTime::UNIX_EPOCH);

                let ended_str: Option<String> = row.get(4)?;
                let ended_at = ended_str.as_deref().and_then(parse_db_time);

                let completed_int: i32 = row.get(5)?;

                Ok(BlockStore {
                    id: row.get(0)?,
                    mode: row.get(1)?,
                    planned_secs: row.get(2)?,
                    started_at,
                    ended_at,
                    completed: completed_int == 1,
                    pauses: row.get(6)?,
                })
            })
            .optional()
            .map_err(|e| e.to_string())?;

        Ok(res)
    }

    pub fn export_sessions(
        &self,
        format: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<String, String> {
        let sessions = self.get_sessions(start, end)?;
        match format {
            "json" => export::export_sessions_json(&sessions),
            "csv" => export::export_sessions_csv(&sessions),
            other => Err(format!("unsupported export format: {}", other)),
        }
    }

    pub fn generate_markdown_report(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<String, String> {
        let sessions = self.get_sessions(start, end)?;
        Ok(export::generate_markdown_report(&sessions, start, end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_parse_db_time_formats() {
        let expected = DateTime::parse_from_rfc3339("2026-07-14T07:21:57.208608329Z")
            .unwrap()
            .with_timezone(&Utc);
        assert_eq!(parse_db_time("2026-07-14T10:21:57.208608329+03:00"), Some(expected));
        assert_eq!(
            parse_db_time("2026-07-14 10:21:57.208608329 +0300 EAT m=+128.698439020"),
            Some(expected)
        );
        assert_eq!(parse_db_time("2026-07-14 10:21:57.208608329 +0300 EAT"), Some(expected));
        assert_eq!(parse_db_time("2026-07-14 10:21:57.208608329+03:00"), Some(expected));
        assert_eq!(parse_db_time("not a time"), None);
    }

    #[test]
    fn test_get_sessions_reads_go_timestamps() {
        let store = Store::in_memory().unwrap();
        store
            .conn
            .execute(
                "INSERT INTO sessions (type, started_at, ended_at, completed, duration_secs)
                 VALUES ('work', '2026-07-14 10:21:57.2 +0300 EAT m=+1.5', '2026-07-14 10:46:57.2 +0300 EAT m=+1501.5', 1, 1500),
                        ('work', '2026-07-15T09:00:00+00:00', NULL, 1, 1500)",
                [],
            )
            .unwrap();

        let day = |d: &str| DateTime::parse_from_rfc3339(d).unwrap().with_timezone(&Utc);
        let all = store.get_sessions(day("2026-07-01T00:00:00Z"), day("2026-08-01T00:00:00Z")).unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0].started_at, day("2026-07-14T07:21:57.2Z"));
        assert_eq!(all[0].ended_at, Some(day("2026-07-14T07:46:57.2Z")));

        let only_15th = store.get_sessions(day("2026-07-15T00:00:00Z"), day("2026-07-16T00:00:00Z")).unwrap();
        assert_eq!(only_15th.len(), 1);
    }

    #[test]
    fn test_store_in_memory_migrations() {
        let store = Store::in_memory().expect("in memory store should initialize");
        let version = store.get_current_schema_version().unwrap();
        assert_eq!(version, 7);
    }

    #[test]
    fn test_record_and_get_sessions() {
        let mut store = Store::in_memory().unwrap();
        let now = Utc::now();
        let mut session = DbSession {
            id: 0,
            session_type: "work".to_string(),
            task: Some("Write Rust code".to_string()),
            note: None,
            started_at: now,
            ended_at: Some(now + Duration::seconds(1500)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: None,
            mode: None,
            block_id: None,
        };

        store.save_session(&mut session).expect("save session");
        assert!(session.id > 0);

        let sessions = store
            .get_sessions(now - Duration::hours(1), now + Duration::hours(1))
            .expect("get sessions");
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].task.as_deref(), Some("Write Rust code"));
        assert_eq!(sessions[0].duration_secs, 1500);
        assert_eq!(sessions[0].session_type, "work");
    }

    #[test]
    fn test_projects_crud() {
        let mut store = Store::in_memory().unwrap();
        let mut p = Project {
            id: 0,
            name: "Pomogo-Rust".to_string(),
            color: "#ff5555".to_string(),
            archived: false,
            icon: "🦀".to_string(),
        };
        store.create_project(&mut p).unwrap();
        assert!(p.id > 0);

        let projects = store.get_projects().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Pomogo-Rust");
        assert_eq!(projects[0].color, "#ff5555");

        let p_by_name = store.get_project_by_name("Pomogo-Rust").unwrap();
        assert!(p_by_name.is_some());
        assert_eq!(p_by_name.unwrap().id, p.id);

        store.archive_project("Pomogo-Rust").unwrap();
        let p_archived = store.get_project_by_name("Pomogo-Rust").unwrap().unwrap();
        assert!(p_archived.archived);
    }

    #[test]
    fn test_unique_tasks() {
        let mut store = Store::in_memory().unwrap();
        let now = Utc::now();
        let mut s1 = DbSession {
            id: 0,
            session_type: "work".to_string(),
            task: Some("Task Alpha".to_string()),
            note: None,
            started_at: now - Duration::minutes(30),
            ended_at: Some(now - Duration::minutes(5)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: None,
            mode: None,
            block_id: None,
        };
        store.save_session(&mut s1).unwrap();

        let mut s2 = DbSession {
            id: 0,
            session_type: "work".to_string(),
            task: Some("Task Beta".to_string()),
            note: None,
            started_at: now,
            ended_at: Some(now + Duration::minutes(25)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: None,
            mode: None,
            block_id: None,
        };
        store.save_session(&mut s2).unwrap();

        let tasks = store.get_unique_tasks(None).unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains(&"Task Alpha".to_string()));
        assert!(tasks.contains(&"Task Beta".to_string()));

        store.delete_task_name("Task Alpha", None).unwrap();
        let tasks_after = store.get_unique_tasks(None).unwrap();
        assert_eq!(tasks_after.len(), 1);
        assert_eq!(tasks_after[0], "Task Beta");
    }

    #[test]
    fn test_blocks_lifecycle() {
        let mut store = Store::in_memory().unwrap();
        let now = Utc::now();
        let mut block = BlockStore {
            id: 0,
            mode: "deep".to_string(),
            planned_secs: 5400,
            started_at: now,
            ended_at: None,
            completed: false,
            pauses: 0,
        };

        store.create_block(&mut block).unwrap();
        assert!(block.id > 0);

        store.increment_block_pauses(block.id).unwrap();
        store.finish_block(block.id, true, now + Duration::seconds(5400)).unwrap();

        let last = store.get_last_block().unwrap().expect("last block exists");
        assert_eq!(last.id, block.id);
        assert!(last.completed);
        assert_eq!(last.pauses, 1);
    }
}


