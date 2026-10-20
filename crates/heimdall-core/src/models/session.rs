//! Sessions (Models/Session.swift): the parsed domain [`Session`] and the IPC [`SessionSummary`]
//! and [`SessionDetail`] from docs/ipc-contract.md §5.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Serialize;
use specta::Type;

use super::context_usage::ContextUsage;
use super::message::{Message, Role};
use super::token_usage::TokenUsage;
use crate::paths::last_component;

pub const UNTITLED_SESSION: &str = "Untitled session";

static XML_TAG: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<[^>]+>").expect("static regex"));

/// Remove `<tag>` markers but keep the text between them, then trim. If nothing is left, return the
/// original (Session.swift `strippingXMLTags`).
pub fn strip_xml_tags(s: &str) -> String {
    let stripped = XML_TAG.replace_all(s, "");
    let trimmed = stripped.trim();
    if trimmed.is_empty() {
        s.to_string()
    } else {
        trimmed.to_string()
    }
}

/// A parsed session. `cached_*` fields come from the summary tier or the sessions index and win
/// over values derived from `messages`, as in the Swift reference app.
#[derive(Debug, Clone, Default)]
pub struct Session {
    pub id: String,
    pub project_path: String,
    pub messages: Vec<Message>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub total_tokens: u64,
    pub tokens: TokenUsage,
    pub file: Option<PathBuf>,
    pub cached_title: Option<String>,
    pub cached_preview: Option<String>,
    pub cached_turn_count: Option<u32>,
    pub first_prompt: Option<String>,
    pub agent_id: Option<String>,
    pub is_sidechain: bool,
    pub parent_session_id: Option<String>,
    pub agent_type: Option<String>,
    pub agent_description: Option<String>,
    /// Lower-cased, length-capped free text of the conversation, for search.
    pub searchable_text: String,
    /// Each `cwd` in the transcript and how many records carry it (summary tier), for project
    /// path resolution (projects/project_path.rs).
    pub cwds: HashMap<String, u32>,
}

impl PartialEq for Session {
    /// Sessions are equal when their ids are (Session.swift `==`).
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Session {
    pub fn new(
        id: impl Into<String>,
        project_path: impl Into<String>,
        messages: Vec<Message>,
        started_at: DateTime<Utc>,
        model: Option<String>,
        total_tokens: u64,
    ) -> Self {
        Self {
            id: id.into(),
            project_path: project_path.into(),
            messages,
            started_at,
            model,
            total_tokens,
            ..Self::default()
        }
    }

    pub fn project_name(&self) -> &str {
        last_component(&self.project_path)
    }

    pub fn title(&self) -> String {
        let first_user = || {
            self.messages
                .iter()
                .find(|m| m.role == Role::User)
                .map(Message::text_content)
        };
        let raw = self
            .cached_title
            .clone()
            .or_else(first_user)
            .unwrap_or_else(|| UNTITLED_SESSION.to_string());
        strip_xml_tags(&raw)
    }

    pub fn preview(&self) -> Option<String> {
        let first_assistant = || {
            self.messages
                .iter()
                .find(|m| m.role == Role::Assistant)
                .map(Message::text_content)
        };
        self.cached_preview
            .clone()
            .or_else(first_assistant)
            .map(|p| strip_xml_tags(&p))
    }

    pub fn turn_count(&self) -> u32 {
        self.cached_turn_count
            .unwrap_or_else(|| self.messages.iter().filter(|m| m.role == Role::User).count() as u32)
    }

    pub fn is_sub_agent(&self) -> bool {
        self.parent_session_id.is_some()
    }

    pub fn duration_ms(&self) -> Option<i64> {
        self.ended_at.map(|end| (end - self.started_at).num_milliseconds())
    }

    pub fn summary(&self, project_id: impl Into<String>) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            project_id: project_id.into(),
            title: self.title(),
            preview: self.preview(),
            started_at: self.started_at,
            ended_at: self.ended_at,
            model: self.model.clone(),
            tokens: self.tokens,
            turn_count: self.turn_count(),
            is_sidechain: self.is_sidechain,
            parent_session_id: self.parent_session_id.clone(),
            agent_type: self.agent_type.clone(),
            agent_description: self.agent_description.clone(),
            has_debug_log: false,
            has_file_history: false,
            has_todos: false,
        }
    }
}

