//! Full-parse tier (TranscriptParser.parse): user and assistant records become messages, and
//! consecutive assistant records with the same message id merge into one message. Every other
//! record type (system, summary, progress, file-history-*) is passed over.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::blocks;
use super::searchable::Searchable;
use crate::error::{AppError, Loaded, Result};
use crate::jsonl;
use crate::models::message::{Message, Role, SYNTHETIC_MODEL};
use crate::models::session::Session;
use crate::models::token_usage::TokenUsage;

/// Where a transcript belongs; `is_sidechain` keeps sidechain records (sub-agent transcripts).
#[derive(Debug, Clone, Copy)]
pub struct ParseContext<'a> {
    pub project_path: &'a str,
    pub session_id: &'a str,
    pub index_started_at: Option<DateTime<Utc>>,
    pub is_sidechain: bool,
}

/// Parses the transcript at `path`. `value` is `None` when it holds no user or assistant message;
/// `skipped` counts lines that were not JSON.
pub fn parse(path: &Path, ctx: ParseContext) -> Result<Loaded<Option<Session>>> {
    let mut records = jsonl::open::<Value>(path)?;
    let mut session = parse_records(records.by_ref(), ctx);
    if let Some(e) = records.take_io_error() {
        return Err(AppError::from_io(&e, path));
    }
    if let Some(s) = &mut session {
        s.file = Some(path.to_path_buf());
    }
    Ok(Loaded::new(session, records.skipped()))
}

/// Parses only enough of the transcript to materialise at least `min_messages` messages, without
/// reading past what a page needs — `get_session_messages`'s (M1-41) fast path for a session
/// whose full parse is not cached yet, so its first page renders before a huge transcript's whole
/// file has been read. Returns whether the read actually reached the end of the file: when it
/// did, the result is already a complete, cacheable parse; when it did not, `messages.len()` is
/// only a lower bound and the caller falls back to [`parse`] for an exact count.
///
/// Re-reads from the start of the file and doubles its own bound each time it comes up short
/// (rare — almost every session fits in the first bound) rather than threading a resumable
/// position through `parse_records`'s merge-across-records logic (a message can span several
/// records, so "stop at record N" cannot simply resume at record N later without re-checking
/// whether N's message continues past it).
pub fn parse_bounded(path: &Path, ctx: ParseContext, min_messages: usize) -> Result<(Loaded<Option<Session>>, bool)> {
    let mut bound = min_messages.saturating_mul(4).max(64);
    loop {
        let mut records = jsonl::open::<Value>(path)?;
        let session = parse_records(records.by_ref().take(bound), ctx);
        let reached_eof = records.next().is_none();
        if let Some(e) = records.take_io_error() {
            return Err(AppError::from_io(&e, path));
        }
        let enough = session.as_ref().is_some_and(|s| s.messages.len() >= min_messages);
        if enough || reached_eof {
            let mut session = session;
            if let Some(s) = &mut session {
                s.file = Some(path.to_path_buf());
            }
            return Ok((Loaded::new(session, records.skipped()), reached_eof));
        }
        bound *= 4;
    }
}

