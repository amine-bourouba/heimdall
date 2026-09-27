//! Projects (Models/Project.swift) and the IPC [`ProjectSummary`] (docs/ipc-contract.md §5).

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

use super::session::Session;

/// A project folder under `~/.claude/projects/` with its sessions.
#[derive(Debug, Clone, Default)]
pub struct Project {
    /// The folder name (contract `ProjectId`).
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub path_source: Option<PathSource>,
    pub sessions: Vec<Session>,
}

impl Project {
    pub fn recent_session(&self) -> Option<&Session> {
        self.sessions.iter().max_by_key(|s| s.started_at)
    }

    pub fn summary(&self, has_memory: bool) -> ProjectSummary {
        ProjectSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            path: self.path.clone(),
            path_source: self.path_source.unwrap_or(PathSource::Decoded),
            session_count: self.sessions.len() as u32,
            last_activity: self.recent_session().map(|s| s.ended_at.unwrap_or(s.started_at)),
            has_memory,
        }
    }
}

/// Which source produced a project's path (ADR 0003 resolution order).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum PathSource {
    TranscriptCwd,
    SessionsIndex,
    Decoded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSummary {
    pub id: String,
    pub name: String,
    pub path: Option<String>,
    pub path_source: PathSource,
    pub session_count: u32,
    pub last_activity: Option<DateTime<Utc>>,
    pub has_memory: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session(id: &str, secs: i64) -> Session {
        Session::new(id, "/path", vec![], DateTime::from_timestamp(secs, 0).unwrap(), None, 0)
    }

    // Ported from PoirotTests/Models/ProjectTests.swift.
    #[test]
    fn recent_session_returns_most_recent() {
        let p = Project {
            id: "p1".into(),
            name: "test".into(),
            sessions: vec![session("1", 1000), session("2", 2000)],
            ..Default::default()
        };
        assert_eq!(p.recent_session().map(|s| s.id.as_str()), Some("2"));
    }
    #[test]
    fn recent_session_with_no_sessions_returns_nil() {
        assert!(Project::default().recent_session().is_none());
    }
    #[test]
    fn recent_session_with_single_session_returns_that_session() {
        let p = Project {
            sessions: vec![session("only", 5)],
            ..Default::default()
        };
        assert_eq!(p.recent_session().map(|s| s.id.as_str()), Some("only"));
    }

    #[test]
    fn summary_reports_path_source_and_last_activity() {
        let p = Project {
            id: "-Users-me-app".into(),
            name: "app".into(),
            path: Some("/Users/me/app".into()),
            path_source: Some(PathSource::TranscriptCwd),
            sessions: vec![session("a", 10), session("b", 20)],
        };
        let json = serde_json::to_value(p.summary(true)).unwrap();
        assert_eq!(json["pathSource"], "transcriptCwd");
        assert_eq!(json["sessionCount"], 2);
        assert_eq!(json["lastActivity"], "1970-01-01T00:00:20Z");
    }
}
