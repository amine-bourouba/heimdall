//! Todo loader (Services/TodoLoader.swift): `~/.claude/todos/<sessionId>.json` and
//! `<sessionId>-agent-<agentId>.json`, merged per session. Files are read in name order so the
//! merge is the same on every OS.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::error::{AppError, Loaded, Result};
use crate::models::session_todo::{TodoItem, TodoList};

const AGENT_INFIX: &str = "-agent-";

/// The todos of one session, from all its files. `skipped` counts files that are not a JSON array
/// and items that do not parse.
pub fn load_for_session(todos_dir: &Path, session_id: &str) -> Result<Loaded<Vec<TodoItem>>> {
    let agent_prefix = format!("{session_id}{AGENT_INFIX}");
    let mut todos = Vec::new();
    let mut skipped = 0;
    for (stem, path) in json_files(todos_dir)? {
        if stem == session_id || stem.starts_with(&agent_prefix) {
            let loaded = read_file(&path);
            skipped += loaded.skipped;
            todos.extend(loaded.value);
        }
    }
    Ok(Loaded::new(todos, skipped))
}

/// Whether the session has any todo file at all (plain or agent-suffixed) — cheap existence check
/// for a sidebar/list badge, without reading a single one.
pub fn has_any(todos_dir: &Path, session_id: &str) -> bool {
    let agent_prefix = format!("{session_id}{AGENT_INFIX}");
    json_files(todos_dir).is_ok_and(|files| {
        files
            .iter()
            .any(|(stem, _)| stem == session_id || stem.starts_with(&agent_prefix))
    })
}

/// Every session's todos, keyed by session id; sessions whose files are all empty are left out.
pub fn load_all(todos_dir: &Path) -> Result<Loaded<BTreeMap<String, Vec<TodoItem>>>> {
    let mut grouped: BTreeMap<String, Vec<TodoItem>> = BTreeMap::new();
    let mut skipped = 0;
    for (stem, path) in json_files(todos_dir)? {
        let loaded = read_file(&path);
        skipped += loaded.skipped;
        if !loaded.value.is_empty() {
            grouped
                .entry(session_id_of(&stem).to_string())
                .or_default()
                .extend(loaded.value);
        }
    }
    Ok(Loaded::new(grouped, skipped))
}

/// The session's todos as one [`TodoList`] per file — the plain file (`agent_id: None`) and each
/// `-agent-<id>` file kept apart, unlike [`load_for_session`], which merges them into one list
/// (contract `TodoList`, `get_session_todos`).
pub fn load_lists_for_session(todos_dir: &Path, session_id: &str) -> Result<Loaded<Vec<TodoList>>> {
    let agent_prefix = format!("{session_id}{AGENT_INFIX}");
    let mut lists = Vec::new();
    let mut skipped = 0;
    for (stem, path) in json_files(todos_dir)? {
        let agent_id = if stem == session_id {
            None
        } else if let Some(id) = stem.strip_prefix(&agent_prefix) {
            Some(id.to_string())
        } else {
            continue;
        };
        let loaded = read_file(&path);
        skipped += loaded.skipped;
        if !loaded.value.is_empty() {
            lists.push(TodoList {
                session_id: session_id.to_string(),
                agent_id,
                items: loaded.value,
            });
        }
    }
    Ok(Loaded::new(lists, skipped))
}

/// Every session with at least one todo file, each as [`load_lists_for_session`] would produce it
/// (contract `TodoGroup.lists`, `list_all_todos`).
pub fn load_all_lists(todos_dir: &Path) -> Result<Loaded<BTreeMap<String, Vec<TodoList>>>> {
    let ids: BTreeSet<String> = json_files(todos_dir)?
        .into_iter()
        .map(|(stem, _)| session_id_of(&stem).to_string())
        .collect();
    let mut grouped = BTreeMap::new();
    let mut skipped = 0;
    for id in ids {
        let loaded = load_lists_for_session(todos_dir, &id)?;
        skipped += loaded.skipped;
        if !loaded.value.is_empty() {
            grouped.insert(id, loaded.value);
        }
    }
    Ok(Loaded::new(grouped, skipped))
}

/// `<sessionId>-agent-<agentId>` → `<sessionId>`.
pub fn session_id_of(stem: &str) -> &str {
    stem.find(AGENT_INFIX).map_or(stem, |i| &stem[..i])
}

