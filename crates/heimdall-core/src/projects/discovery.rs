//! Project and session discovery (Services/SessionLoader.swift): every folder under
//! `~/.claude/projects/` is a project; its sessions come from `sessions-index.json` when present,
//! else from a scan of its transcripts, plus each main session's sub-agents.

use std::collections::HashMap;
use std::path::Path;

use chrono::DateTime;

use super::{project_path, sessions_index, subagents};
use crate::error::{AppError, Loaded, Result};
use crate::models::project::Project;
use crate::paths::{self, TargetOs};
use crate::transcript::full::ParseContext;
use crate::transcript::summary;

/// All projects, sorted by folder name. `skipped` counts transcript lines that did not parse and
/// transcripts that could not be read.
pub fn discover(projects_dir: &Path, os: TargetOs) -> Result<Loaded<Vec<Project>>> {
    discover_with(projects_dir, os, &|p: &str| Path::new(p).is_dir())
}

/// [`discover`] with the folder probe used by the last-resort path decoding injected.
pub fn discover_with(projects_dir: &Path, os: TargetOs, exists: &dyn Fn(&str) -> bool) -> Result<Loaded<Vec<Project>>> {
    let entries = match std::fs::read_dir(projects_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::new(Vec::new(), 0)),
        Err(e) => return Err(AppError::from_io(&e, projects_dir)),
    };
    let mut dirs: Vec<_> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| {
            p.is_dir()
                && !p
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with('.'))
        })
        .collect();
    dirs.sort();
    let mut skipped = 0;
    let projects = dirs
        .iter()
        .map(|dir| {
            let loaded = load_project_with(dir, os, exists);
            skipped += loaded.skipped;
            loaded.value
        })
        .collect();
    Ok(Loaded::new(projects, skipped))
}

/// One project folder (SessionLoader.loadProject), for incremental loading.
pub fn load_project(dir: &Path, os: TargetOs) -> Loaded<Project> {
    load_project_with(dir, os, &|p: &str| Path::new(p).is_dir())
}

fn load_project_with(dir: &Path, os: TargetOs, exists: &dyn Fn(&str) -> bool) -> Loaded<Project> {
    let dir_name = dir.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_string();
    let index = sessions_index::read(dir);
    let mut skipped = 0;
    let mut sessions = Vec::new();
    let mut summarize = |path: &Path, session_id: &str, index_started_at, is_sidechain| {
        let ctx = ParseContext {
            project_path: "",
            session_id,
            index_started_at,
            is_sidechain,
        };
        match summary::parse(path, ctx) {
            Ok(loaded) => {
                skipped += loaded.skipped;
                loaded.value
            }
            Err(_) => {
                skipped += 1;
                None
            }
        }
    };

    let main_ids: Vec<String> = match index.as_ref().filter(|i| !i.entries.is_empty()) {
        // Fast path: the index names the sessions; entries whose file is gone are skipped.
        Some(index) => {
            for entry in &index.entries {
                let path = dir.join(format!("{}.jsonl", entry.session_id));
                if !path.is_file() {
                    continue;
                }
                let started = DateTime::parse_from_rfc3339(&entry.created).ok().map(|t| t.to_utc());
                if let Some(mut s) = summarize(&path, &entry.session_id, started, entry.is_sidechain) {
                    s.first_prompt = entry.first_prompt.clone();
                    s.agent_id = entry.agent_id.clone();
                    sessions.push(s);
                }
            }
            sessions
                .iter()
                .filter(|s| !s.is_sidechain)
                .map(|s| s.id.clone())
                .collect()
        }
        // Fallback: `<uuid>.jsonl` main sessions and older top-level `agent-<id>.jsonl` files. Other
        // stems, such as `<uuid>.orphaned-<ms>-<hex>`, are ignored.
        None => {
            let mut main_ids = Vec::new();
            for path in std::fs::read_dir(dir).into_iter().flatten().flatten().map(|e| e.path()) {
                let Some(stem) = path.file_stem().and_then(|s| s.to_str()).map(String::from) else {
                    continue;
                };
                if path.extension().is_none_or(|e| e != "jsonl") {
                    continue;
                }
                if is_uuid(&stem) {
                    if let Some(s) = summarize(&path, &stem, None, false) {
                        sessions.push(s);
                    }
                    main_ids.push(stem);
                } else if let Some(agent_id) = stem.strip_prefix(subagents::AGENT_PREFIX)
                    && let Some(mut s) = summarize(&path, &stem, None, true)
                {
                    s.agent_id = Some(agent_id.to_string());
                    sessions.push(s);
                }
            }
            main_ids
        }
    };
    for parent in &main_ids {
        let (agents, agent_skipped) = subagents::load(dir, parent);
        skipped += agent_skipped;
        sessions.extend(agents);
    }

    let mut cwds: HashMap<String, u32> = HashMap::new();
    for (cwd, n) in sessions.iter().flat_map(|s| &s.cwds) {
        *cwds.entry(cwd.clone()).or_default() += n;
    }
    let (path, source) = project_path::resolve(
        &dir_name,
        &cwds,
        index.as_ref().and_then(|i| i.project_path.as_deref()),
        os,
        exists,
    );
    for s in &mut sessions {
        s.project_path.clone_from(&path);
    }
    sessions.sort_by_key(|s| std::cmp::Reverse(s.started_at));

    let project = Project {
        id: dir_name,
        name: paths::last_component(&path).to_string(),
        path: Some(path),
        path_source: Some(source),
        sessions,
    };
    Loaded::new(project, skipped)
}

