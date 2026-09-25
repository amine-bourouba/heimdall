//! `sessions-index.json`, the fast path older Claude Code versions write in each project folder
//! (SessionLoader.readSessionsIndex). Current versions do not write it (fixtures/windows MANIFEST fact 5).

use std::path::Path;

use serde_json::Value;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SessionsIndex {
    /// The project path: the first entry's `projectPath`, else the top-level `projectPath` or
    /// `originalPath` (two Poirot readers used different keys; inventory D-1).
    pub project_path: Option<String>,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub session_id: String,
    pub created: String,
    pub is_sidechain: bool,
    pub project_path: Option<String>,
    pub first_prompt: Option<String>,
    pub agent_id: Option<String>,
}

/// Reads `<project_dir>/sessions-index.json`; `None` when it is missing or not a JSON object.
/// Entries without `sessionId` or `created` are dropped.
pub fn read(project_dir: &Path) -> Option<SessionsIndex> {
    let bytes = std::fs::read(project_dir.join("sessions-index.json")).ok()?;
    parse(&serde_json::from_slice(&bytes).ok()?)
}

fn parse(json: &Value) -> Option<SessionsIndex> {
    let json = json.as_object()?;
    let text = |v: &Value, k: &str| v.get(k).and_then(Value::as_str).map(String::from);
    let entries: Vec<Entry> = json
        .get("entries")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|e| {
            Some(Entry {
                session_id: text(e, "sessionId")?,
                created: text(e, "created")?,
                is_sidechain: e.get("isSidechain").and_then(Value::as_bool).unwrap_or(false),
                project_path: text(e, "projectPath"),
                first_prompt: text(e, "firstPrompt"),
                agent_id: text(e, "agentId"),
            })
        })
        .collect();
    let top = |k: &str| json.get(k).and_then(Value::as_str).map(String::from);
    let project_path = entries.first().and_then(|e| e.project_path.clone()).or_else(|| top("projectPath")).or_else(|| top("originalPath"));
    Some(SessionsIndex { project_path, entries })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_entries_and_the_project_path_from_any_known_key() {
        let idx = parse(&json!({"entries": [{"sessionId": "a", "created": "2026-01-01T00:00:00.000Z", "projectPath": "/p", "agentId": "x"}, {"sessionId": "no-created"}]})).unwrap();
        assert_eq!((idx.entries.len(), idx.project_path.as_deref(), idx.entries[0].agent_id.as_deref()), (1, Some("/p"), Some("x")));
        assert_eq!(parse(&json!({"projectPath": "/q"})).unwrap().project_path.as_deref(), Some("/q"));
        assert_eq!(parse(&json!({"originalPath": "/r"})).unwrap().project_path.as_deref(), Some("/r"));
        assert!(parse(&json!([1])).is_none());
    }
}
