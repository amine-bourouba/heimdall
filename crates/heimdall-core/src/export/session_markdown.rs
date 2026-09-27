//! Session Markdown export (Services/Export/SessionExporter.swift): a session or a single tool
//! block rendered as Markdown, with the four toggles `ExportOptionsView` offers.

use std::fmt::Display;

use chrono::{DateTime, Local, TimeZone, Utc};
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::models::message::{ContentBlock, Message, Role, ToolResult, ToolUse};
use crate::models::session::Session;

/// The four export toggles (`ExportOptionsView`); Poirot's defaults are kept.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportOptions {
    pub timestamps: bool,
    pub tool_results: bool,
    pub thinking: bool,
    pub token_usage: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            timestamps: true,
            tool_results: true,
            thinking: false,
            token_usage: false,
        }
    }
}

/// Tool input values longer than this are truncated with `…` (500 in Poirot, but Characters; this
/// counts Unicode scalars, the difference noted elsewhere in the port for very long non-ASCII text).
const INPUT_TRUNCATE: usize = 500;
/// Same, for a tool result's output.
const RESULT_TRUNCATE: usize = 2000;

pub fn to_markdown(session: &Session, options: &ExportOptions) -> String {
    to_markdown_in(session, options, &Local)
}

pub fn to_markdown_in<Tz: TimeZone>(session: &Session, options: &ExportOptions, tz: &Tz) -> String
where
    Tz::Offset: Display,
{
    let mut lines = vec![
        format!("# {}", session.title()),
        String::new(),
        format!("**Project:** {}", session.project_name()),
    ];
    if let Some(model) = &session.model {
        lines.push(format!("**Model:** {model}"));
    }
    lines.push(format!("**Date:** {}", format_session_date(session.started_at, tz)));
    lines.push(format!("**Turns:** {}", session.turn_count()));
    if session.total_tokens > 0 {
        lines.push(format!("**Tokens:** {}", format_token_count(session.total_tokens)));
    }
    lines.push(String::new());
    lines.push("---".into());
    lines.push(String::new());

    let mut turn_number = 0u32;
    for message in &session.messages {
        if message.role == Role::User {
            turn_number += 1;
        }
        let md = message_to_markdown_in(message, turn_number, options, tz);
        if !md.is_empty() {
            lines.push(md);
            lines.push(String::new());
            lines.push("---".into());
            lines.push(String::new());
        }
    }
    lines.join("\n")
}

pub fn message_to_markdown(message: &Message, turn_number: u32, options: &ExportOptions) -> String {
    message_to_markdown_in(message, turn_number, options, &Local)
}

fn message_to_markdown_in<Tz: TimeZone>(message: &Message, turn_number: u32, options: &ExportOptions, tz: &Tz) -> String
where
    Tz::Offset: Display,
{
    let role_label = if message.role == Role::User {
        "User"
    } else {
        "Assistant"
    };
    let turn_tag = if turn_number > 0 {
        format!(" {turn_number}")
    } else {
        String::new()
    };
    let mut lines = vec![format!("## Turn{turn_tag} — {role_label}")];

    // Poirot's Message.timestamp always has a value (missing ones default to `.distantPast`);
    // Heimdall's carries `None` instead, so the line is left out rather than printing a fake time.
    if options.timestamps
        && let Some(ts) = message.timestamp
    {
        lines.push(format!("*{}*", ts.with_timezone(tz).format("%H:%M:%S")));
    }
    if options.token_usage
        && let Some(usage) = &message.usage
    {
        lines.push(format!("*Tokens: {}*", usage.formatted()));
    }
    lines.push(String::new());

    for block in &message.blocks {
        match block {
            ContentBlock::Text { text, .. } => {
                lines.push(text.clone());
                lines.push(String::new());
            }
            ContentBlock::Thinking { text, .. } => {
                if !options.thinking {
                    continue;
                }
                lines.extend([
                    "<details>".into(),
                    "<summary>Thinking</summary>".into(),
                    String::new(),
                    text.clone(),
                    String::new(),
                    "</details>".into(),
                    String::new(),
                ]);
            }
            ContentBlock::ToolUse(tool) => {
                lines.extend(tool_block_lines(tool));
                lines.push(String::new());
            }
            ContentBlock::ToolResult(result) => {
                if !options.tool_results {
                    continue;
                }
                lines.extend(tool_result_lines(result));
                lines.push(String::new());
            }
        }
    }
    lines.join("\n")
}

