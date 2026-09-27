//! One prompt from `~/.claude/history.jsonl` (Models/HistoryEntry.swift). Date buckets and relative
//! times are view logic and live in the history store (port-plan P-1).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

use crate::paths::last_component;

#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    /// Stable within a session: derived from the raw line (data/history.rs).
    pub id: String,
    pub display: String,
    pub pasted_contents: BTreeMap<String, String>,
    pub timestamp: DateTime<Utc>,
    pub project: String,
}

impl PartialEq for HistoryEntry {
    /// Entries are equal when their ids are (HistoryEntry.swift `==`).
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl HistoryEntry {
    pub fn project_name(&self) -> &str {
        last_component(&self.project)
    }

    /// The first three non-empty lines joined by spaces, for previews.
    pub fn snippet(&self) -> String {
        self.display
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .take(3)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, display: &str, project: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.into(),
            display: display.into(),
            pasted_contents: BTreeMap::new(),
            timestamp: DateTime::UNIX_EPOCH,
            project: project.into(),
        }
    }

    // Ported from PoirotTests/Models/HistoryEntryTests.swift (bucket and timeAgo cases moved to Vitest).
    #[test]
    fn project_name_extracts_last_path_component() {
        assert_eq!(
            entry("1-0", "test prompt", "/Users/test/Dev/my-project").project_name(),
            "my-project"
        );
    }
    #[test]
    fn snippet_truncates_multiple_lines() {
        assert_eq!(
            entry("1-0", "Line one\nLine two\nLine three\nLine four\nLine five", "/test").snippet(),
            "Line one Line two Line three"
        );
    }
    #[test]
    fn snippet_skips_empty_lines() {
        assert_eq!(entry("1-0", "First\n\n\nSecond", "/test").snippet(), "First Second");
    }
    #[test]
    fn equality_based_on_id() {
        assert_eq!(entry("same-id", "a", "/a"), entry("same-id", "b", "/b"));
    }
    #[test]
    fn inequality_different_ids() {
        assert_ne!(entry("id-1", "a", "/a"), entry("id-2", "a", "/a"));
    }
}
