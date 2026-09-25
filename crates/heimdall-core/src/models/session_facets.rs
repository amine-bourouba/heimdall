//! AI summary facets from `~/.claude/usage-data/facets/<sessionId>.json` (Models/SessionFacets.swift).
//! Keys are snake_case on disk and camelCase over IPC.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all(serialize = "camelCase", deserialize = "snake_case"), default)]
pub struct SessionFacets {
    pub session_id: String,
    pub underlying_goal: String,
    pub goal_categories: BTreeMap<String, u64>,
    pub outcome: String,
    pub user_satisfaction_counts: BTreeMap<String, u64>,
    pub claude_helpfulness: String,
    pub session_type: String,
    pub friction_counts: BTreeMap<String, u64>,
    pub friction_detail: String,
    pub primary_success: String,
    pub brief_summary: String,
}

/// `feature_implementation` → `Feature Implementation` (Swift `.capitalized` after replacing `_`).
pub fn humanize(raw: &str) -> String {
    raw.replace('_', " ")
        .split(' ')
        .map(|w| {
            let mut c = w.chars();
            c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c.flat_map(char::to_lowercase)).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

impl SessionFacets {
    pub fn id(&self) -> &str {
        &self.session_id
    }

    /// Goal categories by count, highest first (ties by name, for a stable order).
    pub fn sorted_goal_categories(&self) -> Vec<(&str, u64)> {
        sort_counts(self.goal_categories.iter().map(|(k, &v)| (k.as_str(), v)))
    }

    pub fn total_friction_count(&self) -> u64 {
        self.friction_counts.values().sum()
    }

    /// Friction items with a non-zero count, highest first.
    pub fn sorted_friction_items(&self) -> Vec<(&str, u64)> {
        sort_counts(self.friction_counts.iter().filter(|(_, v)| **v > 0).map(|(k, &v)| (k.as_str(), v)))
    }

    pub fn outcome_label(&self) -> String {
        match self.outcome.as_str() {
            "success" => "Success".into(),
            "partially_achieved" => "Partial".into(),
            "unclear_from_transcript" => "Unclear".into(),
            "failure" => "Failed".into(),
            other => humanize(other),
        }
    }

    pub fn helpfulness_label(&self) -> String {
        match self.claude_helpfulness.as_str() {
            "very_helpful" => "Very Helpful".into(),
            "slightly_helpful" => "Slightly Helpful".into(),
            "not_helpful" => "Not Helpful".into(),
            "harmful" => "Harmful".into(),
            other => humanize(other),
        }
    }

    pub fn session_type_label(&self) -> String {
        humanize(&self.session_type)
    }
}

fn sort_counts<'a>(it: impl Iterator<Item = (&'a str, u64)>) -> Vec<(&'a str, u64)> {
    let mut v: Vec<_> = it.collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make(outcome: &str, helpfulness: &str, session_type: &str, goals: &[(&str, u64)], friction: &[(&str, u64)]) -> SessionFacets {
        SessionFacets {
            session_id: "test".into(), outcome: outcome.into(), claude_helpfulness: helpfulness.into(), session_type: session_type.into(),
            goal_categories: goals.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            friction_counts: friction.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
            ..Default::default()
        }
    }
    fn with_outcome(o: &str) -> SessionFacets {
        make(o, "very_helpful", "single_task", &[], &[])
    }
    fn with_helpfulness(h: &str) -> SessionFacets {
        make("success", h, "single_task", &[], &[])
    }

    // Ported from PoirotTests/Models/SessionFacetsTests.swift.
    #[test]
    fn decodes_full_json() {
        let f: SessionFacets = serde_json::from_str(r#"{
          "underlying_goal": "Refactor auth to JWT", "goal_categories": {"refactoring": 2, "security": 1},
          "outcome": "success", "user_satisfaction_counts": {"likely_satisfied": 1}, "claude_helpfulness": "very_helpful",
          "session_type": "feature_implementation", "friction_counts": {"tool_failures": 1, "misunderstandings": 0},
          "friction_detail": "One tool call timed out", "primary_success": "multi_file_changes",
          "brief_summary": "Migrated session auth from cookies to JWT.", "session_id": "abc-123"}"#).unwrap();
        assert_eq!(f.session_id, "abc-123");
        assert_eq!(f.underlying_goal, "Refactor auth to JWT");
        assert_eq!(f.goal_categories, BTreeMap::from([("refactoring".into(), 2), ("security".into(), 1)]));
        assert_eq!((f.outcome.as_str(), f.claude_helpfulness.as_str(), f.session_type.as_str()), ("success", "very_helpful", "feature_implementation"));
        assert_eq!(f.friction_counts, BTreeMap::from([("tool_failures".into(), 1), ("misunderstandings".into(), 0)]));
        assert_eq!((f.friction_detail.as_str(), f.primary_success.as_str()), ("One tool call timed out", "multi_file_changes"));
        assert_eq!(f.brief_summary, "Migrated session auth from cookies to JWT.");
        assert_eq!(f.user_satisfaction_counts, BTreeMap::from([("likely_satisfied".into(), 1)]));
    }
    #[test]
    fn decodes_empty_dictionaries() {
        let f: SessionFacets = serde_json::from_str(r#"{"underlying_goal": "Quick question", "goal_categories": {}, "outcome": "success",
          "user_satisfaction_counts": {}, "claude_helpfulness": "very_helpful", "session_type": "single_task", "friction_counts": {},
          "friction_detail": "", "primary_success": "none", "brief_summary": "Answered a question.", "session_id": "def-456"}"#).unwrap();
        assert!(f.goal_categories.is_empty() && f.friction_counts.is_empty());
        assert_eq!(f.total_friction_count(), 0);
    }
    #[test]
    fn id_matches_session_id() {
        let f: SessionFacets = serde_json::from_str(r#"{"session_id": "test-id-789", "outcome": "success"}"#).unwrap();
        assert_eq!(f.id(), "test-id-789");
    }
    #[test]
    fn sorted_goal_categories_ordered_by_count_descending() {
        let f = make("success", "very_helpful", "single_task", &[("security", 1), ("refactoring", 3), ("testing", 2)], &[]);
        assert_eq!(f.sorted_goal_categories(), [("refactoring", 3), ("testing", 2), ("security", 1)]);
    }
    #[test]
    fn total_friction_count_sums_values() {
        assert_eq!(make("success", "very_helpful", "x", &[], &[("tool_failures", 2), ("misunderstandings", 1)]).total_friction_count(), 3);
    }
    #[test]
    fn sorted_friction_items_filters_zero_values() {
        let f = make("success", "very_helpful", "x", &[], &[("tool_failures", 2), ("misunderstandings", 0), ("buggy_code", 1)]);
        assert_eq!(f.sorted_friction_items(), [("tool_failures", 2), ("buggy_code", 1)]);
    }
    #[test]
    fn outcome_label_maps_known_values() {
        for (raw, label) in [("success", "Success"), ("partially_achieved", "Partial"), ("unclear_from_transcript", "Unclear"), ("failure", "Failed"), ("some_other", "Some Other")] {
            assert_eq!(with_outcome(raw).outcome_label(), label);
        }
    }
    #[test]
    fn helpfulness_label_maps_known_values() {
        for (raw, label) in [("very_helpful", "Very Helpful"), ("slightly_helpful", "Slightly Helpful"), ("not_helpful", "Not Helpful"), ("harmful", "Harmful")] {
            assert_eq!(with_helpfulness(raw).helpfulness_label(), label);
        }
    }
    #[test]
    fn session_type_label_formats_underscores() {
        assert_eq!(make("success", "very_helpful", "feature_implementation", &[], &[]).session_type_label(), "Feature Implementation");
    }
    #[test]
    fn category_label_formats_underscores() {
        assert_eq!((humanize("fix_bugs").as_str(), humanize("write_tests").as_str()), ("Fix Bugs", "Write Tests"));
    }
    #[test]
    fn friction_label_formats_underscores() {
        assert_eq!((humanize("tool_failures").as_str(), humanize("buggy_code").as_str()), ("Tool Failures", "Buggy Code"));
    }

    #[test]
    fn serialises_camel_case_for_ipc() {
        let json = serde_json::to_value(with_outcome("success")).unwrap();
        assert!(json.get("sessionId").is_some() && json.get("briefSummary").is_some() && json.get("session_id").is_none());
    }
}