/// A single tool block (call plus its result, if any) as Markdown — `SessionExporter.toolBlockToMarkdown`.
pub fn tool_block_to_markdown(tool: &ToolUse, result: Option<&ToolResult>) -> String {
    let mut lines = tool_block_lines(tool);
    if let Some(result) = result {
        lines.push(String::new());
        lines.extend(tool_result_lines(result));
    }
    lines.join("\n")
}

fn tool_block_lines(tool: &ToolUse) -> Vec<String> {
    if tool.is_bash()
        && let Some(command) = &tool.command
    {
        return vec!["### Tool: Bash".into(), "```bash".into(), command.clone(), "```".into()];
    }
    if tool.is_edit()
        && let Some(path) = &tool.file_path
    {
        let mut lines = vec![format!("### Tool: Edit — `{path}`")];
        if tool.has_diff_data() {
            let edit = tool.edit.as_ref().expect("has_diff_data implies edit is Some");
            lines.push("```diff".into());
            lines.extend(edit.old_string.split('\n').map(|l| format!("- {l}")));
            lines.extend(edit.new_string.split('\n').map(|l| format!("+ {l}")));
            lines.push("```".into());
        }
        return lines;
    }
    let mut lines = vec![format!("### Tool: {}", tool.name)];
    if let Some(path) = &tool.file_path {
        lines.push(format!("`{path}`"));
    }
    // A BTreeMap, unlike Poirot's `[String: String]`, so the field order here is stable — a
    // deliberate improvement, not a literal port of Swift's unordered dictionary iteration.
    for (key, value) in &tool.input {
        if key == "file_path" || key == "path" {
            continue;
        }
        lines.push(format!("**{key}:** `{}`", truncate_chars(value, INPUT_TRUNCATE)));
    }
    lines
}

fn tool_result_lines(result: &ToolResult) -> Vec<String> {
    let label = if result.is_error { "Error" } else { "Output" };
    vec![
        format!("**{label}:**"),
        "```".into(),
        truncate_chars(&result.content, RESULT_TRUNCATE),
        "```".into(),
    ]
}

fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() > max {
        format!("{}...", s.chars().take(max).collect::<String>())
    } else {
        s.to_string()
    }
}

fn format_token_count(count: u64) -> String {
    if count >= 1000 {
        format!("{:.1}k", count as f64 / 1000.0)
    } else {
        count.to_string()
    }
}