pub fn parse_records(records: impl Iterator<Item = Value>, ctx: ParseContext) -> Option<Session> {
    let empty = Value::Object(serde_json::Map::new());
    let mut messages: Vec<Message> = Vec::new();
    let mut searchable = Searchable::default();
    let mut pending: Option<Message> = None;
    // Every record of one API message repeats its usage, and output tokens grow while it streams,
    // so the last record's usage is the message's (D5). Tool results sit between those records, so
    // one message can form several groups; its usage goes on the first group only.
    let mut usage_by_id: HashMap<String, TokenUsage> = HashMap::new();

    for (n, record) in records.enumerate() {
        let Some(kind @ ("user" | "assistant")) = record.get("type").and_then(Value::as_str) else { continue };
        if !ctx.is_sidechain && record.get("isSidechain") == Some(&Value::Bool(true)) {
            continue;
        }
        let message = record.get("message").filter(|m| m.is_object()).unwrap_or(&empty);
        let model = message.get("model").and_then(Value::as_str);
        if kind == "assistant" && model == Some(SYNTHETIC_MODEL) {
            continue;
        }
        let timestamp = blocks::timestamp(record.get("timestamp"));
        searchable.append(message, kind);

        // Records without an id get one from their position, so ids stay stable across re-parses.
        if kind == "user" {
            flush(&mut pending, &mut messages);
            let id = record.get("uuid").and_then(Value::as_str).map_or_else(|| format!("record-{n}"), String::from);
            let mut blocks = blocks::user_blocks(message.get("content"));
            blocks::assign_ids(&id, &mut blocks);
            messages.push(Message { id, role: Role::User, timestamp, model: None, usage: None, blocks });
        } else {
            let id = message.get("id").and_then(Value::as_str).map_or_else(|| format!("record-{n}"), String::from);
            let new_blocks = blocks::assistant_blocks(message.get("content"));
            if let Some(u) = blocks::usage(message.get("usage")) {
                usage_by_id.insert(id.clone(), u);
            }
            match &mut pending {
                Some(group) if group.id == id => group.blocks.extend(new_blocks),
                _ => {
                    flush(&mut pending, &mut messages);
                    pending = Some(Message {
                        id,
                        role: Role::Assistant,
                        timestamp,
                        model: model.map(String::from),
                        usage: None,
                        blocks: new_blocks,
                    });
                }
            }
        }
    }
    flush(&mut pending, &mut messages);
    if messages.is_empty() {
        return None;
    }

    let started_at = messages.iter().filter_map(|m| m.timestamp).min().or(ctx.index_started_at).unwrap_or(DateTime::UNIX_EPOCH);
    let model = messages.iter().find(|m| m.role == Role::Assistant).and_then(|m| m.model.clone());
    let mut placed: HashSet<String> = HashSet::new();
    for m in messages.iter_mut().filter(|m| m.role == Role::Assistant) {
        if placed.insert(m.id.clone()) {
            m.usage = usage_by_id.get(&m.id).copied();
        }
    }
    let tokens = usage_by_id.values().fold(TokenUsage::default(), |acc, u| acc.add(u));
    Some(Session {
        total_tokens: tokens.total(),
        tokens,
        searchable_text: searchable.finish(),
        ..Session::new(ctx.session_id, ctx.project_path, messages, started_at, model, 0)
    })
}

