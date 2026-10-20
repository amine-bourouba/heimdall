//! Groups sub-agent sessions under their parent (Models/SessionGroup.swift).

use std::collections::HashMap;

use serde::Serialize;
use specta::Type;

use super::session::SessionSummary;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionGroup {
    pub id: String,
    pub parent: SessionSummary,
    pub agents: Vec<SessionSummary>,
}

impl SessionGroup {
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    pub fn all_sessions(&self) -> Vec<&SessionSummary> {
        std::iter::once(&self.parent).chain(&self.agents).collect()
    }

    /// Sub-agents attach to the parent named by `parent_session_id`, sorted oldest first. A sidechain
    /// session without a parent id becomes its own group. Groups are sorted newest parent first.
    /// As in the Swift reference app, a sub-agent whose parent is not in the list is not shown.
    pub fn group(sessions: Vec<SessionSummary>) -> Vec<SessionGroup> {
        let mut parents = Vec::new();
        let mut children: HashMap<String, Vec<SessionSummary>> = HashMap::new();
        let mut orphans = Vec::new();
        for s in sessions {
            match s.parent_session_id.clone() {
                Some(parent) => children.entry(parent).or_default().push(s),
                None if !s.is_sidechain => parents.push(s),
                None => orphans.push(s),
            }
        }
        let mut groups: Vec<SessionGroup> = parents
            .into_iter()
            .map(|parent| {
                let mut agents = children.remove(&parent.id).unwrap_or_default();
                agents.sort_by_key(|a| a.started_at);
                SessionGroup {
                    id: parent.id.clone(),
                    parent,
                    agents,
                }
            })
            .chain(orphans.into_iter().map(|o| SessionGroup {
                id: o.id.clone(),
                parent: o,
                agents: Vec::new(),
            }))
            .collect();
        groups.sort_by_key(|g| std::cmp::Reverse(g.parent.started_at));
        groups
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session::Session;
    use chrono::DateTime;

    fn s(id: &str, secs: i64, parent: Option<&str>, sidechain: bool) -> SessionSummary {
        let mut session = Session::new(id, "/test", vec![], DateTime::from_timestamp(secs, 0).unwrap(), None, 0);
        session.parent_session_id = parent.map(String::from);
        session.is_sidechain = sidechain;
        session.summary("p")
    }
    fn ids(v: &[&SessionSummary]) -> Vec<String> {
        v.iter().map(|s| s.id.clone()).collect()
    }

    // Ported from PoirotTests/Models/SessionGroupTests.swift.
    #[test]
    fn group_parent_with_agents_groups_together() {
        let g = SessionGroup::group(vec![
            s("p1", 100, None, false),
            s("a1", 110, Some("p1"), false),
            s("a2", 120, Some("p1"), false),
        ]);
        assert_eq!(g.len(), 1);
        assert_eq!((g[0].parent.id.as_str(), g[0].agent_count()), ("p1", 2));
    }
    #[test]
    fn group_agents_sorted_by_start_time() {
        let g = SessionGroup::group(vec![
            s("p1", 100, None, false),
            s("a2", 300, Some("p1"), false),
            s("a1", 200, Some("p1"), false),
        ]);
        assert_eq!(
            g[0].agents.iter().map(|a| a.id.as_str()).collect::<Vec<_>>(),
            ["a1", "a2"]
        );
    }
    #[test]
    fn group_multiple_parents_creates_separate_groups() {
        let g = SessionGroup::group(vec![
            s("p1", 200, None, false),
            s("p2", 100, None, false),
            s("a1", 210, Some("p1"), false),
            s("a2", 110, Some("p2"), false),
        ]);
        assert_eq!(g.len(), 2);
        assert_eq!((g[0].parent.id.as_str(), g[0].agents.len()), ("p1", 1));
        assert_eq!((g[1].parent.id.as_str(), g[1].agents.len()), ("p2", 1));
    }
    #[test]
    fn group_parent_with_no_agents_creates_group_with_empty_agents() {
        let g = SessionGroup::group(vec![s("p1", 0, None, false)]);
        assert_eq!((g.len(), g[0].agent_count()), (1, 0));
    }
    #[test]
    fn group_orphan_sidechain_becomes_standalone_group() {
        let g = SessionGroup::group(vec![s("o1", 0, None, true)]);
        assert_eq!((g.len(), g[0].parent.id.as_str(), g[0].agents.len()), (1, "o1", 0));
    }
    #[test]
    fn group_empty_sessions_returns_empty() {
        assert!(SessionGroup::group(vec![]).is_empty());
    }
    #[test]
    fn group_sorted_descending_by_parent_start_time() {
        let g = SessionGroup::group(vec![
            s("p1", 100, None, false),
            s("p2", 300, None, false),
            s("p3", 200, None, false),
        ]);
        assert_eq!(
            g.iter().map(|x| x.parent.id.as_str()).collect::<Vec<_>>(),
            ["p2", "p3", "p1"]
        );
    }
    #[test]
    fn all_sessions_includes_parent_and_agents() {
        let g = SessionGroup::group(vec![s("p1", 100, None, false), s("a1", 110, Some("p1"), false)]);
        assert_eq!(ids(&g[0].all_sessions()), ["p1", "a1"]);
    }
}
