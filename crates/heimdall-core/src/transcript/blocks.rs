//! Content-block parsing shared by the tiers (TranscriptParser.swift:355-554).

use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use serde_json::Value;

use super::{system_content, tool_input};
use crate::models::message::{ContentBlock, ToolResult, ToolUse};
use crate::models::token_usage::TokenUsage;

/// A user message's content: a plain string, or text and tool-result blocks. Text blocks carry
/// their system-tag segments.
pub fn user_blocks(content: Option<&Value>) -> Vec<ContentBlock> {
    match content {
        Some(Value::String(s)) if !s.is_empty() => vec![user_text(s)],
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter_map(|b| match kind(b) {
                Some("tool_result") => Some(tool_result(b)),
                Some("text") => non_empty(b, "text").map(user_text),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// An assistant message's text, thinking and tool-use blocks; other block types are dropped.
pub fn assistant_blocks(content: Option<&Value>) -> Vec<ContentBlock> {
    let Some(Value::Array(blocks)) = content else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter_map(|b| match kind(b) {
            Some("text") => non_empty(b, "text").map(|t| ContentBlock::Text {
                id: String::new(),
                text: t.into(),
                system: None,
            }),
            Some("thinking") => non_empty(b, "thinking").map(|t| ContentBlock::Thinking {
                id: String::new(),
                text: t.into(),
            }),
            Some("tool_use") => tool_use(b),
            _ => None,
        })
        .collect()
}

/// Gives every block the id `<message_id>:<index>`, stable while the file is unchanged.
pub fn assign_ids(message_id: &str, blocks: &mut [ContentBlock]) {
    for (i, block) in blocks.iter_mut().enumerate() {
        let id = format!("{message_id}:{i}");
        match block {
            ContentBlock::Text { id: slot, .. } | ContentBlock::Thinking { id: slot, .. } => *slot = id,
            ContentBlock::ToolUse(t) => t.id = id,
            ContentBlock::ToolResult(r) => r.id = id,
        }
    }
}

/// Token counts from a message's `usage` object.
pub fn usage(v: Option<&Value>) -> Option<TokenUsage> {
    let u = v.filter(|u| u.is_object())?;
    Some(TokenUsage::with_cache(
        int_value(u.get("input_tokens")),
        int_value(u.get("output_tokens")),
        int_value(u.get("cache_read_input_tokens")),
        int_value(u.get("cache_creation_input_tokens")),
    ))
}

/// StatsComputer.intValue: a JSON number as a count; anything else, or a negative, is 0.
pub fn int_value(v: Option<&Value>) -> u64 {
    match v {
        Some(Value::Number(n)) => n
            .as_u64()
            .or_else(|| n.as_f64().filter(|f| *f > 0.0).map(|f| f as u64))
            .unwrap_or(0),
        _ => 0,
    }
}

/// An RFC 3339 timestamp such as `2026-01-28T10:00:00.000Z`.
pub fn timestamp(v: Option<&Value>) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(v?.as_str()?)
        .ok()
        .map(|t| t.with_timezone(&Utc))
}

fn user_text(text: &str) -> ContentBlock {
    ContentBlock::Text {
        id: String::new(),
        text: text.into(),
        system: system_content::segments(text),
    }
}

fn tool_use(b: &Value) -> Option<ContentBlock> {
    let tool_use_id = b.get("id")?.as_str()?;
    let name = b.get("name")?.as_str()?;
    let raw = match b.get("input") {
        Some(Value::Object(m)) => m.clone(),
        _ => serde_json::Map::new(),
    };
    let input: BTreeMap<String, String> = raw
        .iter()
        .filter_map(|(k, v)| Some((k.clone(), stringify(v)?)))
        .collect();
    Some(ContentBlock::ToolUse(ToolUse::from_input(
        "",
        tool_use_id,
        name,
        &input,
        tool_input::pretty(&Value::Object(raw)),
    )))
}

fn tool_result(b: &Value) -> ContentBlock {
    let content = match b.get("content") {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Array(blocks)) => blocks
            .iter()
            .filter(|c| kind(c) == Some("text"))
            .filter_map(|c| c.get("text")?.as_str())
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    };
    ContentBlock::ToolResult(ToolResult {
        id: String::new(),
        tool_use_id: b
            .get("tool_use_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        content,
        is_error: b.get("is_error").and_then(Value::as_bool).unwrap_or(false),
    })
}

/// TranscriptParser.stringifyValue: strings as they are, numbers and booleans as text, arrays and
/// objects as compact JSON with sorted keys; `null` is dropped.
fn stringify(v: &Value) -> Option<String> {
    match v {
        Value::Null => None,
        Value::String(s) => Some(s.clone()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Number(n) => Some(n.to_string()),
        other => Some(tool_input::compact(other)),
    }
}

fn kind(b: &Value) -> Option<&str> {
    b.get("type").and_then(Value::as_str)
}

fn non_empty<'a>(b: &'a Value, key: &str) -> Option<&'a str> {
    b.get(key).and_then(Value::as_str).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn stringifies_non_string_inputs_and_drops_null() {
        assert_eq!(
            stringify(&json!({"b": [1], "a": true})).as_deref(),
            Some(r#"{"a":true,"b":[1]}"#)
        );
        assert_eq!(
            (stringify(&json!(5000)).as_deref(), stringify(&json!(null))),
            (Some("5000"), None)
        );
    }

    #[test]
    fn ids_follow_the_message_id_and_index() {
        let mut b = assistant_blocks(Some(
            &json!([{"type": "text", "text": "a"}, {"type": "tool_use", "id": "t", "name": "Read", "input": {}}]),
        ));
        assign_ids("msg_1", &mut b);
        let ContentBlock::ToolUse(t) = &b[1] else { panic!() };
        assert_eq!((t.id.as_str(), t.tool_use_id.as_str()), ("msg_1:1", "t"));
    }

    #[test]
    fn int_value_ignores_non_numbers() {
        assert_eq!(
            [json!(7), json!(3.9), json!(-2), json!("5"), json!(true)].map(|v| int_value(Some(&v))),
            [7, 3, 0, 0, 0]
        );
    }
}
