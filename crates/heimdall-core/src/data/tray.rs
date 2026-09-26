//! Tray snapshot aggregation (Views/MenuBar/MenuBarState.swift): header stats plus up to 10 most
//! recent sessions across every project, optionally fuzzy-filtered. A pure function so the tray
//! window and the main window build the same snapshot from the same data.

use std::fmt::Display;

use chrono::{Local, TimeZone, Utc};
use serde::Serialize;
use specta::Type;

use crate::models::project::Project;
use crate::models::session::SessionSummary;
use crate::models::stats_cache::StatsCache;
use crate::search::matcher;

const MAX_RECENT_SESSIONS: usize = 10;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RecentSession {
    pub project_id: String,
    pub project_name: String,
    pub session: SessionSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TraySnapshot {
    pub total_sessions: u64,
    pub total_messages: u64,
    /// Message count of today's entry in `stats.dailyActivity`; 0 without one.
    pub today_messages: u64,
    pub recent: Vec<RecentSession>,
}

/// The 10 most recently started sessions across every project (MenuBarState.loadRecentSessions).
pub fn recent_sessions(projects: &[Project]) -> Vec<(&Project, &crate::models::session::Session)> {
    let mut all: Vec<_> = projects.iter().flat_map(|p| p.sessions.iter().map(move |s| (p, s))).collect();
    all.sort_by_key(|(_, s)| std::cmp::Reverse(s.started_at));
    all.truncate(MAX_RECENT_SESSIONS);
    all
}

/// `recent`, kept to pairs whose session title or project name fuzzy-matches `query`
/// (MenuBarState.filteredSessions). A blank query, after trimming, returns `recent` unchanged —
/// the filter narrows the already-capped top 10, it never looks further back for a better match.
pub fn filter_recent<'a>(recent: &[(&'a Project, &'a crate::models::session::Session)], query: Option<&str>) -> Vec<(&'a Project, &'a crate::models::session::Session)> {
    let trimmed = query.map(str::trim).filter(|q| !q.is_empty());
    let Some(q) = trimmed else { return recent.to_vec() };
    recent.iter().copied().filter(|(p, s)| matcher::fuzzy(&s.title(), q).is_some() || matcher::fuzzy(&p.name, q).is_some()).collect()
}

/// The snapshot in local time.
pub fn snapshot(projects: &[Project], stats: Option<&StatsCache>, query: Option<&str>) -> TraySnapshot {
    snapshot_in(projects, stats, query, &Local)
}

