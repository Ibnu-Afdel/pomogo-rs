use std::collections::BTreeMap;
use chrono::{DateTime, Utc};
use crate::store::models::DbSession;

pub fn export_sessions_json(sessions: &[DbSession]) -> Result<String, String> {
    serde_json::to_string_pretty(sessions).map_err(|e| e.to_string())
}

pub fn export_sessions_csv(sessions: &[DbSession]) -> Result<String, String> {
    let mut wtr = csv::Writer::from_writer(vec![]);
    wtr.write_record([
        "id",
        "type",
        "task",
        "note",
        "started_at",
        "ended_at",
        "completed",
        "duration_secs",
        "project_name",
    ])
    .map_err(|e| e.to_string())?;

    for s in sessions {
        wtr.write_record(&[
            s.id.to_string(),
            s.session_type.clone(),
            s.task.clone().unwrap_or_default(),
            s.note.clone().unwrap_or_default(),
            s.started_at.to_rfc3339(),
            s.ended_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
            if s.completed { "true" } else { "false" }.to_string(),
            s.duration_secs.to_string(),
            s.project_name.clone().unwrap_or_default(),
        ])
        .map_err(|e| e.to_string())?;
    }

    let bytes = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

pub fn generate_markdown_report(
    sessions: &[DbSession],
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> String {
    let mut total_work_secs: i64 = 0;
    let mut completed_count = 0;
    let mut skipped_count = 0;
    let mut project_durations: BTreeMap<String, i64> = BTreeMap::new();

    for sess in sessions {
        if sess.session_type == "work" {
            if sess.completed {
                completed_count += 1;
                total_work_secs += sess.duration_secs;
                let proj = sess
                    .project_name
                    .clone()
                    .unwrap_or_else(|| "(no project)".to_string());
                *project_durations.entry(proj).or_default() += sess.duration_secs;
            } else {
                skipped_count += 1;
            }
        }
    }

    let mut out = String::new();
    out.push_str("# PomoGo Focus Report\n\n");
    out.push_str(&format!(
        "Report Period: {} to {}\n\n",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d")
    ));

    out.push_str("## Summary\n");
    let total_hours = (total_work_secs as f64) / 3600.0;
    out.push_str(&format!("- **Total Focus Time**: {:.2} hours\n", total_hours));
    out.push_str(&format!("- **Completed Sessions**: {}\n", completed_count));
    out.push_str(&format!("- **Skipped Sessions**: {}\n\n", skipped_count));

    out.push_str("## Project Breakdown\n");
    if project_durations.is_empty() {
        out.push_str("No completed sessions in this period.\n");
    } else {
        out.push_str("| Project | Hours Focused |\n");
        out.push_str("|---|---|\n");
        for (proj, secs) in &project_durations {
            let hours = (*secs as f64) / 3600.0;
            out.push_str(&format!("| {} | {:.2} |\n", proj, hours));
        }
        out.push('\n');
    }

    out
}