/// Closes the open assistant group; a group with no blocks is dropped.
fn flush(pending: &mut Option<Message>, messages: &mut Vec<Message>) {
    if let Some(mut group) = pending.take().filter(|g| !g.blocks.is_empty()) {
        let id = group.id.clone();
        blocks::assign_ids(&id, &mut group.blocks);
        messages.push(group);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonl::Records;
    use crate::models::message::ContentBlock;
    use serde_json::json;

    const CTX: ParseContext = ParseContext { project_path: "/test", session_id: "s1", index_started_at: None, is_sidechain: false };

    fn parse_lines(lines: &[String]) -> Option<Session> {
        let text = lines.join("\n");
        parse_records(Records::<_, Value>::new(text.as_bytes()), CTX)
    }

    fn write_transcript(lines: &[String]) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("t.jsonl");
        std::fs::write(&path, lines.join("\n")).unwrap();
        (dir, path)
    }

    fn user(content: Value) -> String {
        user_with(content, "2026-01-28T10:00:00.000Z", false, None)
    }
    fn user_with(content: Value, timestamp: &str, sidechain: bool, uuid: Option<&str>) -> String {
        json!({"type": "user", "uuid": uuid.unwrap_or("u-default"), "timestamp": timestamp, "isSidechain": sidechain,
               "message": {"role": "user", "content": content}}).to_string()
    }
    fn assistant(content: Value) -> String {
        assistant_with(content, "msg_123", "claude-opus-4-6", "2026-01-28T10:01:00.000Z", (100, 50))
    }
    fn assistant_with(content: Value, msg_id: &str, model: &str, timestamp: &str, tokens: (u64, u64)) -> String {
        json!({"type": "assistant", "uuid": "a-uuid", "timestamp": timestamp, "isSidechain": false,
               "message": {"id": msg_id, "role": "assistant", "model": model, "content": content,
                           "usage": {"input_tokens": tokens.0, "output_tokens": tokens.1}}}).to_string()
    }
    /// `n` independent user/assistant turns, each with its own id — 2n messages total.
    fn turns(n: usize) -> Vec<String> {
        (0..n)
            .flat_map(|i| {
                [
                    user_with(json!(format!("q{i}")), &format!("2026-01-28T10:{i:02}:00.000Z"), false, Some(&format!("u{i}"))),
                    assistant_with(json!([{"type": "text", "text": format!("a{i}")}]), &format!("msg_{i}"), "claude-opus-4-6", &format!("2026-01-28T10:{i:02}:01.000Z"), (10, 5)),
                ]
            })
            .collect()
    }

    #[test]
    fn parse_bounded_stops_short_of_a_huge_transcript() {
        let (_dir, path) = write_transcript(&turns(1000));
        let (loaded, reached_eof) = parse_bounded(&path, CTX, 4).unwrap();
        let messages = loaded.value.unwrap().messages;
        assert!(!reached_eof);
        assert!((4..2000).contains(&messages.len()), "got {} messages", messages.len());
        // The window it did read is a correct, ordered prefix of the full parse.
        let full = parse(&path, CTX).unwrap().value.unwrap().messages;
        assert_eq!(messages, full[..messages.len()]);
    }

    #[test]
    fn parse_bounded_reaches_eof_and_matches_the_full_parse_for_a_small_transcript() {
        let (_dir, path) = write_transcript(&turns(3));
        let (loaded, reached_eof) = parse_bounded(&path, CTX, 4).unwrap();
        assert!(reached_eof);
        assert_eq!(loaded.value, parse(&path, CTX).unwrap().value);
    }

    #[test]
    fn parse_bounded_widens_its_read_when_the_first_bound_comes_up_short() {
        // min_messages (2) starts a bound of 64 records; 70 skipped-type records push the real
        // turns past that first bound, forcing a re-read at the next (wider) bound.
        let mut lines: Vec<String> = (0..70).map(|_| r#"{"type":"progress"}"#.to_string()).collect();
        lines.extend(turns(2));
        let (_dir, path) = write_transcript(&lines);
        let (loaded, reached_eof) = parse_bounded(&path, CTX, 2).unwrap();
        assert!(reached_eof);
        assert_eq!(loaded.value.unwrap().messages.len(), 4);
    }

    fn first_block(s: &Option<Session>) -> &ContentBlock {
        &s.as_ref().unwrap().messages[0].blocks[0]
    }
    fn text_of(b: &ContentBlock) -> &str {
        match b {
            ContentBlock::Text { text, .. } | ContentBlock::Thinking { text, .. } => text,
            other => panic!("not text: {other:?}"),
        }
    }

    // Ported from PoirotTests/Services/TranscriptParserTests.swift (full-parse cases).
    #[test]
    fn parse_empty_file_returns_nil() {
        assert!(parse_lines(&[String::new()]).is_none());
    }
    #[test]
    fn parse_only_skipped_record_types_returns_nil() {
        let lines = [
            r#"{"type":"progress","data":{"type":"hook_progress"},"timestamp":"2026-01-28T10:00:00.000Z"}"#.to_string(),
            r#"{"type":"file-history-snapshot","snapshot":{},"timestamp":"2026-01-28T10:00:01.000Z"}"#.to_string(),
            r#"{"type":"file-history-delta","delta":{}}"#.to_string(),
            r#"{"type":"system","content":"x"}"#.to_string(),
            r#"{"type":"summary","summary":"x","leafUuid":"u"}"#.to_string(),
        ];
        assert!(parse_lines(&lines).is_none());
    }
    #[test]
    fn parse_malformed_lines_skips_gracefully() {
        let text = ["not valid json".to_string(), "{broken json".to_string(), user(json!("Hello"))].join("\n");
        let mut records = Records::<_, Value>::new(text.as_bytes());
        let s = parse_records(records.by_ref(), CTX).unwrap();
        assert_eq!((s.messages.len(), records.skipped()), (1, 2));
    }
    #[test]
    fn parse_sidechain_records_skipped() {
        let lines = [user_with(json!("Hello"), "2026-01-28T10:00:00.000Z", true, None), user(json!("World"))];
        let s = parse_lines(&lines).unwrap();
        assert_eq!(s.messages.len(), 1);
        assert_eq!(s.messages[0].text_content(), "World");
        let kept = parse_records(Records::<_, Value>::new(lines.join("\n").as_bytes()), ParseContext { is_sidechain: true, ..CTX });
        assert_eq!(kept.unwrap().messages.len(), 2);
    }
    #[test]
    fn parse_user_string_content_creates_text_block() {
        let s = parse_lines(&[user(json!("Hello world"))]);
        let m = &s.as_ref().unwrap().messages[0];
        assert_eq!((m.role, m.blocks.len(), text_of(&m.blocks[0])), (Role::User, 1, "Hello world"));
    }
    #[test]
    fn parse_user_tool_result_creates_tool_result_block() {
        let s = parse_lines(&[user(json!([{"type": "tool_result", "tool_use_id": "toolu_abc", "content": "File created successfully"}]))]);
        let ContentBlock::ToolResult(r) = first_block(&s) else { panic!("expected tool result") };
        assert_eq!((r.tool_use_id.as_str(), r.content.as_str(), r.is_error), ("toolu_abc", "File created successfully", false));
    }
    #[test]
    fn parse_tool_result_string_content_normalized() {
        let s = parse_lines(&[user(json!([{"type": "tool_result", "tool_use_id": "toolu_1", "content": "Simple string content"}]))]);
        let ContentBlock::ToolResult(r) = first_block(&s) else { panic!() };
        assert_eq!(r.content, "Simple string content");
    }
    #[test]
    fn parse_tool_result_array_content_extracts_text_only() {
        let content = json!([{"type": "text", "text": "Line 1"}, {"type": "tool_reference", "ref": "something"}, {"type": "text", "text": "Line 2"}]);
        let s = parse_lines(&[user(json!([{"type": "tool_result", "tool_use_id": "toolu_2", "content": content}]))]);
        let ContentBlock::ToolResult(r) = first_block(&s) else { panic!() };
        assert_eq!(r.content, "Line 1\nLine 2");
    }
    #[test]
    fn parse_tool_result_is_error_flagged() {
        let s = parse_lines(&[user(json!([{"type": "tool_result", "tool_use_id": "toolu_err", "content": "Error", "is_error": true}]))]);
        let ContentBlock::ToolResult(r) = first_block(&s) else { panic!() };
        assert!(r.is_error);
    }
    #[test]
    fn parse_assistant_text_block() {
        let s = parse_lines(&[assistant(json!([{"type": "text", "text": "Hello from assistant"}]))]);
        let m = &s.as_ref().unwrap().messages[0];
        assert_eq!((m.role, m.blocks.len(), text_of(&m.blocks[0])), (Role::Assistant, 1, "Hello from assistant"));
    }
    #[test]
    fn parse_assistant_thinking_block() {
        let s = parse_lines(&[assistant(json!([{"type": "thinking", "thinking": "Let me think about this"}]))]);
        assert!(matches!(first_block(&s), ContentBlock::Thinking { text, .. } if text == "Let me think about this"));
    }
    #[test]
    fn parse_assistant_tool_use_block() {
        let s = parse_lines(&[assistant(json!([{"type": "tool_use", "id": "toolu_xyz", "name": "Read", "input": {"file_path": "/foo/bar.swift"}}]))]);
        let ContentBlock::ToolUse(t) = first_block(&s) else { panic!() };
        assert_eq!((t.tool_use_id.as_str(), t.name.as_str()), ("toolu_xyz", "Read"));
        assert_eq!(t.input["file_path"], "/foo/bar.swift");
        assert_eq!(t.file_path.as_deref(), Some("/foo/bar.swift"));
    }
    #[test]
    fn parse_tool_use_non_string_inputs_stringified() {
        let input = json!({"command": "ls", "timeout": 5000, "dangerouslyDisableSandbox": true});
        let s = parse_lines(&[assistant(json!([{"type": "tool_use", "id": "toolu_num", "name": "Bash", "input": input}]))]);
        let ContentBlock::ToolUse(t) = first_block(&s) else { panic!() };
        assert_eq!((t.input["command"].as_str(), t.input["timeout"].as_str(), t.input["dangerouslyDisableSandbox"].as_str()), ("ls", "5000", "true"));
        assert!(t.input_pretty.contains("\"timeout\": 5000"));
    }
    #[test]
    fn parse_synthetic_model_skipped() {
        let lines = [assistant_with(json!([{"type": "text", "text": "Synthetic"}]), "m", SYNTHETIC_MODEL, "2026-01-28T10:01:00.000Z", (1, 1)), user(json!("Real user message"))];
        let s = parse_lines(&lines).unwrap();
        assert_eq!((s.messages.len(), s.messages[0].role), (1, Role::User));
    }
    #[test]
    fn parse_multiple_records_same_id_grouped() {
        let lines = [
            assistant_with(json!([{"type": "text", "text": "Part 1"}]), "msg_grouped", "claude-opus-4-6", "2026-01-28T10:01:00.000Z", (100, 50)),
            assistant_with(json!([{"type": "tool_use", "id": "t1", "name": "Read", "input": {"path": "/a"}}]), "msg_grouped", "claude-opus-4-6", "2026-01-28T10:01:01.000Z", (100, 50)),
        ];
        let s = parse_lines(&lines).unwrap();
        let m = &s.messages[0];
        assert_eq!((s.messages.len(), m.role, m.blocks.len(), text_of(&m.blocks[0])), (1, Role::Assistant, 2, "Part 1"));
        assert!(matches!(&m.blocks[1], ContentBlock::ToolUse(t) if t.name == "Read"));
    }
    #[test]
    /// Deviation (D5): the group's usage is its last record's, where Poirot kept the first (100, 50).
    fn parse_usage_deduplicated_per_group() {
        let lines = [
            assistant_with(json!([{"type": "text", "text": "A"}]), "msg_dup", "claude-opus-4-6", "2026-01-28T10:01:00.000Z", (100, 50)),
            assistant_with(json!([{"type": "text", "text": "B"}]), "msg_dup", "claude-opus-4-6", "2026-01-28T10:01:00.000Z", (200, 100)),
        ];
        let s = parse_lines(&lines).unwrap();
        let u = s.messages[0].usage.unwrap();
        assert_eq!((s.messages.len(), u.input, u.output, s.total_tokens), (1, 200, 100, 300));
    }
    #[test]
    fn usage_counts_once_when_tool_results_split_a_message() {
        let tool = |id: &str| json!([{"type": "tool_use", "id": id, "name": "Read", "input": {}}]);
        let lines = [
            assistant_with(json!([{"type": "text", "text": "A"}]), "msg_1", "m", "2026-01-28T10:01:00.000Z", (2, 8)),
            assistant_with(tool("t1"), "msg_1", "m", "2026-01-28T10:01:01.000Z", (2, 8)),
            user(json!([{"type": "tool_result", "tool_use_id": "t1", "content": "ok"}])),
            assistant_with(tool("t2"), "msg_1", "m", "2026-01-28T10:01:02.000Z", (2, 209)),
        ];
        let s = parse_lines(&lines).unwrap();
        assert_eq!(s.messages.len(), 3);
        assert_eq!((s.messages[0].usage, s.messages[2].usage, s.total_tokens), (Some(TokenUsage::new(2, 209)), None, 211));
    }
    #[test]
    fn parse_started_at_uses_earliest_timestamp() {
        let lines = [user_with(json!("First"), "2026-01-28T09:00:00.000Z", false, None), assistant_with(json!([{"type": "text", "text": "Reply"}]), "m", "x", "2026-01-28T10:00:00.000Z", (1, 1))];
        assert_eq!(parse_lines(&lines).unwrap().started_at.to_rfc3339(), "2026-01-28T09:00:00+00:00");
    }
    #[test]
    fn parse_model_from_first_assistant_group() {
        let lines = [
            user(json!("Hi")),
            assistant_with(json!([{"type": "text", "text": "A"}]), "msg_1", "claude-sonnet-4-5-20250514", "2026-01-28T10:01:00.000Z", (1, 1)),
            assistant_with(json!([{"type": "text", "text": "B"}]), "msg_2", "claude-opus-4-6", "2026-01-28T10:02:00.000Z", (1, 1)),
        ];
        assert_eq!(parse_lines(&lines).unwrap().model.as_deref(), Some("claude-sonnet-4-5-20250514"));
    }
    #[test]
    fn parse_total_tokens_summed_across_groups() {
        let lines = [
            user(json!("Hi")),
            assistant_with(json!([{"type": "text", "text": "A"}]), "msg_1", "m", "2026-01-28T10:01:00.000Z", (100, 50)),
            assistant_with(json!([{"type": "text", "text": "B"}]), "msg_2", "m", "2026-01-28T10:02:00.000Z", (200, 100)),
        ];
        assert_eq!(parse_lines(&lines).unwrap().total_tokens, 450);
    }
    #[test]
    fn parse_interleaved_user_and_assistant_preserves_order() {
        let lines = [
            user_with(json!("Q1"), "2026-01-28T10:00:00.000Z", false, Some("u1")),
            assistant_with(json!([{"type": "text", "text": "A1"}]), "msg_a1", "m", "2026-01-28T10:01:00.000Z", (1, 1)),
            user_with(json!("Q2"), "2026-01-28T10:02:00.000Z", false, Some("u2")),
            assistant_with(json!([{"type": "text", "text": "A2"}]), "msg_a2", "m", "2026-01-28T10:03:00.000Z", (1, 1)),
        ];
        let s = parse_lines(&lines).unwrap();
        let got: Vec<_> = s.messages.iter().map(|m| (m.role, m.text_content())).collect();
        let want = [(Role::User, "Q1"), (Role::Assistant, "A1"), (Role::User, "Q2"), (Role::Assistant, "A2")].map(|(r, t)| (r, t.to_string()));
        assert_eq!(got, want);
    }
    #[test]
    fn parse_searchable_text_indexes_content() {
        let s = parse_lines(&[user(json!("FindMe in Content")), assistant(json!([{"type": "text", "text": "Here is the Response"}]))]).unwrap();
        assert!(s.searchable_text.contains("findme in content") && s.searchable_text.contains("here is the response"));
    }

    #[test]
    fn block_ids_are_message_id_and_index_and_stable() {
        let lines = [user_with(json!([{"type": "text", "text": "q"}, {"type": "tool_result", "tool_use_id": "t", "content": "r"}]), "2026-01-28T10:00:00.000Z", false, Some("u1"))];
        let ids = |s: Session| s.messages[0].blocks.iter().map(|b| match b {
            ContentBlock::Text { id, .. } | ContentBlock::Thinking { id, .. } => id.clone(),
            ContentBlock::ToolUse(t) => t.id.clone(),
            ContentBlock::ToolResult(r) => r.id.clone(),
        }).collect::<Vec<_>>();
        let first = ids(parse_lines(&lines).unwrap());
        assert_eq!(first, ["u1:0", "u1:1"]);
        assert_eq!(first, ids(parse_lines(&lines).unwrap()));
    }

    #[test]
    fn user_text_carries_system_segments() {
        let s = parse_lines(&[user(json!("fix it <system-reminder>be brief</system-reminder>"))]);
        let ContentBlock::Text { system, .. } = first_block(&s) else { panic!() };
        assert_eq!(system.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn every_windows_fixture_transcript_parses_without_skipped_lines() {
        fn walk(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
            for entry in std::fs::read_dir(dir).unwrap().flatten() {
                let p = entry.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|e| e == "jsonl") {
                    out.push(p);
                }
            }
        }
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows");
        let mut files = Vec::new();
        for projects in [root.join("claude-home/projects"), root.join("crlf/claude-home/projects")] {
            walk(&projects, &mut files);
        }
        assert!(files.len() >= 20, "found {} transcripts", files.len());
        for f in files {
            let loaded = parse(&f, ParseContext { is_sidechain: true, ..CTX }).unwrap();
            assert_eq!(loaded.skipped, 0, "{}", f.display());
        }
    }
}