/// A lower-case hyphenated UUID, as Claude Code names session files.
pub fn is_uuid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_digit() || (b'a'..=b'f').contains(c),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::project::PathSource;
    use serde_json::json;
    use std::fs;
    use std::path::PathBuf;

    const SESSION: &str = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";

    fn user(content: &str, timestamp: &str) -> String {
        json!({"type": "user", "uuid": "u", "timestamp": timestamp, "isSidechain": false, "message": {"role": "user", "content": content}}).to_string()
    }
    fn assistant(text: &str, timestamp: &str) -> String {
        json!({"type": "assistant", "uuid": "a", "timestamp": timestamp, "isSidechain": false,
               "message": {"id": "msg_test", "role": "assistant", "model": "claude-opus-4-6", "content": [{"type": "text", "text": text}],
                           "usage": {"input_tokens": 100, "output_tokens": 50}}}).to_string()
    }
    fn simple_jsonl(user_text: &str, user_ts: &str) -> String {
        [
            user(user_text, user_ts),
            assistant("Hi there", "2026-01-28T10:01:00.000Z"),
        ]
        .join("\n")
    }
    fn simple() -> String {
        simple_jsonl("Hello", "2026-01-28T10:00:00.000Z")
    }
    fn project_with(base: &Path, dir: &str, session_id: &str, jsonl: &str, index: Option<&str>) -> PathBuf {
        let d = base.join(dir);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join(format!("{session_id}.jsonl")), jsonl).unwrap();
        if let Some(index) = index {
            fs::write(d.join("sessions-index.json"), index).unwrap();
        }
        d
    }
    /// Pretends only `/Users` exists, like a Mac without the test's folders.
    fn run(base: &Path) -> Vec<Project> {
        discover_with(base, TargetOs::Macos, &|p: &str| p == "/Users")
            .unwrap()
            .value
    }
    fn index_json(entries: serde_json::Value) -> String {
        json!({"version": 1, "entries": entries}).to_string()
    }

    // Ported from PoirotTests/Services/SessionLoaderTests.swift.
    #[test]
    fn discover_projects_non_existent_path_returns_empty() {
        assert!(run(Path::new("/nonexistent/path/that/does/not/exist")).is_empty());
    }
    #[test]
    fn discover_projects_empty_directory_returns_empty() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(run(tmp.path()).is_empty());
    }
    #[test]
    fn discover_projects_with_subdirectories_returns_projects() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("project-alpha")).unwrap();
        fs::create_dir(tmp.path().join("project-beta")).unwrap();
        let names: Vec<_> = run(tmp.path()).into_iter().map(|p| p.name).collect();
        assert_eq!(names, ["project-alpha", "project-beta"]);
    }
    #[test]
    fn discover_projects_skips_files_only_includes_directories() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir(tmp.path().join("real-project")).unwrap();
        fs::write(tmp.path().join("not-a-project.txt"), "hello").unwrap();
        let projects = run(tmp.path());
        assert_eq!((projects.len(), projects[0].name.as_str()), (1, "real-project"));
    }
    #[test]
    fn discover_projects_with_jsonl_files_parses_sessions() {
        let tmp = tempfile::tempdir().unwrap();
        project_with(tmp.path(), "test-project", SESSION, &simple(), None);
        let projects = run(tmp.path());
        let s = &projects[0].sessions[0];
        assert_eq!(
            (projects.len(), projects[0].sessions.len(), s.id.as_str()),
            (1, 1, SESSION)
        );
        assert!(s.messages.is_empty() && s.file.is_some());
        assert_eq!(s.title(), "Hello");
    }
    #[test]
    fn discover_projects_fallback_path_loads_agent_files() {
        let tmp = tempfile::tempdir().unwrap();
        let d = project_with(tmp.path(), "test-project", SESSION, &simple(), None);
        fs::write(d.join("agent-e7b2889f.jsonl"), simple()).unwrap();
        let p = &run(tmp.path())[0];
        assert_eq!(p.sessions.len(), 2);
        let main = p.sessions.iter().find(|s| s.id == SESSION).unwrap();
        assert!(!main.is_sidechain && main.agent_id.is_none());
        let agent = p.sessions.iter().find(|s| s.id == "agent-e7b2889f").unwrap();
        assert!(agent.is_sidechain);
        assert_eq!(agent.agent_id.as_deref(), Some("e7b2889f"));
    }
    #[test]
    fn discover_projects_sessions_sorted_descending() {
        let tmp = tempfile::tempdir().unwrap();
        let old = "00000000-0000-0000-0000-000000000001";
        let new = "00000000-0000-0000-0000-000000000002";
        let d = project_with(
            tmp.path(),
            "test-project",
            old,
            &simple_jsonl("Hello", "2026-01-01T10:00:00.000Z"),
            None,
        );
        fs::write(
            d.join(format!("{new}.jsonl")),
            simple_jsonl("Hello", "2026-02-01T10:00:00.000Z"),
        )
        .unwrap();
        let ids: Vec<_> = run(tmp.path())[0].sessions.iter().map(|s| s.id.clone()).collect();
        assert_eq!(ids, [new, old]);
    }
    #[test]
    fn discover_projects_uses_original_path_from_index() {
        let tmp = tempfile::tempdir().unwrap();
        let index = index_json(
            json!([{"sessionId": SESSION, "created": "2026-01-28T10:00:00.000Z", "isSidechain": false,
                                       "projectPath": "/Users/dev/projects/my-cool-app", "firstPrompt": "Hello from index"}]),
        );
        project_with(
            tmp.path(),
            "-Users-dev-projects-my-cool-app",
            SESSION,
            &simple(),
            Some(&index),
        );
        let p = &run(tmp.path())[0];
        assert_eq!(
            (p.name.as_str(), p.path.as_deref(), p.path_source),
            (
                "my-cool-app",
                Some("/Users/dev/projects/my-cool-app"),
                Some(PathSource::SessionsIndex)
            )
        );
        let s = &p.sessions[0];
        assert!(s.messages.is_empty() && s.file.is_some());
        assert_eq!(s.title(), "Hello");
    }
    #[test]
    fn discover_projects_with_missing_index_uses_fallback() {
        let tmp = tempfile::tempdir().unwrap();
        project_with(tmp.path(), "-Users-dev-projects-my-app", SESSION, &simple(), None);
        let p = &run(tmp.path())[0];
        assert_eq!(
            (p.name.as_str(), p.path_source),
            ("dev-projects-my-app", Some(PathSource::Decoded))
        );
    }
    #[test]
    fn discover_projects_empty_jsonl_file_produces_no_session() {
        let tmp = tempfile::tempdir().unwrap();
        project_with(tmp.path(), "test-project", SESSION, "", None);
        assert!(run(tmp.path())[0].sessions.is_empty());
    }
    #[test]
    fn discover_projects_index_path_includes_sidechain_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let side = "b2c3d4e5-f6a7-8901-bcde-f12345678901";
        let index = index_json(json!([
            {"sessionId": SESSION, "created": "2026-01-28T10:00:00.000Z", "isSidechain": false, "projectPath": "/test/project", "firstPrompt": "Main session"},
            {"sessionId": side, "created": "2026-01-28T11:00:00.000Z", "isSidechain": true, "agentId": "e7b2889f", "projectPath": "/test/project"},
        ]));
        let d = project_with(tmp.path(), "test-project", SESSION, &simple(), Some(&index));
        fs::write(d.join(format!("{side}.jsonl")), simple()).unwrap();
        let p = &run(tmp.path())[0];
        assert_eq!(p.sessions.len(), 2);
        let s = p.sessions.iter().find(|s| s.id == side).unwrap();
        assert!(s.is_sidechain);
        assert_eq!(s.agent_id.as_deref(), Some("e7b2889f"));
        let m = p.sessions.iter().find(|s| s.id == SESSION).unwrap();
        assert!(!m.is_sidechain && m.agent_id.is_none());
    }
    #[test]
    fn discover_projects_index_path_loads_all_sessions() {
        let tmp = tempfile::tempdir().unwrap();
        let d = tmp.path().join("test-project");
        fs::create_dir(&d).unwrap();
        let mut entries = Vec::new();
        for i in 0..25 {
            let id = format!("00000000-0000-0000-0000-{i:012}");
            let created = format!("2026-01-{:02}T10:00:00.000Z", i + 1);
            fs::write(d.join(format!("{id}.jsonl")), simple_jsonl("Hello", &created)).unwrap();
            entries.push(
                json!({"sessionId": id, "created": created, "isSidechain": false, "projectPath": "/test/project"}),
            );
        }
        fs::write(d.join("sessions-index.json"), index_json(json!(entries))).unwrap();
        assert_eq!(run(tmp.path())[0].sessions.len(), 25);
    }
    #[test]
    fn discover_projects_fallback_path_uses_header_parse() {
        let tmp = tempfile::tempdir().unwrap();
        project_with(
            tmp.path(),
            "test-project",
            SESSION,
            &simple_jsonl("Fallback title", "2026-01-28T10:00:00.000Z"),
            None,
        );
        let s = &run(tmp.path())[0].sessions[0];
        assert!(s.messages.is_empty() && s.file.is_some());
        assert_eq!(s.title(), "Fallback title");
    }
    #[test]
    fn discover_projects_index_path_passes_first_prompt_to_session() {
        let tmp = tempfile::tempdir().unwrap();
        let index = index_json(
            json!([{"sessionId": SESSION, "created": "2026-01-28T10:00:00.000Z", "isSidechain": false, "projectPath": "/test/project", "firstPrompt": "Fix the login bug"}]),
        );
        project_with(tmp.path(), "test-project", SESSION, &simple(), Some(&index));
        assert_eq!(
            run(tmp.path())[0].sessions[0].first_prompt.as_deref(),
            Some("Fix the login bug")
        );
    }
    #[test]
    fn discover_projects_index_path_nil_first_prompt_when_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let index = index_json(
            json!([{"sessionId": SESSION, "created": "2026-01-28T10:00:00.000Z", "isSidechain": false, "projectPath": "/test/project"}]),
        );
        project_with(tmp.path(), "test-project", SESSION, &simple(), Some(&index));
        assert!(run(tmp.path())[0].sessions[0].first_prompt.is_none());
    }
    #[test]
    fn discover_projects_fallback_path_first_prompt_is_nil() {
        let tmp = tempfile::tempdir().unwrap();
        project_with(tmp.path(), "test-project", SESSION, &simple(), None);
        assert!(run(tmp.path())[0].sessions[0].first_prompt.is_none());
    }

    #[test]
    fn index_entries_whose_file_is_missing_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let gone = "00000000-0000-0000-0000-00000000dead";
        let index = index_json(
            json!([{"sessionId": SESSION, "created": "2026-01-28T10:00:00.000Z"}, {"sessionId": gone, "created": "2026-01-29T10:00:00.000Z"}]),
        );
        project_with(tmp.path(), "p", SESSION, &simple(), Some(&index));
        assert_eq!(run(tmp.path())[0].sessions.len(), 1);
    }

    #[test]
    fn windows_fixture_projects_resolve_from_transcript_cwd() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows/claude-home/projects");
        let loaded = discover_with(&root, TargetOs::Windows, &|_| false).unwrap();
        assert_eq!(loaded.skipped, 0);
        let by_id: HashMap<_, _> = loaded.value.iter().map(|p| (p.id.as_str(), p)).collect();
        let p = |id: &str| by_id[id];

        // Every project with transcripts resolves from cwd to a drive-letter path.
        for project in loaded.value.iter().filter(|p| !p.sessions.is_empty()) {
            let path = project.path.as_deref().unwrap();
            assert_eq!(project.path_source, Some(PathSource::TranscriptCwd), "{}", project.id);
            assert!(
                path.as_bytes()[1] == b':' && paths::encodes_to(path, &project.id),
                "{} -> {path}",
                project.id
            );
            assert!(project.sessions.iter().all(|s| s.project_path == path));
        }
        // The tech folder's sessions mostly ran in a subfolder; the project stays the folder itself.
        assert_eq!(
            p("C--Users-fixture-Desktop-tech").path.as_deref(),
            Some(r"C:\Users\fixture\Desktop\tech")
        );
        // Folders differing only in drive-letter case stay separate projects.
        assert!(
            by_id.contains_key("c--Users-fixture-Desktop-tech-proj-11")
                && by_id.contains_key("C--Users-fixture-Desktop-tech")
        );
        // Orphaned transcripts are not sessions.
        assert!(
            p("C--Users-fixture-Desktop-tech")
                .sessions
                .iter()
                .all(|s| !s.id.contains("orphaned"))
        );
        // Sub-agents come from <session>/subagents with their meta.
        let agents: Vec<_> = p("c--Users-fixture-Desktop-tech-proj-11")
            .sessions
            .iter()
            .filter(|s| s.parent_session_id.is_some())
            .collect();
        assert!(!agents.is_empty() && agents.iter().all(|s| s.is_sidechain && s.agent_type.is_some()));
        // A folder with only memory files has no cwd to go on.
        assert_eq!(
            p("c--Users-fixture-Desktop-tech-proj-2").path_source,
            Some(PathSource::Decoded)
        );
    }
}
