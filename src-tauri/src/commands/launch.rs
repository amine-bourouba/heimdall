//! Commands in commands/launch.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! `open_external_url`'s real body landed in M2-11; `open_in_editor`/`open_terminal`/
//! `reveal_in_file_manager` land here (M3-53). Every body runs through `state::blocking`
//! (contract §1).
//!
//! AC1 ("no command accepts a path from the webview"): [`EditorTarget`]/[`RevealTarget`] carry
//! only ids (`heimdall_core::os_integration::launch`'s own doc comment: "every variant names an id
//! the command resolves itself, never a path from the webview") -- this file's whole job is
//! resolving each one to a real, validated path server-side, the same way `commands/sessions.rs`/
//! `commands/config_read.rs` already resolve every other id-shaped input.

use std::path::{Path, PathBuf};

use heimdall_core::config::read::item_path;
use heimdall_core::config::write::mcp::line_number_for_server;
use heimdall_core::models::message::{ContentBlock, ToolUse};
use heimdall_core::models::session::Session;
use heimdall_core::os_integration::launch::{self, EditorTarget, Launch, RevealTarget};
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::load_project;
use heimdall_core::transcript::full::{self, ParseContext};
use heimdall_core::{AppError, Result};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::config_read::{memory_files_for_source, resolve_project_scope};
use super::sessions::{find_session, project_dir};
use crate::state::{AppState, blocking};
use crate::{platform, prefs};

#[tauri::command]
#[specta::specta]
pub async fn open_external_url(url: String) -> Result<()> {
    // The allowlist check and the opener call both happen inside `links::open_external_url`
    // (heimdall-core, tauri-free and directly unit-tested there, ADR 0010 control 2) -- this is
    // only the real opener plugin wired in as the injected closure.
    heimdall_core::os_integration::links::open_external_url(&url, |u| {
        tauri_plugin_opener::open_url(u, None::<&str>).map_err(std::io::Error::other)
    })
}

/// `heimdall_core::data::history` and friends already resolve a project/session id this exact
/// way; the only reason this is re-derived here rather than reusing `sessions::get_session_inner`
/// is that *that* function returns a `SessionDetail` DTO built for IPC, not the raw `Session` a
/// tool block needs to be searched within -- and reuses the cache, which an infrequent,
/// user-initiated "open in editor" click has no real need for.
pub(super) fn load_full_session(state: &AppState, project_id: &str, session_id: &str) -> Result<Session> {
    let dir = project_dir(&state.claude_home, project_id)?;
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
    full::parse(&path, ctx)?
        .value
        .ok_or_else(|| AppError::not_found("transcript has no messages"))
}

/// `block_id` is `<messageId>:<index>` (`ContentBlock`'s own doc comment) -- the message half
/// narrows the search before confirming the exact block, rather than scanning every block of
/// every message for an id match.
pub(super) fn find_tool_block<'a>(session: &'a Session, block_id: &str) -> Result<&'a ToolUse> {
    let message_id = block_id.split(':').next().unwrap_or(block_id);
    session
        .messages
        .iter()
        .find(|m| m.id == message_id)
        .into_iter()
        .flat_map(|m| &m.blocks)
        .find_map(|b| match b {
            ContentBlock::ToolUse(t) if t.id == block_id => Some(t),
            _ => None,
        })
        .ok_or_else(|| AppError::not_found(format!("no tool block {block_id}")))
}

/// The directory a config item's kind folder lives under: `~/.claude` for a global item, or
/// `<project>/.claude` for a project-scoped one -- the same resolution `config_write.rs`'s own
/// `write_dir` already established for writes, needed again here for reads-to-open.
fn config_dir<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    project: Option<&heimdall_core::models::config::scope::ProjectContext>,
) -> Result<PathBuf> {
    let project_root = project.map(|c| resolve_project_scope(app, state, c)).transpose()?;
    Ok(project_root.map_or_else(|| state.claude_home.root().to_path_buf(), |r| r.join(".claude")))
}

