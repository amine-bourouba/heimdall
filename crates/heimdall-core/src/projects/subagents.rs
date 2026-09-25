//! Sub-agent transcripts: `<project>/<sessionId>/subagents/agent-<id>.jsonl` with an optional
//! `agent-<id>.meta.json` (SessionLoader.loadSubAgentSessions). Compaction agents
//! (`agent-acompact-*`) are internal and left out.

use std::path::Path;

use serde_json::Value;

use crate::models::session::Session;
use crate::transcript::full::ParseContext;
use crate::transcript::summary;

pub const AGENT_PREFIX: &str = "agent-";

/// Summaries of the sub-agents of `parent_id`, and the number of transcript lines that did not parse.
pub fn load(project_dir: &Path, parent_id: &str) -> (Vec<Session>, u32) {
    let dir = project_dir.join(parent_id).join("subagents");
    let Ok(entries) = std::fs::read_dir(&dir) else { return (Vec::new(), 0) };
    let mut sessions = Vec::new();
    let mut skipped = 0;
    for path in entries.flatten().map(|e| e.path()) {
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let is_agent = path.extension().is_some_and(|e| e == "jsonl")
            && stem.starts_with(AGENT_PREFIX)
            && !stem.starts_with("agent-acompact-");
        if !is_agent {
            continue;
        }
        let ctx = ParseContext { project_path: "", session_id: stem, index_started_at: None, is_sidechain: true };
        let Ok(loaded) = summary::parse(&path, ctx) else {
            skipped += 1;
            continue;
        };
        skipped += loaded.skipped;
        let Some(mut session) = loaded.value else { continue };
        let meta = std::fs::read(dir.join(format!("{stem}.meta.json"))).ok().and_then(|b| serde_json::from_slice::<Value>(&b).ok());
        let meta_text = |k: &str| meta.as_ref().and_then(|m| m.get(k)).and_then(Value::as_str).map(String::from);
        session.agent_id = Some(stem[AGENT_PREFIX.len()..].to_string());
        session.parent_session_id = Some(parent_id.to_string());
        session.agent_type = meta_text("agentType");
        session.agent_description = meta_text("description");
        sessions.push(session);
    }
    (sessions, skipped)
}
