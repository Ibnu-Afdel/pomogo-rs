// Aggregation and calculation of focus metrics and streaks.

use std::collections::BTreeMap;
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate};
use crate::store::models::DbSession;

#[derive(Debug, Clone, Default)]
pub struct DayStats {
    pub date: NaiveDate,
    pub count: usize,
    pub minutes: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Stats {
    pub today_count: usize,
    pub today_minutes: usize,
    pub week_count: usize,
    pub week_minutes: usize,
    pub month_count: usize,
    pub current_streak: usize,
    pub best_streak: usize,
    pub completion_rate: f64,
    pub week_days: [DayStats; 7],
    pub lifetime_minutes: usize,
    pub lifetime_sessions: usize,
}

pub fn calculate(
    sessions: &[DbSession],
    now: DateTime<Local>,
    project_filter: Option<&str>,
) -> Stats {
    let mut stats = Stats::default();
    let today = now.date_naive();

    // Initialize 7 days ending today
    for i in 0..7 {
        let offset = 6 - i;
        let d = today - Duration::days(offset as i64);
        stats.week_days[i] = DayStats {
            date: d,
            count: 0,
            minutes: 0,
        };
    }

    if sessions.is_empty() {
        return stats;
    }

    let yesterday = today - Duration::days(1);
    let this_year = now.year();
    let this_month = now.month();

    let mut completed_per_day: BTreeMap<NaiveDate, usize> = BTreeMap::new();
    let mut completed_mins_per_day: BTreeMap<NaiveDate, usize> = BTreeMap::new();

    let mut total_work_sessions = 0;
    let mut completed_work_sessions = 0;

    for s in sessions {
        if s.session_type != "work" {
            continue;
        }

        if let Some(f) = project_filter {
            if s.project_name.as_deref().unwrap_or_default() != f {
                continue;
            }
        }

        total_work_sessions += 1;
        if !s.completed {
            continue;
        }
        completed_work_sessions += 1;

        let local_dt = s.started_at.with_timezone(&Local);
        let s_date = local_dt.date_naive();
        let mins = (s.duration_secs / 60) as usize;

        *completed_per_day.entry(s_date).or_default() += 1;
        *completed_mins_per_day.entry(s_date).or_default() += mins;

        stats.lifetime_sessions += 1;
        stats.lifetime_minutes += mins;

        if s_date == today {
            stats.today_count += 1;
            stats.today_minutes += mins;
        }

        if local_dt.year() == this_year && local_dt.month() == this_month {
            stats.month_count += 1;
        }
    }

    if total_work_sessions > 0 {
        stats.completion_rate = (completed_work_sessions as f64) / (total_work_sessions as f64);
    }

    // Last 7 days breakdown
    for i in 0..7 {
        let d = stats.week_days[i].date;
        let c = completed_per_day.get(&d).copied().unwrap_or(0);
        let m = completed_mins_per_day.get(&d).copied().unwrap_or(0);
        stats.week_days[i].count = c;
        stats.week_days[i].minutes = m;
        stats.week_count += c;
        stats.week_minutes += m;
    }

    // Streaks calculation
    let has_today = completed_per_day.get(&today).copied().unwrap_or(0) > 0;
    let has_yesterday = completed_per_day.get(&yesterday).copied().unwrap_or(0) > 0;

    let streak_start = if has_today {
        Some(today)
    } else if has_yesterday {
        Some(yesterday)
    } else {
        None
    };

    if let Some(start_d) = streak_start {
        let mut cur = 0;
        let mut check_d = start_d;
        while completed_per_day.get(&check_d).copied().unwrap_or(0) > 0 {
            cur += 1;
            check_d -= Duration::days(1);
        }
        stats.current_streak = cur;
    }

    // Best streak
    if !completed_per_day.is_empty() {
        let mut best = 0;
        let mut run = 0;
        let mut prev_date: Option<NaiveDate> = None;

        for &date in completed_per_day.keys() {
            match prev_date {
                Some(prev) if date == prev + Duration::days(1) => run += 1,
                _ => run = 1,
            }
            if run > best {
                best = run;
            }
            prev_date = Some(date);
        }

        stats.best_streak = best;
    }

    stats
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_empty_stats() {
        let now = Local::now();
        let stats = calculate(&[], now, None);
        assert_eq!(stats.today_count, 0);
        assert_eq!(stats.current_streak, 0);
        assert_eq!(stats.best_streak, 0);
        assert_eq!(stats.week_count, 0);
    }

    #[test]
    fn test_streak_calculation() {
        let fixed_now = Local.with_ymd_and_hms(2026, 7, 15, 14, 0, 0).unwrap();
        let d1 = Utc.with_ymd_and_hms(2026, 7, 13, 10, 0, 0).unwrap();
        let d2 = Utc.with_ymd_and_hms(2026, 7, 14, 11, 0, 0).unwrap();
        let d3 = Utc.with_ymd_and_hms(2026, 7, 15, 12, 0, 0).unwrap();

        let s1 = DbSession {
            id: 1,
            session_type: "work".to_string(),
            task: Some("Task 1".to_string()),
            note: None,
            started_at: d1,
            ended_at: Some(d1 + Duration::minutes(25)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: Some("Rust".to_string()),
            mode: None,
            block_id: None,
        };
        let s2 = DbSession {
            id: 2,
            session_type: "work".to_string(),
            task: Some("Task 2".to_string()),
            note: None,
            started_at: d2,
            ended_at: Some(d2 + Duration::minutes(25)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: Some("Rust".to_string()),
            mode: None,
            block_id: None,
        };
        let s3 = DbSession {
            id: 3,
            session_type: "work".to_string(),
            task: Some("Task 3".to_string()),
            note: None,
            started_at: d3,
            ended_at: Some(d3 + Duration::minutes(25)),
            completed: true,
            duration_secs: 1500,
            project_id: None,
            project_name: Some("Go".to_string()),
            mode: None,
            block_id: None,
        };

        let stats = calculate(&[s1.clone(), s2.clone(), s3.clone()], fixed_now, None);
        assert_eq!(stats.current_streak, 3);
        assert_eq!(stats.best_streak, 3);
        assert_eq!(stats.today_count, 1);
        assert_eq!(stats.today_minutes, 25);
        assert_eq!(stats.lifetime_sessions, 3);
        assert_eq!(stats.lifetime_minutes, 75);

        // Project filter test
        let rust_stats = calculate(&[s1, s2, s3], fixed_now, Some("Rust"));
        assert_eq!(rust_stats.lifetime_sessions, 2);
        assert_eq!(rust_stats.today_count, 0);
    }
}