/// Resolves an [`EditorTarget`] to a real, absolute path plus an optional line number -- the one
/// place any of this ever touches a path at all (AC1).
fn resolve_editor_target<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    target: &EditorTarget,
) -> Result<(PathBuf, Option<u32>)> {
    match target {
        EditorTarget::ToolBlock {
            project_id,
            session_id,
            block_id,
            line,
        } => {
            let session = load_full_session(state, project_id, session_id)?;
            let block = find_tool_block(&session, block_id)?;
            let path = block
                .file_path
                .clone()
                .ok_or_else(|| AppError::not_found("this tool call has no file path"))?;
            Ok((PathBuf::from(path), *line))
        }
        EditorTarget::ConfigItem { config_ref } => {
            let dir = config_dir(app, state, config_ref.project.as_ref())?;
            Ok((item_path(&dir, config_ref.kind, &config_ref.slug)?, None))
        }
        EditorTarget::MemoryFile { source_id, file_id } => {
            let file = memory_files_for_source(app, state, source_id)?
                .into_iter()
                .find(|f| &f.id == file_id)
                .ok_or_else(|| AppError::not_found(format!("no memory file {file_id}")))?;
            Ok((file.file, None))
        }
        EditorTarget::McpConfig { server_name } => {
            let config_file = state.claude_home.config_file();
            let line = line_number_for_server(config_file, server_name).map(|n| n as u32);
            Ok((config_file.to_path_buf(), line))
        }
    }
}

fn env_var(name: &str) -> String {
    std::env::var(name).unwrap_or_default()
}

