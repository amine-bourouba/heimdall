//! Commands in commands/sessions.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! real bodies landed in M1-41 for the MVP rows; `delete_session`/`delete_project` land here
//! (M3-46). Every body runs through `state::blocking` (contract §1).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use heimdall_core::classify::Area;
use heimdall_core::data::tray::RecentSession;
use heimdall_core::data::{debug_log, file_history, todos};
use heimdall_core::models::context_usage::ContextUsage;
use heimdall_core::models::message::MessagePage;
use heimdall_core::models::project::{Project, ProjectSummary};
use heimdall_core::models::session::{Session, SessionDetail, SessionSummary};
use heimdall_core::paths::{ClaudeHome, TargetOs, plain_file_name};
use heimdall_core::projects::delete as delete_core;
use heimdall_core::projects::discovery::{discover, load_project};
use heimdall_core::transcript::full::{self, ParseContext};
use heimdall_core::{AppError, Loaded, Result, WriteOutcome};
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking, confirm};

/// Resolves a `ProjectId` to its folder under `~/.claude/projects/`; rejects a malformed id
/// before it ever becomes a path (ADR 0010) and one that does not exist as `notFound` (contract §4).
pub(super) fn project_dir(claude_home: &ClaudeHome, project_id: &str) -> Result<PathBuf> {
    let name = plain_file_name(project_id)?;
    let dir = claude_home.projects_dir().join(name);
    if !dir.is_dir() {
        return Err(AppError::not_found(format!("no project: {project_id}")));
    }
    Ok(dir)
}

/// Rejects a session id that could escape its project folder before it is ever compared to one
/// (ADR 0010) — the same `plain_file_name` safety check every other session-id-as-filename loader
/// already uses (debug logs, todos, file history), rather than the launch mechanism's stricter
/// UUID/`agent-` shape (`os_integration::launch::valid_session_id`), which real Claude Code ids
/// always satisfy but this task's own fixture data (`fixtures/screenshot`, short names like
/// `conv-001` from ScreenshotData.swift) does not.
fn find_session<'a>(project: &'a Project, session_id: &str) -> Result<&'a Session> {
    plain_file_name(session_id)?;
    project
        .sessions
        .iter()
        .find(|s| s.id == session_id)
        .ok_or_else(|| AppError::not_found(format!("no session: {session_id}")))
}

fn fill_flags(claude_home: &ClaudeHome, summary: &mut SessionSummary) {
    summary.has_debug_log = debug_log::has_log(&claude_home.debug_dir(), &summary.id);
    summary.has_file_history = file_history::has_any(&claude_home.file_history_dir(), &summary.id);
    summary.has_todos = todos::has_any(&claude_home.todos_dir(), &summary.id);
}

fn has_memory(claude_home: &ClaudeHome, project_id: &str) -> bool {
    heimdall_core::config::read::memory::folder_file_count(&claude_home.projects_dir().join(project_id).join("memory"))
        > 0
}

/// The session's full parse, from the cache if a fresh entry exists, else read and cached fresh.
fn cached_full_parse(state: &AppState, path: &Path, ctx: ParseContext) -> Result<(Session, u32)> {
    let (mtime, size) = metadata(path)?;
    if let Some(cached) = state.cache.get::<(Session, u32)>(path, mtime, size) {
        return Ok(cached);
    }
    let loaded = full::parse(path, ctx)?;
    let session = loaded
        .value
        .ok_or_else(|| AppError::not_found("transcript has no messages"))?;
    let result = (session, loaded.skipped);
    state.cache.put(path, mtime, size, Area::Transcript, result.clone());
    Ok(result)
}

fn metadata(path: &Path) -> Result<(std::time::SystemTime, u64)> {
    let meta = std::fs::metadata(path).map_err(|e| AppError::from_io(&e, path))?;
    let mtime = meta.modified().map_err(|e| AppError::from_io(&e, path))?;
    Ok((mtime, meta.len()))
}

fn message_page(messages: &[heimdall_core::models::message::Message], start: u32, count: u32) -> MessagePage {
    let from = (start as usize).min(messages.len());
    let to = from.saturating_add(count as usize).min(messages.len());
    MessagePage {
        start,
        messages: messages[from..to].to_vec(),
        total: messages.len() as u32,
    }
}