pub fn snapshot_in<Tz: TimeZone>(projects: &[Project], stats: Option<&StatsCache>, query: Option<&str>, tz: &Tz) -> TraySnapshot
where
    Tz::Offset: Display,
{
    let recent = filter_recent(&recent_sessions(projects), query)
        .into_iter()
        .map(|(p, s)| RecentSession { project_id: p.id.clone(), project_name: p.name.clone(), session: s.summary(p.id.clone()) })
        .collect();
    let today = Utc::now().with_timezone(tz).format("%Y-%m-%d").to_string();
    let today_messages = stats.and_then(|s| s.daily_activity.iter().find(|d| d.date == today)).map_or(0, |d| d.message_count);
    TraySnapshot {
        total_sessions: stats.map_or(0, |s| s.total_sessions),
        total_messages: stats.map_or(0, |s| s.total_messages),
        today_messages,
        recent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session::Session;
    use chrono::{DateTime, FixedOffset};

    fn session(id: &str, secs: i64) -> Session {
        Session::new(id, "/test/path", vec![], DateTime::from_timestamp(secs, 0).unwrap(), None, 0)
    }
    fn project(id: &str, name: &str, sessions: Vec<Session>) -> Project {
        Project { id: id.into(), name: name.into(), path: Some("/test/path".into()), path_source: None, sessions }
    }
    fn ids<'a>(pairs: &'a [(&'a Project, &'a Session)]) -> Vec<&'a str> {
        pairs.iter().map(|(_, s)| s.id.as_str()).collect()
    }

    // Ported from PoirotTests/Views/MenuBarStateTests.swift.
    #[test]
    fn initial_state_has_correct_defaults() {
        let s = snapshot(&[], None, None);
        assert!(s.recent.is_empty() && s.total_sessions == 0);
    }
    #[test]
    fn load_recent_sessions_sorts_by_most_recent() {
        let projects = [project("p1", "alpha", vec![session("s1", 1000)]), project("p2", "beta", vec![session("s2", 9000)])];
        assert_eq!(ids(&recent_sessions(&projects)), ["s2", "s1"]);
    }
    #[test]
    fn load_recent_sessions_limits_to_ten() {
        let sessions: Vec<_> = (0..15).map(|i| session(&format!("s{i}"), i * 1000)).collect();
        let projects = [project("p1", "big-project", sessions)];
        assert_eq!(recent_sessions(&projects).len(), 10);
    }
    #[test]
    fn load_recent_sessions_includes_project_info() {
        let projects = [project("p1", "my-app", vec![session("s1", 0)])];
        let r = recent_sessions(&projects);
        assert_eq!((r[0].0.name.as_str(), r[0].1.id.as_str()), ("my-app", "s1"));
    }
    #[test]
    fn load_recent_sessions_empty_projects() {
        assert!(recent_sessions(&[]).is_empty());
    }
    #[test]
    fn filtered_sessions_empty_query_returns_all() {
        let projects = [project("p1", "alpha", vec![session("s1", 0)])];
        assert_eq!(filter_recent(&recent_sessions(&projects), Some("")).len(), 1);
    }
    #[test]
    fn filtered_sessions_whitespace_query_returns_all() {
        let projects = [project("p1", "alpha", vec![session("s1", 0)])];
        assert_eq!(filter_recent(&recent_sessions(&projects), Some("   ")).len(), 1);
    }

    #[test]
    fn filter_matches_project_name_or_session_title() {
        let mut alpha_session = session("s1", 2000);
        alpha_session.cached_title = Some("Fix the login flow".into());
        let projects = [project("p1", "alpha", vec![alpha_session]), project("p2", "beta", vec![session("s2", 1000)])];
        let recent = recent_sessions(&projects);
        assert_eq!(ids(&filter_recent(&recent, Some("login"))), ["s1"]);
        assert_eq!(ids(&filter_recent(&recent, Some("beta"))), ["s2"]);
        assert!(filter_recent(&recent, Some("zzz-nope")).is_empty());
    }

    #[test]
    fn filter_only_narrows_the_capped_top_ten() {
        // An 11th, older session matching the query is not brought back in.
        let mut old = session("old", 0);
        old.cached_title = Some("unique-match".into());
        let mut sessions: Vec<_> = (0..10).map(|i| session(&format!("s{i}"), (i + 1) * 1000)).collect();
        sessions.push(old);
        let projects = [project("p1", "proj", sessions)];
        assert!(filter_recent(&recent_sessions(&projects), Some("unique-match")).is_empty());
    }

    #[test]
    fn snapshot_reports_stats_and_todays_messages() {
        let tz = FixedOffset::east_opt(0).unwrap();
        let today = Utc::now().with_timezone(&tz).format("%Y-%m-%d").to_string();
        let stats = StatsCache {
            total_sessions: 42,
            total_messages: 1234,
            daily_activity: vec![
                crate::models::stats_cache::DailyActivity { date: "2020-01-01".into(), message_count: 999, session_count: 1, tool_call_count: 0 },
                crate::models::stats_cache::DailyActivity { date: today, message_count: 7, session_count: 1, tool_call_count: 0 },
            ],
            ..Default::default()
        };
        let s = snapshot_in(&[], Some(&stats), None, &tz);
        assert_eq!((s.total_sessions, s.total_messages, s.today_messages), (42, 1234, 7));
    }

    #[test]
    fn snapshot_without_a_matching_day_reports_zero() {
        let stats = StatsCache { daily_activity: vec![crate::models::stats_cache::DailyActivity { date: "2020-01-01".into(), message_count: 999, session_count: 1, tool_call_count: 0 }], ..Default::default() };
        assert_eq!(snapshot_in(&[], Some(&stats), None, &FixedOffset::east_opt(0).unwrap()).today_messages, 0);
    }

    #[test]
    fn snapshot_with_no_stats_reports_zeros() {
        let s = snapshot(&[], None, None);
        assert_eq!((s.total_sessions, s.total_messages, s.today_messages), (0, 0, 0));
    }
}