/// A fixed, English long-date format ("September 26, 2026 at 2:30 PM" in `tz`) standing in for
/// Swift's locale- and timezone-dependent `DateFormatter(.long, .short)`, which has no
/// byte-for-byte Rust equivalent — and no fixed one even in Swift, across locales.
fn format_session_date<Tz: TimeZone>(dt: DateTime<Utc>, tz: &Tz) -> String
where
    Tz::Offset: Display,
{
    dt.with_timezone(tz).format("%B %-d, %Y at %-I:%M %p").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transcript::full::{ParseContext, parse};
    use chrono::FixedOffset;
    use std::path::Path;

    fn conv_session() -> Session {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/screenshot/claude-home/projects/-Users-fixture-Dev-poirot/conv-001.jsonl");
        let ctx = ParseContext {
            project_path: "/test",
            session_id: "conv-001",
            index_started_at: None,
            is_sidechain: false,
        };
        parse(&path, ctx).unwrap().value.unwrap()
    }
    fn utc() -> FixedOffset {
        FixedOffset::east_opt(0).unwrap()
    }
    fn md(session: &Session, options: &ExportOptions) -> String {
        to_markdown_in(session, options, &utc())
    }

    #[test]
    fn header_carries_the_session_metadata() {
        let s = conv_session();
        let out = md(&s, &ExportOptions::default());
        assert!(out.starts_with(&format!("# {}\n\n**Project:** {}\n", s.title(), s.project_name())));
        assert!(out.contains(&format!("**Model:** {}", s.model.as_deref().unwrap())));
        assert!(out.contains(&format!("**Turns:** {}", s.turn_count())));
        assert!(out.contains(&format!("**Tokens:** {}", format_token_count(s.total_tokens))));
    }

    #[test]
    fn timestamps_toggle_controls_the_per_message_time_line() {
        let s = conv_session();
        let on = md(
            &s,
            &ExportOptions {
                timestamps: true,
                ..Default::default()
            },
        );
        let off = md(
            &s,
            &ExportOptions {
                timestamps: false,
                ..Default::default()
            },
        );
        assert!(on.contains('*') && on.lines().any(|l| l.starts_with('*') && l.trim_matches('*').len() == 8));
        assert!(
            !off.lines()
                .any(|l| l.starts_with('*') && l.trim_matches('*').len() == 8)
        );
    }

    #[test]
    fn tool_results_toggle_controls_output_blocks() {
        let s = conv_session();
        let on = md(
            &s,
            &ExportOptions {
                tool_results: true,
                ..Default::default()
            },
        );
        let off = md(
            &s,
            &ExportOptions {
                tool_results: false,
                ..Default::default()
            },
        );
        assert!(on.contains("**Output:**") && on.contains("Successfully edited"));
        assert!(!off.contains("**Output:**") && !off.contains("Successfully edited"));
    }

    #[test]
    fn thinking_toggle_controls_the_details_block() {
        let s = conv_session();
        let on = md(
            &s,
            &ExportOptions {
                thinking: true,
                ..Default::default()
            },
        );
        let off = md(
            &s,
            &ExportOptions {
                thinking: false,
                ..Default::default()
            },
        );
        assert!(on.contains("<summary>Thinking</summary>") && on.contains("session count badges in the sidebar"));
        assert!(!off.contains("<summary>Thinking</summary>"));
    }

    #[test]
    fn token_usage_toggle_controls_the_per_message_tokens_line() {
        let s = conv_session();
        let on = md(
            &s,
            &ExportOptions {
                token_usage: true,
                ..Default::default()
            },
        );
        let off = md(
            &s,
            &ExportOptions {
                token_usage: false,
                ..Default::default()
            },
        );
        assert!(on.contains("*Tokens: "));
        assert!(!off.contains("*Tokens: "));
    }

    #[test]
    fn bash_and_edit_tools_render_their_own_shapes() {
        let out = md(&conv_session(), &ExportOptions::default());
        assert!(out.contains("### Tool: Bash") && out.contains("```bash") && out.contains("xcodebuild build"));
        assert!(out.contains("### Tool: Edit — `Poirot/Sources/Views/Components/SidebarView.swift`"));
        assert!(out.contains("```diff"), "{out}");
        assert!(out.contains("- Text(project.name)"), "{out}");
        assert!(out.contains("+ Text(project.name)"), "{out}");
        // Exact indentation isn't asserted here (it is the fixture's own, "+ " plus whatever
        // leading whitespace that line already had); just that the added line is present.
        assert!(
            out.lines()
                .any(|l| l.strip_prefix('+').is_some_and(|rest| rest.trim() == "Spacer()")),
            "{out}"
        );
    }

    #[test]
    fn a_non_edit_non_bash_tool_lists_its_input_sorted_by_key() {
        let tool = ToolUse::from_input(
            "id",
            "id",
            "Grep",
            &[
                ("pattern".into(), "foo".into()),
                ("path".into(), "/x".into()),
                ("glob".into(), "*.rs".into()),
            ]
            .into(),
            "",
        );
        let lines = tool_block_lines(&tool);
        assert_eq!(lines[0], "### Tool: Grep");
        assert_eq!(lines[1], "`/x`"); // shown once, from file_path/path, before the input list
        // "path" is excluded from the input list itself; the rest come out key-sorted (Poirot's
        // Swift dictionary had no defined order at all).
        assert_eq!(
            lines[2..],
            ["**glob:** `*.rs`".to_string(), "**pattern:** `foo`".to_string()]
        );
    }

    #[test]
    fn long_values_are_truncated() {
        assert_eq!(
            truncate_chars(&"a".repeat(600), INPUT_TRUNCATE),
            format!("{}...", "a".repeat(500))
        );
        assert_eq!(truncate_chars("short", INPUT_TRUNCATE), "short");
    }

    #[test]
    fn tool_block_to_markdown_matches_the_in_message_rendering() {
        let tool = ToolUse::from_input("id", "id", "Bash", &[("command".into(), "ls -la".into())].into(), "");
        let result = ToolResult {
            id: "r".into(),
            tool_use_id: "id".into(),
            content: "file1\nfile2".into(),
            is_error: false,
        };
        let out = tool_block_to_markdown(&tool, Some(&result));
        assert!(
            out.contains("### Tool: Bash")
                && out.contains("ls -la")
                && out.contains("**Output:**")
                && out.contains("file1\nfile2")
        );
        assert!(!tool_block_to_markdown(&tool, None).contains("Output"));
    }

    #[test]
    fn format_token_count_matches_poirot() {
        assert_eq!(
            (
                format_token_count(999),
                format_token_count(1000),
                format_token_count(12_345)
            ),
            ("999".into(), "1.0k".into(), "12.3k".into())
        );
    }
}