fn tool_names(session: &Session) -> Vec<String> {
    session
        .messages
        .iter()
        .flat_map(|m| m.tool_blocks())
        .map(|t| t.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn list_projects_inner(state: &AppState) -> Result<Loaded<Vec<ProjectSummary>>> {
    let claude_home = &state.claude_home;
    let loaded = discover(&claude_home.projects_dir(), TargetOs::current())?;
    let projects = loaded
        .value
        .iter()
        .map(|p| p.summary(has_memory(claude_home, &p.id)))
        .collect();
    Ok(Loaded::new(projects, loaded.skipped))
}

#[tauri::command]
#[specta::specta]
pub async fn list_projects(app: AppHandle) -> Result<Loaded<Vec<ProjectSummary>>> {
    blocking(move || list_projects_inner(&app.state::<AppState>())).await
}

fn list_sessions_inner(
    state: &AppState,
    project_id: &str,
    include_agents: bool,
) -> Result<Loaded<Vec<SessionSummary>>> {
    let claude_home = &state.claude_home;
    let dir = project_dir(claude_home, project_id)?;
    let loaded = load_project(&dir, TargetOs::current());
    let sessions = loaded
        .value
        .sessions
        .iter()
        .filter(|s| include_agents || !s.is_sidechain)
        .map(|s| {
            let mut summary = s.summary(project_id);
            fill_flags(claude_home, &mut summary);
            summary
        })
        .collect();
    Ok(Loaded::new(sessions, loaded.skipped))
}

#[tauri::command]
#[specta::specta]
pub async fn list_sessions(
    app: AppHandle,
    project_id: String,
    include_agents: bool,
) -> Result<Loaded<Vec<SessionSummary>>> {
    blocking(move || list_sessions_inner(&app.state::<AppState>(), &project_id, include_agents)).await
}

fn get_session_inner(state: &AppState, project_id: &str, session_id: &str) -> Result<SessionDetail> {
    let claude_home = &state.claude_home;
    let dir = project_dir(claude_home, project_id)?;
    let project = load_project(&dir, TargetOs::current()).value;
    let found = find_session(&project, session_id)?;
    let path = found
        .file
        .clone()
        .ok_or_else(|| AppError::not_found(format!("no transcript for {session_id}")))?;
    let project_path = project.path.clone().unwrap_or_default();
    let ctx = ParseContext {
        project_path: &project_path,
        session_id,
        index_started_at: Some(found.started_at),
        is_sidechain: found.is_sidechain,
    };

    let (full_session, skipped) = cached_full_parse(state, &path, ctx)?;
    let mut summary = full_session.summary(project_id);
    fill_flags(claude_home, &mut summary);
    let context = ContextUsage::from_messages(&full_session.messages);
    Ok(SessionDetail {
        summary,
        message_count: full_session.messages.len() as u32,
        context: (!context.is_empty()).then_some(context),
        tool_names: tool_names(&full_session),
        skipped,
    })
}

#[tauri::command]
#[specta::specta]
pub async fn get_session(app: AppHandle, project_id: String, session_id: String) -> Result<SessionDetail> {
    blocking(move || get_session_inner(&app.state::<AppState>(), &project_id, &session_id)).await
}

fn get_session_messages_inner(
    state: &AppState,
    project_id: &str,
    session_id: &str,
    start: u32,
    count: u32,
) -> Result<MessagePage> {
    let claude_home = &state.claude_home;
    let dir = project_dir(claude_home, project_id)?;
    let project = load_project(&dir, TargetOs::current()).value;
    let found = find_session(&project, session_id)?;
    let path = found
        .file
        .clone()
        .ok_or_else(|| AppError::not_found(format!("no transcript for {session_id}")))?;
    let project_path = project.path.clone().unwrap_or_default();
    let ctx = ParseContext {
        project_path: &project_path,
        session_id,
        index_started_at: Some(found.started_at),
        is_sidechain: found.is_sidechain,
    };

    let (mtime, size) = metadata(&path)?;
    if let Some((session, _)) = state.cache.get::<(Session, u32)>(&path, mtime, size) {
        return Ok(message_page(&session.messages, start, count));
    }

    // Fast path: read only as much of the transcript as this page needs, so a huge session's
    // first page renders without waiting on a full parse (M1-41 acceptance criterion).
    let (loaded, reached_eof) = full::parse_bounded(&path, ctx, (start as usize).saturating_add(count as usize))?;
    let bounded = loaded
        .value
        .ok_or_else(|| AppError::not_found("transcript has no messages"))?;
    if reached_eof {
        state
            .cache
            .put(&path, mtime, size, Area::Transcript, (bounded.clone(), loaded.skipped));
        return Ok(message_page(&bounded.messages, start, count));
    }

    // ponytail: the bounded scan did not reach the end of the file, so its message count is
    // only a lower bound — falling back to a full parse trades away the speed win rather than
    // exposing an under-count as `total`. Upgrade path: track a running count during the
    // bounded scan and let a later request refine `total` once the full parse is cached.
    let (full_session, _) = cached_full_parse(state, &path, ctx)?;
    Ok(message_page(&full_session.messages, start, count))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_messages(
    app: AppHandle,
    project_id: String,
    session_id: String,
    start: u32,
    count: u32,
) -> Result<MessagePage> {
    blocking(move || get_session_messages_inner(&app.state::<AppState>(), &project_id, &session_id, start, count)).await
}

fn list_recent_sessions_inner(state: &AppState, limit: u32) -> Result<Vec<RecentSession>> {
    let claude_home = &state.claude_home;
    let projects = discover(&claude_home.projects_dir(), TargetOs::current())?.value;
    let mut all: Vec<(&Project, &Session)> = projects
        .iter()
        .flat_map(|p| p.sessions.iter().map(move |s| (p, s)))
        .collect();
    all.sort_by_key(|(_, s)| std::cmp::Reverse(s.started_at));
    all.truncate(limit as usize);
    Ok(all
        .into_iter()
        .map(|(p, s)| RecentSession {
            project_id: p.id.clone(),
            project_name: p.name.clone(),
            session: s.summary(p.id.clone()),
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn list_recent_sessions(app: AppHandle, limit: u32) -> Result<Vec<RecentSession>> {
    blocking(move || list_recent_sessions_inner(&app.state::<AppState>(), limit)).await
}

fn delete_session_inner(state: &AppState, project_id: &str, session_id: &str, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    let claude_home = &state.claude_home;
    delete_core::delete_session(
        &claude_home.projects_dir(),
        &claude_home.todos_dir(),
        project_id,
        session_id,
    )
}

#[tauri::command]
#[specta::specta]
pub async fn delete_session(app: AppHandle, project_id: String, session_id: String) -> Result<WriteOutcome> {
    blocking(move || {
        let confirmed = confirm(
            &app,
            "Confirm Delete",
            "Delete this session? This cannot be undone.",
            "Delete",
        );
        delete_session_inner(&app.state::<AppState>(), &project_id, &session_id, confirmed)
    })
    .await
}

fn delete_project_inner(state: &AppState, project_id: &str, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    delete_core::delete_project(&state.claude_home.projects_dir(), project_id)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_project(app: AppHandle, project_id: String) -> Result<WriteOutcome> {
    blocking(move || {
        let confirmed = confirm(
            &app,
            "Confirm Delete",
            "Delete this whole project, including every session in it? This cannot be undone.",
            "Delete",
        );
        delete_project_inner(&app.state::<AppState>(), &project_id, confirmed)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    const POIROT: &str = "-Users-fixture-Dev-poirot";

    // Every command's real body is a thin `blocking` + `AppHandle` wrapper around one of the
    // `*_inner` functions above, which is all the tests below exercise: `AppHandle` only ever
    // resolves to a managed `AppState`, so testing against a plain one built straight from the
    // fixture (no mock Tauri app, no runtime) covers the same logic without needing either.
    fn state() -> AppState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/screenshot");
        AppState {
            claude_home: ClaudeHome::new(root.join("claude-home"), root.join("claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn list_projects_covers_all_four_fixture_projects() {
        let loaded = list_projects_inner(&state()).unwrap();
        let mut ids: Vec<_> = loaded.value.iter().map(|p| p.id.clone()).collect();
        ids.sort();
        assert_eq!(
            ids,
            [
                "-Users-fixture-Dev-ignio-web",
                "-Users-fixture-Dev-poirot",
                "-Users-fixture-Dev-reellette-ios",
                "-Users-fixture-Dev-swift-openapi"
            ]
        );
    }

    #[test]
    fn list_projects_rejects_nothing_and_never_errors_on_this_fixture() {
        assert_eq!(list_projects_inner(&state()).unwrap().skipped, 0);
    }

    #[test]
    fn list_sessions_returns_every_session_in_the_poirot_project() {
        let loaded = list_sessions_inner(&state(), POIROT, true).unwrap();
        let mut ids: Vec<_> = loaded.value.iter().map(|s| s.id.clone()).collect();
        ids.sort();
        // include_agents: true also surfaces conv-001's four real subagent transcripts
        // (conv-001/subagents/agent-*.jsonl) as their own sidechain sessions — matching
        // SessionLoader.swift's own isSidechain: true synthesis and AppState.swift's
        // showAgentSessions filter, confirmed by reading both directly. This test's own
        // expected list predates those four fixture files and never actually ran to catch the
        // drift: this crate's tests had never executed on any OS until a real cross-OS CI run
        // found this (an earlier heimdall-core bug always aborted `cargo test --workspace` first).
        assert_eq!(
            ids,
            [
                "agent-a5984cda07de98171",
                "agent-a7e39e5ee252c7f4e",
                "agent-acb7b6172385db44c",
                "agent-af4e08a895ed07f7c",
                "alltools-001",
                "conv-001",
                "diff-001",
                "sysctx-001",
                "think-001",
                "tool-001",
                "write-001"
            ]
        );
        assert!(loaded.value.iter().all(|s| s.project_id == POIROT));
    }

    #[test]
    fn list_sessions_rejects_an_unknown_project_as_not_found() {
        assert!(matches!(
            list_sessions_inner(&state(), "-Users-nobody-nowhere", true).unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    #[test]
    fn list_sessions_rejects_a_path_escaping_project_id_as_invalid_input() {
        assert!(matches!(
            list_sessions_inner(&state(), "../etc", true).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn get_session_returns_context_and_tool_names_for_the_tool_blocks_session() {
        let detail = get_session_inner(&state(), POIROT, "tool-001").unwrap();
        assert!(detail.message_count > 0);
        assert!(!detail.tool_names.is_empty());
        assert_eq!(detail.summary.id, "tool-001");
    }

    #[test]
    fn get_session_rejects_an_unknown_session_as_not_found() {
        assert!(matches!(
            get_session_inner(&state(), POIROT, "nonexistent-999").unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    fn fixture_transcript(project_id: &str, session_id: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures/screenshot/claude-home/projects")
            .join(project_id)
            .join(format!("{session_id}.jsonl"))
    }

    #[test]
    fn get_session_caches_the_full_parse_for_a_later_call() {
        let state = state();
        get_session_inner(&state, POIROT, "conv-001").unwrap();
        let path = fixture_transcript(POIROT, "conv-001");
        let (mtime, size) = metadata(&path).unwrap();
        assert!(state.cache.get::<(Session, u32)>(&path, mtime, size).is_some());
    }

    #[test]
    fn get_session_messages_pages_a_cached_full_parse() {
        let state = state();
        get_session_inner(&state, POIROT, "conv-001").unwrap(); // warms the cache
        let full = get_session_messages_inner(&state, POIROT, "conv-001", 0, 1000).unwrap();
        let page = get_session_messages_inner(&state, POIROT, "conv-001", 1, 2).unwrap();
        assert_eq!(page.total, full.total);
        assert_eq!(page.messages, full.messages[1..3]);
    }

    #[test]
    fn get_session_messages_serves_a_first_page_without_a_warm_cache() {
        let page = get_session_messages_inner(&state(), POIROT, "conv-001", 0, 2).unwrap();
        assert_eq!(page.messages.len(), 2);
        assert_eq!(page.start, 0);
        assert!(page.total >= 2);
    }

    #[test]
    fn get_session_messages_rejects_an_unknown_session_as_not_found() {
        assert!(matches!(
            get_session_messages_inner(&state(), POIROT, "nope", 0, 10).unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    #[test]
    fn list_recent_sessions_honours_the_limit_and_sorts_by_recency() {
        let recent = list_recent_sessions_inner(&state(), 3).unwrap();
        assert_eq!(recent.len(), 3);
        assert!(
            recent
                .windows(2)
                .all(|w| w[0].session.started_at >= w[1].session.started_at)
        );
    }

    #[test]
    fn list_recent_sessions_limit_zero_returns_nothing() {
        assert!(list_recent_sessions_inner(&state(), 0).unwrap().is_empty());
    }

    /// A fresh tempdir with one project and one session file -- never the shared
    /// `fixtures/screenshot` tree `state()` reads above, since these tests delete from it.
    fn state_with_project(project_id: &str, session_id: &str) -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let project_dir = dir.path().join("projects").join(project_id);
        std::fs::create_dir_all(&project_dir).unwrap();
        std::fs::write(project_dir.join(format!("{session_id}.jsonl")), "{}").unwrap();
        std::fs::create_dir_all(dir.path().join("todos")).unwrap();
        let root = dir.keep();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn delete_session_declined_is_cancelled_and_writes_nothing() {
        let state = state_with_project("proj", "s1");
        let path = state.claude_home.projects_dir().join("proj").join("s1.jsonl");
        assert_eq!(
            delete_session_inner(&state, "proj", "s1", false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert!(path.exists());
    }

    #[test]
    fn delete_session_confirmed_removes_the_file() {
        let state = state_with_project("proj", "s1");
        let path = state.claude_home.projects_dir().join("proj").join("s1.jsonl");
        assert_eq!(
            delete_session_inner(&state, "proj", "s1", true).unwrap(),
            WriteOutcome::Written
        );
        assert!(!path.exists());
    }

    #[test]
    fn delete_project_declined_is_cancelled_and_writes_nothing() {
        let state = state_with_project("proj", "s1");
        let dir = state.claude_home.projects_dir().join("proj");
        assert_eq!(
            delete_project_inner(&state, "proj", false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert!(dir.exists());
    }

    #[test]
    fn delete_project_confirmed_removes_the_whole_folder() {
        let state = state_with_project("proj", "s1");
        let dir = state.claude_home.projects_dir().join("proj");
        assert_eq!(
            delete_project_inner(&state, "proj", true).unwrap(),
            WriteOutcome::Written
        );
        assert!(!dir.exists());
    }
}
