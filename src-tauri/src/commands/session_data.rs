//! Commands in commands/session_data.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! real bodies land here (M1-42). Every body runs through `state::blocking` (contract §1).

use heimdall_core::data::{debug_log, facets, file_history, todos};
use heimdall_core::diff::{self, DiffResult};
use heimdall_core::models::debug_log_entry::LogLevel;
use heimdall_core::models::file_version::{file_history as to_file_history, FileHistory, FileVersionContent};
use heimdall_core::models::session_facets::SessionFacets;
use heimdall_core::models::session_todo::{TodoGroup, TodoList};
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::discover;
use heimdall_core::{AppError, Result};
use tauri::{AppHandle, Manager};

use super::sessions::project_dir;
use crate::state::{blocking, AppState};

fn parse_level(level: Option<&str>) -> Result<Option<LogLevel>> {
    match level {
        None => Ok(None),
        Some("debug") => Ok(Some(LogLevel::Debug)),
        Some("warn") => Ok(Some(LogLevel::Warn)),
        Some("error") => Ok(Some(LogLevel::Error)),
        Some(other) => Err(AppError::invalid_input(format!("not a debug log level: {other}"))),
    }
}

fn get_session_todos_inner(state: &AppState, session_id: &str) -> Result<Vec<TodoList>> {
    Ok(todos::load_lists_for_session(&state.claude_home.todos_dir(), session_id)?.value)
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_todos(app: AppHandle, session_id: String) -> Result<Vec<TodoList>> {
    blocking(move || get_session_todos_inner(&app.state::<AppState>(), &session_id)).await
}

fn list_all_todos_inner(state: &AppState) -> Result<Vec<TodoGroup>> {
    let claude_home = &state.claude_home;
    let projects = discover(&claude_home.projects_dir(), TargetOs::current())?.value;
    let mut project_of: std::collections::HashMap<&str, (&str, &str)> = std::collections::HashMap::new();
    for p in &projects {
        for s in &p.sessions {
            project_of.insert(s.id.as_str(), (p.id.as_str(), p.name.as_str()));
        }
    }

    let all = todos::load_all_lists(&claude_home.todos_dir())?.value;
    let mut groups: std::collections::BTreeMap<Option<(String, String)>, Vec<TodoList>> = std::collections::BTreeMap::new();
    for (session_id, lists) in all {
        let key = project_of.get(session_id.as_str()).map(|(id, name)| (id.to_string(), name.to_string()));
        groups.entry(key).or_default().extend(lists);
    }
    Ok(groups
        .into_iter()
        .map(|(key, lists)| {
            let (project_id, project_name) = key.map_or((None, None), |(id, name)| (Some(id), Some(name)));
            TodoGroup { project_id, project_name, lists }
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn list_all_todos(app: AppHandle) -> Result<Vec<TodoGroup>> {
    blocking(move || list_all_todos_inner(&app.state::<AppState>())).await
}

fn get_file_history_inner(state: &AppState, project_id: &str, session_id: &str) -> Result<FileHistory> {
    project_dir(&state.claude_home, project_id)?; // the project must exist; the loader finds the transcript itself
    let entries = file_history::load(&state.claude_home.projects_dir(), session_id, TargetOs::current())?.value;
    Ok(to_file_history(entries))
}

#[tauri::command]
#[specta::specta]
pub async fn get_file_history(app: AppHandle, project_id: String, session_id: String) -> Result<FileHistory> {
    blocking(move || get_file_history_inner(&app.state::<AppState>(), &project_id, &session_id)).await
}

fn version_content(state: &AppState, session_id: &str, version_id: &str) -> Result<String> {
    file_history::load_content(&state.claude_home.file_history_dir(), session_id, version_id)?
        .ok_or_else(|| AppError::not_found(format!("no version {version_id} for session {session_id}")))
}

fn get_file_version_inner(state: &AppState, session_id: &str, version_id: &str) -> Result<FileVersionContent> {
    let content = version_content(state, session_id, version_id)?;
    Ok(FileVersionContent { version_id: version_id.to_string(), content })
}

#[tauri::command]
#[specta::specta]
pub async fn get_file_version(app: AppHandle, session_id: String, version_id: String) -> Result<FileVersionContent> {
    blocking(move || get_file_version_inner(&app.state::<AppState>(), &session_id, &version_id)).await
}

fn diff_file_versions_inner(state: &AppState, session_id: &str, from_version_id: &str, to_version_id: &str) -> Result<DiffResult> {
    let old = version_content(state, session_id, from_version_id)?;
    let new = version_content(state, session_id, to_version_id)?;
    Ok(diff::diff(&old, &new))
}

#[tauri::command]
#[specta::specta]
pub async fn diff_file_versions(app: AppHandle, session_id: String, from_version_id: String, to_version_id: String) -> Result<DiffResult> {
    blocking(move || diff_file_versions_inner(&app.state::<AppState>(), &session_id, &from_version_id, &to_version_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn diff_lines(old_text: String, new_text: String) -> Result<DiffResult> {
    blocking(move || Ok(diff::diff(&old_text, &new_text))).await
}

fn get_debug_log_page_inner(state: &AppState, session_id: &str, offset: u32, limit: u32, level: Option<&str>) -> Result<heimdall_core::data::debug_log::DebugLogPage> {
    debug_log::page(&state.claude_home.debug_dir(), session_id, offset, limit, parse_level(level)?)
}

#[tauri::command]
#[specta::specta]
pub async fn get_debug_log_page(app: AppHandle, session_id: String, offset: u32, limit: u32, level: Option<String>) -> Result<debug_log::DebugLogPage> {
    blocking(move || get_debug_log_page_inner(&app.state::<AppState>(), &session_id, offset, limit, level.as_deref())).await
}

fn get_session_facets_inner(state: &AppState, session_id: &str) -> Result<Option<SessionFacets>> {
    facets::load(&state.claude_home.facets_dir(), session_id)
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_facets(app: AppHandle, session_id: String) -> Result<Option<SessionFacets>> {
    blocking(move || get_session_facets_inner(&app.state::<AppState>(), &session_id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;

    const PROJECT: &str = "-Users-fixture-src-app";
    const SESSION: &str = "11111111-1111-4111-8111-111111111111";

    fn state() -> AppState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        AppState { claude_home: ClaudeHome::new(root.join("claude-home"), root.join("claude.json")), cache: crate::state::Cache::default() }
    }

    #[test]
    fn get_session_todos_keeps_the_main_list_and_the_agent_list_apart() {
        let lists = get_session_todos_inner(&state(), SESSION).unwrap();
        assert_eq!(lists.len(), 2);
        assert!(lists.iter().any(|l| l.agent_id.is_none() && !l.items.is_empty()));
        assert!(lists.iter().any(|l| l.agent_id.as_deref() == Some("a9b1c2d3")));
    }

    #[test]
    fn get_session_todos_unknown_session_is_empty_not_an_error() {
        assert!(get_session_todos_inner(&state(), "nonexistent-999").unwrap().is_empty());
    }

    #[test]
    fn list_all_todos_groups_the_fixture_session_under_its_real_project() {
        let groups = list_all_todos_inner(&state()).unwrap();
        let group = groups.iter().find(|g| g.project_id.as_deref() == Some(PROJECT)).expect("fixture project group");
        assert_eq!(group.project_name.as_deref(), Some("app"));
        assert_eq!(group.lists.iter().map(|l| l.session_id.as_str()).collect::<std::collections::BTreeSet<_>>().len(), 1);
    }

    #[test]
    fn get_file_history_lists_both_versions_of_the_tracked_file() {
        let history = get_file_history_inner(&state(), PROJECT, SESSION).unwrap();
        assert_eq!(history.files.len(), 1);
        let versions: Vec<_> = history.files[0].versions.iter().map(|v| v.version_id.as_str()).collect();
        assert_eq!(versions, ["3f2504e5f2a14f9f@v1", "3f2504e5f2a14f9f@v2"]);
    }

    #[test]
    fn get_file_history_rejects_an_unknown_project_as_not_found() {
        assert!(matches!(get_file_history_inner(&state(), "-Users-nobody-nowhere", SESSION).unwrap_err(), AppError::NotFound { .. }));
    }

    #[test]
    fn get_file_version_returns_the_backups_real_content() {
        let content = get_file_version_inner(&state(), SESSION, "3f2504e5f2a14f9f@v1").unwrap();
        assert_eq!(content.version_id, "3f2504e5f2a14f9f@v1");
        assert!(!content.content.is_empty());
    }

    #[test]
    fn get_file_version_rejects_an_unknown_version_as_not_found() {
        assert!(matches!(get_file_version_inner(&state(), SESSION, "nope@v9").unwrap_err(), AppError::NotFound { .. }));
    }

    #[test]
    fn diff_file_versions_diffs_the_two_real_backups() {
        let result = diff_file_versions_inner(&state(), SESSION, "3f2504e5f2a14f9f@v1", "3f2504e5f2a14f9f@v2").unwrap();
        assert!(!result.lines.is_empty());
    }

    #[test]
    fn diff_lines_truncates_at_5000_lines() {
        let huge = (0..6000).map(|i| format!("line{i}")).collect::<Vec<_>>().join("\n");
        let result = tauri::async_runtime::block_on(diff_lines(huge, "x".to_string())).unwrap();
        assert!(result.truncated);
    }

    #[test]
    fn diff_lines_small_input_is_not_truncated() {
        let result = tauri::async_runtime::block_on(diff_lines("a\nb".to_string(), "a\nc".to_string())).unwrap();
        assert!(!result.truncated);
    }

    #[test]
    fn get_debug_log_page_reads_the_real_fixture_log() {
        let page = get_debug_log_page_inner(&state(), SESSION, 0, 10, None).unwrap();
        assert_eq!(page.total, 3);
        assert_eq!(page.entries.len(), 3);
    }

    #[test]
    fn get_debug_log_page_filters_by_level() {
        let page = get_debug_log_page_inner(&state(), SESSION, 0, 10, Some("error")).unwrap();
        assert_eq!(page.total, 1);
        assert_eq!(page.entries[0].level, LogLevel::Error);
    }

    #[test]
    fn get_debug_log_page_rejects_an_unknown_level_as_invalid_input() {
        assert!(matches!(get_debug_log_page_inner(&state(), SESSION, 0, 10, Some("verbose")).unwrap_err(), AppError::InvalidInput { .. }));
    }

    #[test]
    fn get_session_facets_reads_the_real_fixture_file() {
        let facets = get_session_facets_inner(&state(), SESSION).unwrap();
        assert!(facets.is_some());
    }

    #[test]
    fn get_session_facets_unknown_session_returns_none() {
        assert!(get_session_facets_inner(&state(), "nonexistent-999").unwrap().is_none());
    }
}