/// `(stem, path)` of each visible `.json` file, sorted by name. A missing folder has none.
fn json_files(dir: &Path) -> Result<Vec<(String, PathBuf)>> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(AppError::from_io(&e, dir)),
    };
    let mut files: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| Some((p.file_stem()?.to_str()?.to_string(), p)))
        .filter(|(stem, _)| !stem.starts_with('.'))
        .collect();
    files.sort();
    Ok(files)
}

fn read_file(path: &Path) -> Loaded<Vec<TodoItem>> {
    let parsed = std::fs::read_to_string(path)
        .ok()
        .and_then(|text| TodoItem::list_from_json(&text).ok());
    parsed.unwrap_or_else(|| Loaded::new(Vec::new(), 1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::session_todo::TodoStatus;

    fn dir_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            std::fs::write(dir.path().join(name), content).unwrap();
        }
        dir
    }
    fn todo(content: &str, status: &str) -> String {
        format!(r#"[{{"content": "{content}", "status": "{status}", "activeForm": "Doing {content}"}}]"#)
    }
    fn contents(todos: &[TodoItem]) -> Vec<&str> {
        todos.iter().map(|t| t.content.as_str()).collect()
    }
    const MISSING: &str = "/nonexistent/path/5d1c0f2a";

    // Ported from PoirotTests/Services/TodoLoaderTests.swift.
    #[test]
    fn load_todos_non_existent_path_returns_empty() {
        assert!(
            load_for_session(Path::new(MISSING), "some-session-id")
                .unwrap()
                .value
                .is_empty()
        );
    }
    #[test]
    fn load_todos_empty_directory_returns_empty() {
        let d = dir_with(&[]);
        assert!(load_for_session(d.path(), "some-session-id").unwrap().value.is_empty());
    }
    #[test]
    fn load_todos_matches_agent_file() {
        let d = dir_with(&[("abc-1234-agent-def-5678.json", &todo("Build feature", "in_progress"))]);
        let t = load_for_session(d.path(), "abc-1234").unwrap().value;
        assert_eq!(
            (t.len(), t[0].content.as_str(), t[0].status),
            (1, "Build feature", TodoStatus::InProgress)
        );
    }
    #[test]
    fn load_todos_matches_exact_session_id_file() {
        let d = dir_with(&[("abc-1234.json", &todo("Deploy", "pending"))]);
        assert_eq!(
            contents(&load_for_session(d.path(), "abc-1234").unwrap().value),
            ["Deploy"]
        );
    }
    #[test]
    fn load_todos_merges_multiple_agent_files() {
        let d = dir_with(&[
            ("abc-1234-agent-agent1.json", &todo("Task A", "completed")),
            ("abc-1234-agent-agent2.json", &todo("Task B", "pending")),
        ]);
        assert_eq!(
            contents(&load_for_session(d.path(), "abc-1234").unwrap().value),
            ["Task A", "Task B"]
        );
    }
    #[test]
    fn load_todos_does_not_match_unrelated_files() {
        let d = dir_with(&[("other-session-agent-xyz.json", &todo("Unrelated", "pending"))]);
        assert!(load_for_session(d.path(), "abc-1234").unwrap().value.is_empty());
    }
    #[test]
    fn load_todos_handles_empty_array_file() {
        let d = dir_with(&[("abc-1234-agent-def.json", "[]")]);
        let loaded = load_for_session(d.path(), "abc-1234").unwrap();
        assert!(loaded.value.is_empty() && loaded.skipped == 0);
    }
    #[test]
    fn load_todos_skips_invalid_json() {
        let d = dir_with(&[
            ("abc-1234-agent-bad.json", "not valid json"),
            ("abc-1234-agent-good.json", &todo("Valid", "completed")),
        ]);
        let loaded = load_for_session(d.path(), "abc-1234").unwrap();
        assert_eq!((contents(&loaded.value), loaded.skipped), (vec!["Valid"], 1));
    }
    #[test]
    fn load_todos_uuid_session_id_matches_agent_file() {
        let json = r#"[{"content": "Create diagnostic script", "status": "in_progress", "activeForm": "Creating script"},
                       {"content": "Check collection count", "status": "pending", "activeForm": "Checking count"}]"#;
        let d = dir_with(&[(
            "00163b58-3b30-49fa-af28-052f06724f50-agent-f57e3842-efed-43d9-a4c9-1e644a82141e.json",
            json,
        )]);
        let t = load_for_session(d.path(), "00163b58-3b30-49fa-af28-052f06724f50")
            .unwrap()
            .value;
        assert_eq!(
            (t.len(), t[0].content.as_str(), t[1].status),
            (2, "Create diagnostic script", TodoStatus::Pending)
        );
    }
    #[test]
    fn load_all_todos_non_existent_path_returns_empty() {
        assert!(load_all(Path::new(MISSING)).unwrap().value.is_empty());
    }
    #[test]
    fn load_all_todos_groups_by_session_id() {
        let d = dir_with(&[
            ("session-1.json", &todo("Task A", "completed")),
            ("session-2.json", &todo("Task B", "pending")),
        ]);
        let r = load_all(d.path()).unwrap().value;
        assert_eq!((r.len(), r["session-1"].len(), r["session-2"].len()), (2, 1, 1));
    }
    #[test]
    fn load_all_todos_merges_agent_files_under_same_session() {
        let d = dir_with(&[
            ("session-1-agent-a1.json", &todo("Task A", "completed")),
            ("session-1-agent-a2.json", &todo("Task B", "pending")),
        ]);
        let r = load_all(d.path()).unwrap().value;
        assert_eq!((r.len(), r["session-1"].len()), (1, 2));
    }
    #[test]
    fn load_all_todos_skips_empty_array_files() {
        let d = dir_with(&[
            ("session-empty.json", "[]"),
            ("session-real.json", &todo("Real task", "pending")),
        ]);
        let r = load_all(d.path()).unwrap().value;
        assert_eq!(
            (r.len(), r.contains_key("session-empty"), r["session-real"].len()),
            (1, false, 1)
        );
    }

    #[test]
    fn an_unknown_status_drops_only_that_item() {
        let json = r#"[{"content": "ok", "status": "pending", "activeForm": "a"}, {"content": "new", "status": "blocked", "activeForm": "b"}]"#;
        let d = dir_with(&[("s.json", json)]);
        let loaded = load_for_session(d.path(), "s").unwrap();
        assert_eq!((contents(&loaded.value), loaded.skipped), (vec!["ok"], 1));
    }

    #[test]
    fn load_lists_for_session_keeps_the_plain_file_and_each_agent_file_apart() {
        let d = dir_with(&[
            ("abc-1234.json", &todo("Main task", "pending")),
            ("abc-1234-agent-a1.json", &todo("Agent task", "completed")),
        ]);
        let lists = load_lists_for_session(d.path(), "abc-1234").unwrap().value;
        assert_eq!(lists.len(), 2);
        let main = lists.iter().find(|l| l.agent_id.is_none()).unwrap();
        let agent = lists.iter().find(|l| l.agent_id.is_some()).unwrap();
        assert_eq!(
            (contents(&main.items), main.session_id.as_str()),
            (vec!["Main task"], "abc-1234")
        );
        assert_eq!(
            (contents(&agent.items), agent.agent_id.as_deref()),
            (vec!["Agent task"], Some("a1"))
        );
    }

    #[test]
    fn load_lists_for_session_drops_an_empty_file_but_keeps_a_real_one() {
        let d = dir_with(&[
            ("abc-1234.json", "[]"),
            ("abc-1234-agent-a1.json", &todo("Task", "pending")),
        ]);
        let lists = load_lists_for_session(d.path(), "abc-1234").unwrap().value;
        assert_eq!(lists.len(), 1);
        assert_eq!(lists[0].agent_id.as_deref(), Some("a1"));
    }

    #[test]
    fn load_all_lists_groups_by_parent_session_and_keeps_agents_apart() {
        let d = dir_with(&[
            ("s1.json", &todo("A", "pending")),
            ("s1-agent-x.json", &todo("B", "completed")),
            ("s2.json", &todo("C", "pending")),
        ]);
        let grouped = load_all_lists(d.path()).unwrap().value;
        assert_eq!(grouped.keys().collect::<Vec<_>>(), ["s1", "s2"]);
        assert_eq!(grouped["s1"].len(), 2);
        assert_eq!(grouped["s2"].len(), 1);
    }

    #[test]
    fn load_all_lists_non_existent_path_returns_empty() {
        assert!(load_all_lists(Path::new(MISSING)).unwrap().value.is_empty());
    }
}
