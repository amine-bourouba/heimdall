//! Summary tier (TranscriptParser.parseSummary): one pass over the file for the session list —
//! title, preview, turn count, model, tokens, time range and search text — without building messages.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::blocks;
use super::full::ParseContext;
use super::header::{first_assistant_text, first_user_text};
use super::searchable::Searchable;
use crate::error::{AppError, Loaded, Result};
use crate::jsonl;
use crate::models::message::SYNTHETIC_MODEL;
use crate::models::session::Session;
use crate::models::token_usage::TokenUsage;

/// The caller fills in `first_prompt` and the sub-agent fields it knows from the file's location.
pub fn parse(path: &Path, ctx: ParseContext) -> Result<Loaded<Option<Session>>> {
    let mut records = jsonl::open::<Value>(path)?;
    let mut session = summarize(records.by_ref(), ctx);
    if let Some(e) = records.take_io_error() {
        return Err(AppError::from_io(&e, path));
    }
    if let Some(s) = &mut session {
        s.file = Some(path.to_path_buf());
    }
    Ok(Loaded::new(session, records.skipped()))
}

pub fn summarize(records: impl Iterator<Item = Value>, ctx: ParseContext) -> Option<Session> {
    let mut title: Option<String> = None;
    let mut preview: Option<String> = None;
    let mut model: Option<String> = None;
    let mut user_count: u32 = 0;
    let mut earliest: Option<DateTime<Utc>> = None;
    let mut latest: Option<DateTime<Utc>> = None;
    let mut seen_ids: HashSet<String> = HashSet::new();
    let mut usage_by_id: HashMap<String, TokenUsage> = HashMap::new();
    let mut searchable = Searchable::default();
    let mut cwds: HashMap<String, u32> = HashMap::new();
    let empty = Value::Object(serde_json::Map::new());

    for (n, record) in records.enumerate() {
        if let Some(cwd) = record.get("cwd").and_then(Value::as_str) {
            *cwds.entry(cwd.to_string()).or_default() += 1;
        }
        let Some(kind @ ("user" | "assistant")) = record.get("type").and_then(Value::as_str) else {
            continue;
        };
        if !ctx.is_sidechain && record.get("isSidechain") == Some(&Value::Bool(true)) {
            continue;
        }
        let message = record.get("message").filter(|m| m.is_object()).unwrap_or(&empty);
        if kind == "assistant" && message.get("model").and_then(Value::as_str) == Some(SYNTHETIC_MODEL) {
            continue;
        }
        searchable.append(message, kind);
        if let Some(ts) = blocks::timestamp(record.get("timestamp")) {
            earliest = Some(earliest.map_or(ts, |e| e.min(ts)));
            latest = Some(latest.map_or(ts, |l| l.max(ts)));
        }
        if kind == "user" {
            user_count += 1;
            if title.is_none() {
                title = first_user_text(message.get("content"));
            }
            continue;
        }
        if model.is_none() {
            model = message.get("model").and_then(Value::as_str).map(String::from);
        }
        if preview.is_none() {
            preview = first_assistant_text(message.get("content"));
        }
        // Records of one API message repeat its usage and output grows while it streams, so the
        // last record's usage counts, once per message id (D5).
        let id = message
            .get("id")
            .and_then(Value::as_str)
            .map_or_else(|| format!("record-{n}"), String::from);
        if let Some(u) = blocks::usage(message.get("usage")) {
            usage_by_id.insert(id.clone(), u);
        }
        seen_ids.insert(id);
    }

    if user_count == 0 && seen_ids.is_empty() {
        return None;
    }
    let tokens = usage_by_id.values().fold(TokenUsage::default(), |acc, u| acc.add(u));
    Some(Session {
        tokens,
        ended_at: latest,
        cached_title: title,
        cached_preview: preview,
        cached_turn_count: Some(user_count),
        is_sidechain: ctx.is_sidechain,
        searchable_text: searchable.finish(),
        cwds,
        ..Session::new(
            ctx.session_id,
            ctx.project_path,
            Vec::new(),
            earliest.or(ctx.index_started_at).unwrap_or(DateTime::UNIX_EPOCH),
            model,
            tokens.total(),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jsonl::Records;
    use crate::transcript::full;
    use crate::transcript::searchable::SEARCHABLE_TEXT_CAP;
    use serde_json::json;

    const CTX: ParseContext = ParseContext {
        project_path: "/test",
        session_id: "s1",
        index_started_at: None,
        is_sidechain: false,
    };

    fn run(lines: &[String]) -> Option<Session> {
        summarize(Records::<_, Value>::new(lines.join("\n").as_bytes()), CTX)
    }
    fn user(content: Value, uuid: &str, timestamp: &str, sidechain: bool) -> String {
        json!({"type": "user", "uuid": uuid, "timestamp": timestamp, "isSidechain": sidechain, "message": {"role": "user", "content": content}}).to_string()
    }
    fn u(content: &str) -> String {
        user(json!(content), "u", "2026-01-28T10:00:00.000Z", false)
    }
    fn assistant(content: Value, msg_id: &str, model: &str, timestamp: &str, tokens: (u64, u64)) -> String {
        json!({"type": "assistant", "uuid": "a", "timestamp": timestamp, "isSidechain": false,
               "message": {"id": msg_id, "role": "assistant", "model": model, "content": content,
                           "usage": {"input_tokens": tokens.0, "output_tokens": tokens.1}}})
        .to_string()
    }
    fn a(text: &str) -> String {
        assistant(
            json!([{"type": "text", "text": text}]),
            "msg_123",
            "claude-opus-4-6",
            "2026-01-28T10:01:00.000Z",
            (100, 50),
        )
    }

    // Ported from PoirotTests/Services/TranscriptParserTests.swift (parseSummary cases).
    #[test]
    fn parse_summary_empty_file_returns_nil() {
        assert!(run(&[String::new()]).is_none());
    }
    #[test]
    fn parse_summary_extracts_title() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s1.jsonl");
        std::fs::write(&path, [u("Hello world"), a("Hi")].join("\n")).unwrap();
        let s = parse(&path, CTX).unwrap().value.unwrap();
        assert_eq!(s.title(), "Hello world");
        assert!(s.messages.is_empty());
        assert_eq!(s.file.as_deref(), Some(path.as_path()));
    }
    #[test]
    fn parse_summary_extracts_turn_count() {
        let lines = [
            user(json!("Q1"), "u1", "2026-01-28T10:00:00.000Z", false),
            assistant(
                json!([{"type": "text", "text": "A1"}]),
                "msg_a1",
                "m",
                "2026-01-28T10:01:00.000Z",
                (1, 1),
            ),
            user(json!("Q2"), "u2", "2026-01-28T10:02:00.000Z", false),
        ];
        assert_eq!(run(&lines).unwrap().turn_count(), 2);
    }
    #[test]
    fn parse_summary_extracts_model() {
        let lines = [
            u("Hello"),
            assistant(
                json!([{"type": "text", "text": "Hi"}]),
                "m1",
                "claude-sonnet-4-6",
                "2026-01-28T10:01:00.000Z",
                (1, 1),
            ),
        ];
        assert_eq!(run(&lines).unwrap().model.as_deref(), Some("claude-sonnet-4-6"));
    }
    #[test]
    /// Deviation (D5): msg_1 counts its last record (200 + 100), where Poirot counted the first
    /// (100 + 50) and expected 600.
    fn parse_summary_sums_tokens_per_group() {
        let lines = [
            u("Hi"),
            assistant(
                json!([{"type": "text", "text": "A"}]),
                "msg_1",
                "m",
                "2026-01-28T10:01:00.000Z",
                (100, 50),
            ),
            assistant(
                json!([{"type": "tool_use", "id": "t1", "name": "Read", "input": {"path": "/a"}}]),
                "msg_1",
                "m",
                "2026-01-28T10:01:01.000Z",
                (200, 100),
            ),
            assistant(
                json!([{"type": "text", "text": "B"}]),
                "msg_2",
                "m",
                "2026-01-28T10:02:00.000Z",
                (300, 150),
            ),
        ];
        assert_eq!(run(&lines).unwrap().total_tokens, 750);
    }
    #[test]
    fn parse_summary_skips_sidechain_records() {
        let lines = [
            user(json!("Sidechain"), "u1", "2026-01-28T10:00:00.000Z", true),
            user(json!("Real message"), "u2", "2026-01-28T10:00:00.000Z", false),
        ];
        let s = run(&lines).unwrap();
        assert_eq!((s.title().as_str(), s.turn_count()), ("Real message", 1));
    }
    #[test]
    fn parse_summary_skips_synthetic_assistant() {
        let lines = [
            assistant(
                json!([{"type": "text", "text": "Synthetic"}]),
                "m",
                SYNTHETIC_MODEL,
                "2026-01-28T10:01:00.000Z",
                (1, 1),
            ),
            u("Real user"),
        ];
        let s = run(&lines).unwrap();
        assert_eq!((s.model.as_deref(), s.turn_count()), (None, 1));
    }
    #[test]
    fn parse_summary_uses_earliest_timestamp() {
        let lines = [
            user(json!("First"), "u", "2026-01-28T09:00:00.000Z", false),
            assistant(
                json!([{"type": "text", "text": "Reply"}]),
                "m",
                "x",
                "2026-01-28T10:00:00.000Z",
                (1, 1),
            ),
        ];
        let s = run(&lines).unwrap();
        assert_eq!(s.started_at.to_rfc3339(), "2026-01-28T09:00:00+00:00");
        assert_eq!(s.ended_at.unwrap().to_rfc3339(), "2026-01-28T10:00:00+00:00");
    }
    #[test]
    fn parse_summary_searchable_text_indexes_conversation_content() {
        let lines = [
            u("Deploy to Digital Ocean"),
            assistant(
                json!([
                    {"type": "text", "text": "Provisioning a droplet"},
                    {"type": "thinking", "thinking": "The user wants Kubernetes"},
                    {"type": "tool_use", "id": "t1", "name": "Bash", "input": {"command": "doctl compute droplet create"}},
                ]),
                "m",
                "x",
                "2026-01-28T10:01:00.000Z",
                (1, 1),
            ),
            user(
                json!([{"type": "tool_result", "tool_use_id": "t1", "content": "droplet 12345 created"}]),
                "u2",
                "2026-01-28T10:02:00.000Z",
                false,
            ),
        ];
        let text = run(&lines).unwrap().searchable_text;
        for want in [
            "digital ocean",
            "provisioning a droplet",
            "kubernetes",
            "doctl compute droplet create",
            "droplet 12345 created",
        ] {
            assert!(text.contains(want), "{want}");
        }
        assert!(!text.contains("Digital Ocean"));
    }
    #[test]
    fn parse_summary_searchable_text_excludes_synthetic_and_sidechain() {
        let lines = [
            assistant(
                json!([{"type": "text", "text": "SECRETSYNTH"}]),
                "m",
                SYNTHETIC_MODEL,
                "2026-01-28T10:01:00.000Z",
                (1, 1),
            ),
            user(json!("SECRETSIDE"), "u1", "2026-01-28T10:00:00.000Z", true),
            user(json!("realcontent"), "u2", "2026-01-28T10:05:00.000Z", false),
        ];
        let text = run(&lines).unwrap().searchable_text;
        assert!(text.contains("realcontent") && !text.contains("secretsynth") && !text.contains("secretside"));
    }
    #[test]
    fn parse_summary_searchable_text_respects_cap() {
        let text = run(&[u(&"a".repeat(SEARCHABLE_TEXT_CAP + 5000))])
            .unwrap()
            .searchable_text;
        assert!(text.chars().count() <= SEARCHABLE_TEXT_CAP + 1 && text.contains("aaaa"));
    }

    #[test]
    fn summary_tokens_match_the_full_parse_for_every_fixture_transcript() {
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
        let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
        let mut files = Vec::new();
        for dir in [
            "windows/claude-home/projects",
            "windows/crlf/claude-home/projects",
            "synthetic",
            "screenshot",
        ] {
            walk(&fixtures.join(dir), &mut files);
        }
        let mut compared = 0;
        for f in files
            .iter()
            .filter(|f| f.components().any(|c| c.as_os_str() == "projects"))
        {
            for is_sidechain in [false, true] {
                let ctx = ParseContext { is_sidechain, ..CTX };
                let full = full::parse(f, ctx).unwrap().value;
                let summary = parse(f, ctx).unwrap().value;
                let totals = (
                    full.map(|s| (s.total_tokens, s.tokens)),
                    summary.map(|s| (s.total_tokens, s.tokens)),
                );
                if let (Some(full), Some(summary)) = totals {
                    assert_eq!(full, summary, "{} (sidechain {is_sidechain})", f.display());
                    compared += 1;
                }
            }
        }
        assert!(compared >= 20, "compared {compared}");
    }
}
