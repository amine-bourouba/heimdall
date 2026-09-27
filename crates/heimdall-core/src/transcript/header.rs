//! Header tier (TranscriptParser.parseHeader): title, preview and start time from the first 50
//! JSON lines, read in 8 KiB chunks.

use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::blocks;
use crate::error::{Loaded, Result};
use crate::jsonl;
use crate::models::message::SYNTHETIC_MODEL;
use crate::models::session::Session;

pub const HEADER_MAX_LINES: usize = 50;

pub fn parse(path: &Path, project_path: &str, session_id: &str) -> Result<Loaded<Option<Session>>> {
    let loaded = jsonl::read_head::<Value>(path, HEADER_MAX_LINES, |_| true)?;
    let mut session = from_records(&loaded.value, project_path, session_id);
    if let Some(s) = &mut session {
        s.file = Some(path.to_path_buf());
    }
    Ok(Loaded::new(session, loaded.skipped))
}

fn from_records(records: &[Value], project_path: &str, session_id: &str) -> Option<Session> {
    let mut title: Option<String> = None;
    let mut preview: Option<String> = None;
    let mut earliest: Option<DateTime<Utc>> = None;
    for record in records {
        let Some(kind @ ("user" | "assistant")) = record.get("type").and_then(Value::as_str) else {
            continue;
        };
        if record.get("isSidechain") == Some(&Value::Bool(true)) {
            continue;
        }
        let Some(message) = record.get("message") else { continue };
        if kind == "assistant" && message.get("model").and_then(Value::as_str) == Some(SYNTHETIC_MODEL) {
            continue;
        }
        if let Some(ts) = blocks::timestamp(record.get("timestamp")) {
            earliest = Some(earliest.map_or(ts, |e| e.min(ts)));
        }
        if kind == "user" && title.is_none() {
            title = first_user_text(message.get("content"));
        }
        if kind == "assistant" && preview.is_none() {
            preview = first_assistant_text(message.get("content"));
        }
        if title.is_some() && preview.is_some() && earliest.is_some() {
            break;
        }
    }
    if title.is_none() && earliest.is_none() {
        return None;
    }
    Some(Session {
        cached_title: title,
        cached_preview: preview,
        ..Session::new(
            session_id,
            project_path,
            Vec::new(),
            earliest.unwrap_or(DateTime::UNIX_EPOCH),
            None,
            0,
        )
    })
}

/// A non-empty string content, else the first text block's text (which may be missing).
pub(crate) fn first_user_text(content: Option<&Value>) -> Option<String> {
    match content? {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Array(blocks) => blocks
            .iter()
            .find(|b| b.get("type").and_then(Value::as_str) == Some("text"))?
            .get("text")?
            .as_str()
            .map(String::from),
        _ => None,
    }
}

/// The first text block that is not blank, trimmed.
pub(crate) fn first_assistant_text(content: Option<&Value>) -> Option<String> {
    let Value::Array(blocks) = content? else { return None };
    blocks
        .iter()
        .filter(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        .filter_map(|b| b.get("text")?.as_str())
        .map(str::trim)
        .find(|t| !t.is_empty())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_title_preview_and_earliest_time_skipping_sidechain_and_synthetic() {
        let records = [
            json!({"type": "summary", "summary": "x"}),
            json!({"type": "user", "isSidechain": true, "timestamp": "2026-01-01T00:00:00.000Z", "message": {"content": "side"}}),
            json!({"type": "assistant", "timestamp": "2026-01-02T00:00:00.000Z", "message": {"model": SYNTHETIC_MODEL, "content": [{"type": "text", "text": "synth"}]}}),
            json!({"type": "user", "timestamp": "2026-01-28T10:00:00.000Z", "message": {"content": [{"type": "text", "text": "Hello"}]}}),
            json!({"type": "assistant", "timestamp": "2026-01-28T09:00:00.000Z", "message": {"content": [{"type": "text", "text": "  "}, {"type": "text", "text": " Hi \n"}]}}),
        ];
        let s = from_records(&records, "/p", "s1").unwrap();
        assert_eq!(
            (s.cached_title.as_deref(), s.cached_preview.as_deref()),
            (Some("Hello"), Some("Hi"))
        );
        assert_eq!(s.started_at.to_rfc3339(), "2026-01-28T09:00:00+00:00");
        assert!(s.messages.is_empty());
    }

    #[test]
    fn nothing_useful_returns_none() {
        assert!(from_records(&[json!({"type": "progress"})], "/p", "s").is_none());
    }

    #[test]
    fn stops_reading_after_fifty_lines() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.jsonl");
        let mut lines = vec![json!({"type": "progress"}).to_string(); HEADER_MAX_LINES];
        lines.push(json!({"type": "user", "message": {"content": "late"}}).to_string());
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();
        assert!(parse(&path, "/p", "s").unwrap().value.is_none());
    }
}