/// One session in a list (contract `SessionSummary`). The `has_*` flags are filled by the commands layer.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub preview: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub tokens: TokenUsage,
    pub turn_count: u32,
    pub is_sidechain: bool,
    pub parent_session_id: Option<String>,
    pub agent_type: Option<String>,
    pub agent_description: Option<String>,
    pub has_debug_log: bool,
    pub has_file_history: bool,
    pub has_todos: bool,
}

/// The header of an opened session (contract `SessionDetail`); messages are fetched in pages.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub summary: SessionSummary,
    pub message_count: u32,
    pub context: Option<ContextUsage>,
    pub tool_names: Vec<String>,
    pub skipped: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::message::test_support::{message, text};

    fn session(id: &str, path: &str, messages: Vec<Message>) -> Session {
        Session::new(id, path, messages, DateTime::UNIX_EPOCH, None, 0)
    }
    fn user(t: &str) -> Message {
        message("m", Role::User, vec![text(t)])
    }
    fn assistant(t: &str) -> Message {
        message("m", Role::Assistant, vec![text(t)])
    }

    // Ported from PoirotTests/Models/SessionTests.swift.
    #[test]
    fn title_with_user_message_returns_first_user_message_text() {
        assert_eq!(
            session("1", "/Users/dev/my-project", vec![user("Hello world")]).title(),
            "Hello world"
        );
    }
    #[test]
    fn title_with_no_user_message_returns_untitled() {
        assert_eq!(
            session("2", "/path", vec![assistant("Hi there")]).title(),
            UNTITLED_SESSION
        );
    }
    #[test]
    fn title_with_empty_messages_returns_untitled() {
        assert_eq!(session("3", "/path", vec![]).title(), UNTITLED_SESSION);
    }
    #[test]
    fn project_name_returns_last_path_component() {
        assert_eq!(
            session("4", "/Users/dev/my-project", vec![]).project_name(),
            "my-project"
        );
        assert_eq!(
            session("4w", r"C:\Users\dev\my-project", vec![]).project_name(),
            "my-project"
        );
    }
    #[test]
    fn turn_count_counts_only_user_messages() {
        let s = session(
            "5",
            "/path",
            vec![
                user("a"),
                assistant("b"),
                user("c"),
                message("m4", Role::System, vec![text("d")]),
            ],
        );
        assert_eq!(s.turn_count(), 2);
    }
    #[test]
    fn equality_based_on_id() {
        let mut other = session("same", "/b", vec![]);
        other.total_tokens = 100;
        assert_eq!(session("same", "/a", vec![]), other);
    }
    #[test]
    fn title_with_cached_title_prefers_cached() {
        let mut s = session("c1", "/path", vec![user("From messages")]);
        s.cached_title = Some("From cache".into());
        assert_eq!(s.title(), "From cache");
    }
    #[test]
    fn title_with_nil_cached_title_falls_back_to_messages() {
        assert_eq!(
            session("c2", "/path", vec![user("From messages")]).title(),
            "From messages"
        );
    }
    #[test]
    fn turn_count_with_cached_turn_count_prefers_cached() {
        let mut s = session("c3", "/path", vec![user("a"), user("b")]);
        s.cached_turn_count = Some(5);
        assert_eq!(s.turn_count(), 5);
    }
    #[test]
    fn turn_count_with_nil_cached_turn_count_falls_back_to_messages() {
        assert_eq!(
            session("c4", "/path", vec![user("a"), assistant("b"), user("c")]).turn_count(),
            2
        );
    }
    #[test]
    fn file_url_defaults_to_nil() {
        assert!(session("c5", "/path", vec![]).file.is_none());
    }
    #[test]
    fn file_url_preserves_value() {
        let mut s = session("c6", "/path", vec![]);
        s.file = Some(PathBuf::from("/tmp/test.jsonl"));
        assert_eq!(s.file.as_deref(), Some(std::path::Path::new("/tmp/test.jsonl")));
    }
    fn with_first_prompt(value: Option<&str>) -> Option<String> {
        let mut s = session("fp", "/path", vec![]);
        s.first_prompt = value.map(String::from);
        s.first_prompt
    }
    #[test]
    fn first_prompt_defaults_to_nil() {
        assert!(session("fp1", "/path", vec![]).first_prompt.is_none());
    }
    #[test]
    fn first_prompt_preserves_value() {
        assert_eq!(
            with_first_prompt(Some("Fix the login bug")).as_deref(),
            Some("Fix the login bug")
        );
    }
    #[test]
    fn first_prompt_preserves_nil_explicitly() {
        assert_eq!(with_first_prompt(None), None);
    }
    #[test]
    fn first_prompt_preserves_empty_string() {
        assert_eq!(with_first_prompt(Some("")).as_deref(), Some(""));
    }

    // Ported from PoirotTests/Models/SessionXMLStrippingTests.swift.
    fn titled(t: &str) -> String {
        session("t", "/path", vec![user(t)]).title()
    }
    #[test]
    fn title_strips_tag_markers_preserves_inner_text() {
        assert_eq!(
            titled("<system-reminder>ignored</system-reminder>Fix the bug"),
            "ignoredFix the bug"
        );
    }
    #[test]
    fn title_strips_multiple_tag_markers() {
        assert_eq!(titled("<tag1>a</tag1>Hello<tag2>b</tag2> world"), "aHellob world");
    }
    #[test]
    fn title_strips_nested_tag_markers() {
        assert_eq!(
            titled("<outer><inner>content</inner></outer>Clean text"),
            "contentClean text"
        );
    }
    #[test]
    fn title_preserves_plain_text() {
        assert_eq!(titled("No XML here, just plain text"), "No XML here, just plain text");
    }
    #[test]
    fn title_strips_tag_markers_from_cached_title() {
        let mut s = session("t", "/path", vec![]);
        s.cached_title = Some("<reminder>junk</reminder>Actual title".into());
        assert_eq!(s.title(), "junkActual title");
    }
    #[test]
    fn title_only_tag_markers_inner_text_preserved() {
        assert_eq!(titled("<tag>content</tag>"), "content");
    }
    #[test]
    fn title_only_empty_tags_falls_back_to_original() {
        assert_eq!(titled("<tag> </tag>"), "<tag> </tag>");
    }
    #[test]
    fn title_trims_whitespace_after_stripping() {
        assert_eq!(titled("  <tag>x</tag>  Hello  "), "x  Hello");
    }
    #[test]
    fn title_self_closing_tags_stripped() {
        assert_eq!(titled("Hello<br/>world"), "Helloworld");
    }
    #[test]
    fn title_angled_brackets_matched_as_tag() {
        assert_eq!(titled("a < b and c > d"), "a  d");
    }
    #[test]
    fn preview_strips_tag_markers_preserves_content() {
        assert_eq!(
            session("t", "/path", vec![assistant("<system>ctx</system>Here is the answer")])
                .preview()
                .as_deref(),
            Some("ctxHere is the answer")
        );
    }
    #[test]
    fn preview_plain_text_unchanged() {
        assert_eq!(
            session("t", "/path", vec![assistant("Simple response")])
                .preview()
                .as_deref(),
            Some("Simple response")
        );
    }
    #[test]
    fn preview_nil_when_no_assistant_message() {
        assert_eq!(session("t", "/path", vec![user("Hello")]).preview(), None);
    }
    #[test]
    fn preview_cached_preview_takes_priority() {
        let mut s = session("t", "/path", vec![assistant("From message")]);
        s.cached_preview = Some("From cache".into());
        assert_eq!(s.preview().as_deref(), Some("From cache"));
    }
    #[test]
    fn preview_cached_preview_also_strips_tag_markers() {
        let mut s = session("t", "/path", vec![]);
        s.cached_preview = Some("<tag>junk</tag>Clean preview".into());
        assert_eq!(s.preview().as_deref(), Some("junkClean preview"));
    }
    #[test]
    fn preview_nil_for_empty_messages() {
        assert_eq!(session("t", "/path", vec![]).preview(), None);
    }

    #[test]
    fn summary_serialises_camel_case_with_iso_dates() {
        let json = serde_json::to_value(session("s", "/p", vec![user("hi")]).summary("proj")).unwrap();
        assert_eq!(json["projectId"], "proj");
        assert_eq!(json["turnCount"], 1);
        assert_eq!(json["startedAt"], "1970-01-01T00:00:00Z");
    }
}