fn open_in_editor_inner<R: Runtime>(app: &AppHandle<R>, state: &AppState, target: &EditorTarget) -> Result<()> {
    let (path, line) = resolve_editor_target(app, state, target)?;
    let abs_path = path
        .to_str()
        .ok_or_else(|| AppError::invalid_input("path is not valid UTF-8"))?;
    let os = TargetOs::current();
    let is_regular_file = path.is_file();
    let editor = prefs::load(app)?.text_editor;

    match launch::editor_url(editor, abs_path, line, os, is_regular_file)? {
        Some(url) => platform::open_url(&url),
        // Only Xcode has no URL scheme (`editor_url`'s own doc comment) -- the CLI fallback below
        // is reached for that one editor only, never as a "URL scheme didn't work" retry, since
        // `editor_url` never fails for a reason the CLI path could recover from.
        None => {
            let cli = heimdall_core::os_integration::cli_locate::find_editor_cli(
                editor.cli_command(),
                &env_var("PATH"),
                &env_var("PATHEXT"),
                os,
                &|p| p.exists(),
            )
            .ok_or_else(|| AppError::not_found(format!("{} not found on PATH", editor.display_name())))?;
            let argv = launch::editor_cli_argv(editor, abs_path, line, &cli, os, is_regular_file)?;
            platform::run(&Launch::Argv(argv))
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn open_in_editor(app: AppHandle, target: EditorTarget) -> Result<()> {
    blocking(move || open_in_editor_inner(&app, &app.state::<AppState>(), &target)).await
}

/// Copies `claude --resume <sessionId>` to the clipboard (only when a session is given -- a bare
/// "open a terminal here" has nothing to resume) and opens the preferred terminal at the
/// project's own working directory. No command is ever passed to the terminal itself
/// (`heimdall_core::os_integration::launch::open_terminal`'s own doc comment explains why);
/// the Swift reference app's own `TerminalLauncher.run` injects the command via AppleScript instead
/// (`TerminalLauncher.swift:4-16`), the one behaviour this task's own AC2 deliberately does not
/// port (port-plan C-5).
fn open_terminal_inner<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    project_id: &str,
    session_id: Option<&str>,
) -> Result<()> {
    let dir = project_dir(&state.claude_home, project_id)?;
    let project = load_project(&dir, TargetOs::current()).value;
    let cwd = project
        .path
        .ok_or_else(|| AppError::not_found(format!("no project: {project_id}")))?;

    if let Some(session_id) = session_id {
        let text = launch::resume_command_text(session_id)?;
        app.clipboard()
            .write_text(text)
            .map_err(|e| AppError::io(e.to_string()))?;
    }

    let terminal = prefs::load(app)?.preferred_terminal;
    let is_directory = Path::new(&cwd).is_dir();
    let launched = launch::open_terminal(terminal, &cwd, TargetOs::current(), is_directory)?;
    platform::run(&launched)
}

#[tauri::command]
#[specta::specta]
pub async fn open_terminal(app: AppHandle, project_id: String, session_id: Option<String>) -> Result<()> {
    blocking(move || open_terminal_inner(&app, &app.state::<AppState>(), &project_id, session_id.as_deref())).await
}

/// Resolves a [`RevealTarget`] to a real, absolute path -- the one place `reveal_in_file_manager`
/// ever touches a path at all (AC1).
fn resolve_reveal_target<R: Runtime>(app: &AppHandle<R>, state: &AppState, target: &RevealTarget) -> Result<PathBuf> {
    match target {
        RevealTarget::Session { project_id, session_id } => {
            let dir = project_dir(&state.claude_home, project_id)?;
            let project = load_project(&dir, TargetOs::current()).value;
            let found = find_session(&project, session_id)?;
            found
                .file
                .clone()
                .ok_or_else(|| AppError::not_found(format!("no transcript for {session_id}")))
        }
        RevealTarget::ConfigItem { config_ref } => {
            let dir = config_dir(app, state, config_ref.project.as_ref())?;
            item_path(&dir, config_ref.kind, &config_ref.slug)
        }
        RevealTarget::McpConfigFile => Ok(state.claude_home.config_file().to_path_buf()),
    }
}

fn reveal_in_file_manager_inner<R: Runtime>(app: &AppHandle<R>, state: &AppState, target: &RevealTarget) -> Result<()> {
    let path = resolve_reveal_target(app, state, target)?;
    platform::reveal(&path)
}

#[tauri::command]
#[specta::specta]
pub async fn reveal_in_file_manager(app: AppHandle, target: RevealTarget) -> Result<()> {
    blocking(move || reveal_in_file_manager_inner(&app, &app.state::<AppState>(), &target)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::models::config::scope::{ConfigKind, ConfigRef, ProjectContext};
    use heimdall_core::paths::ClaudeHome;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    // MARK: - find_tool_block (pure; a real parsed session, no fixture needed)

    fn session_with_tool(session_id: &str, file_path: &str) -> Session {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(format!("{session_id}.jsonl"));
        let jsonl = format!(
            "{}\n{}",
            serde_json::json!({"type":"user","uuid":"u1","timestamp":"2026-01-01T00:00:00.000Z","isSidechain":false,"message":{"role":"user","content":"hi"}}),
            serde_json::json!({"type":"assistant","uuid":"a1","timestamp":"2026-01-01T00:00:01.000Z","isSidechain":false,"message":{"id":"msg1","role":"assistant","model":"claude-opus-4-6","content":[{"type":"tool_use","id":"tu1","name":"Read","input":{"file_path":file_path}}],"usage":{"input_tokens":1,"output_tokens":1}}}),
        );
        std::fs::write(&path, jsonl).unwrap();
        let ctx = ParseContext {
            project_path: "",
            session_id,
            index_started_at: None,
            is_sidechain: false,
        };
        full::parse(&path, ctx).unwrap().value.unwrap()
    }

    #[test]
    fn find_tool_block_locates_the_exact_block() {
        let session = session_with_tool("s1", "/abs/file.rs");
        let message_id = &session.messages[1].id;
        let block_id = format!("{message_id}:0");
        let block = find_tool_block(&session, &block_id).unwrap();
        assert_eq!(block.file_path.as_deref(), Some("/abs/file.rs"));
    }

    #[test]
    fn find_tool_block_unknown_id_is_not_found() {
        let session = session_with_tool("s1", "/abs/file.rs");
        assert!(matches!(
            find_tool_block(&session, "nonexistent:0").unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    #[test]
    fn find_tool_block_wrong_index_on_a_real_message_is_not_found() {
        let session = session_with_tool("s1", "/abs/file.rs");
        let message_id = &session.messages[1].id;
        assert!(matches!(
            find_tool_block(&session, &format!("{message_id}:9")).unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    // MARK: - resolve_editor_target / resolve_reveal_target (the real path-resolution logic;
    // never the `_inner` wrappers, which end by calling the real `platform` module -- opening a
    // real editor/terminal/file manager window on this machine, exactly the kind of side effect
    // this session's own earlier mishap (see the M3-56 progress-log entry) showed is not worth
    // the risk of triggering from an automated test).

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        let claude_home = ClaudeHome::new(root.join("claude-home"), root.join("claude.json"));
        let app = mock_builder()
            .plugin(tauri_plugin_store::Builder::default().build())
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        app.manage(AppState {
            claude_home,
            cache: crate::state::Cache::default(),
        });
        app
    }

    #[test]
    fn resolve_editor_target_tool_block_extracts_the_real_file_path() {
        // Confirms the full chain (project -> session -> transcript -> block), not just
        // `find_tool_block` in isolation: a fresh project dir under its own `AppState`, with one
        // real session file containing one real tool_use block.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        let project_dir = root.join("projects").join("proj");
        std::fs::create_dir_all(&project_dir).unwrap();
        let session_id = "a1b2c3d4-e5f6-7890-abcd-ef1234567890";
        let jsonl = format!(
            "{}\n{}",
            serde_json::json!({"type":"user","uuid":"u1","timestamp":"2026-01-01T00:00:00.000Z","isSidechain":false,"message":{"role":"user","content":"hi"}}),
            serde_json::json!({"type":"assistant","uuid":"a1","timestamp":"2026-01-01T00:00:01.000Z","isSidechain":false,"message":{"id":"msg1","role":"assistant","model":"claude-opus-4-6","content":[{"type":"tool_use","id":"tu1","name":"Edit","input":{"file_path":"/abs/target.rs"}}],"usage":{"input_tokens":1,"output_tokens":1}}}),
        );
        std::fs::write(project_dir.join(format!("{session_id}.jsonl")), jsonl).unwrap();

        let claude_home = ClaudeHome::new(&root, root.join(".claude.json"));
        let mock = mock_builder()
            .plugin(tauri_plugin_store::Builder::default().build())
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        mock.manage(AppState {
            claude_home: claude_home.clone(),
            cache: crate::state::Cache::default(),
        });
        let state = AppState {
            claude_home,
            cache: crate::state::Cache::default(),
        };

        let session = load_full_session(&state, "proj", session_id).unwrap();
        let message_id = &session.messages[1].id;
        let target = EditorTarget::ToolBlock {
            project_id: "proj".to_string(),
            session_id: session_id.to_string(),
            block_id: format!("{message_id}:0"),
            line: Some(7),
        };
        let (path, line) = resolve_editor_target(&mock.handle().clone(), &state, &target).unwrap();
        assert_eq!((path, line), (PathBuf::from("/abs/target.rs"), Some(7)));
    }

    /// A fresh tempdir `AppState` -- never the shared `fixtures/synthetic` tree `app()` reads,
    /// since this test writes a config item into it.
    fn state_with_tempdir() -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn resolve_editor_target_config_item_global_scope() {
        let state = state_with_tempdir();
        let dir = state.claude_home.root().join("commands");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("review.md"), "body").unwrap();
        let mock = mock_builder()
            .plugin(tauri_plugin_store::Builder::default().build())
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        let target = EditorTarget::ConfigItem {
            config_ref: ConfigRef {
                kind: ConfigKind::Command,
                project: None,
                slug: "review".to_string(),
            },
        };
        let (path, line) = resolve_editor_target(&mock.handle().clone(), &state, &target).unwrap();
        assert_eq!((path, line), (dir.join("review.md"), None));
    }

    #[test]
    fn resolve_editor_target_memory_file_resolves_a_real_project_memory_file() {
        let app = app();
        let state = app.state::<AppState>();
        let source_id = "project:-Users-fixture-src-app";
        let files = memory_files_for_source(&app.handle().clone(), &state, source_id).unwrap();
        let target = EditorTarget::MemoryFile {
            source_id: source_id.to_string(),
            file_id: files[0].id.clone(),
        };
        let (path, line) = resolve_editor_target(&app.handle().clone(), &state, &target).unwrap();
        assert_eq!((path, line), (files[0].file.clone(), None));
    }

    #[test]
    fn resolve_editor_target_mcp_config_names_the_claude_json_file_and_its_line() {
        let app = app();
        let state = app.state::<AppState>();
        let target = EditorTarget::McpConfig {
            server_name: "github".to_string(),
        };
        let (path, line) = resolve_editor_target(&app.handle().clone(), &state, &target).unwrap();
        assert_eq!(path, state.claude_home.config_file());
        assert!(line.is_some(), "expected a real line number for a known server");
    }

    #[test]
    fn resolve_editor_target_rejects_an_unknown_project_for_a_project_scoped_config_item() {
        let app = app();
        let state = app.state::<AppState>();
        let target = EditorTarget::ConfigItem {
            config_ref: ConfigRef {
                kind: ConfigKind::Command,
                project: Some(ProjectContext::Project {
                    project_id: "-Users-nobody-nowhere".to_string(),
                }),
                slug: "x".to_string(),
            },
        };
        assert!(matches!(
            resolve_editor_target(&app.handle().clone(), &state, &target).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn resolve_reveal_target_session_names_the_real_transcript_file() {
        let app = app();
        let state = app.state::<AppState>();
        let path = resolve_reveal_target(
            &app.handle().clone(),
            &state,
            &RevealTarget::Session {
                project_id: "-Users-fixture-Dev-poirot".to_string(),
                session_id: "conv-001".to_string(),
            },
        )
        .unwrap();
        assert!(path.ends_with("conv-001.jsonl"));
    }

    #[test]
    fn resolve_reveal_target_mcp_config_file_is_the_claude_json_path() {
        let app = app();
        let state = app.state::<AppState>();
        let path = resolve_reveal_target(&app.handle().clone(), &state, &RevealTarget::McpConfigFile).unwrap();
        assert_eq!(path, state.claude_home.config_file());
    }
}
