//! Commands in commands/export.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! `export_session`/`export_edit_patch` land here (M3-09); `export_analytics_csv`/`save_png` stay
//! stubs (M3-36). Every body runs through `state::blocking` (contract §1).

use heimdall_core::export::session_markdown::{self, ExportOptions};
use heimdall_core::export::{CsvKind, ExportDestination, PngKind, analytics_csv};
use heimdall_core::{AppError, Result, WriteOutcome};
use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_clipboard_manager::ClipboardExt;

use super::launch::{find_tool_block, load_full_session};
use crate::state::{AppState, blocking, pick_export_path};

/// Mirrors `SessionExporter.sanitizeFilename` (`/`, `:`, `\` -- every separator a real OS path
/// could treat specially -- replaced with `-`, truncated to 80 *characters*; Rust's own `chars()`
/// count matches Swift's `String.prefix` here since a sanitized title has already dropped anything
/// that could make a Unicode scalar vs. grapheme-cluster distinction matter for a dialog's own
/// suggested filename).
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | ':' | '\\' => '-',
            other => other,
        })
        .take(80)
        .collect()
}

fn export_session_inner<R: Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    project_id: &str,
    session_id: &str,
    options: &ExportOptions,
    destination: ExportDestination,
    picked: Option<std::path::PathBuf>,
) -> Result<WriteOutcome> {
    let session = load_full_session(state, project_id, session_id)?;
    let markdown = session_markdown::to_markdown(&session, options);

    match destination {
        ExportDestination::Clipboard => {
            app.clipboard()
                .write_text(markdown)
                .map_err(|e| AppError::io(e.to_string()))?;
            Ok(WriteOutcome::Written)
        }
        ExportDestination::File => {
            let Some(path) = picked else {
                return Ok(WriteOutcome::Cancelled);
            };
            std::fs::write(&path, markdown).map_err(|e| AppError::from_io(&e, &path))?;
            Ok(WriteOutcome::Written)
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn export_session(
    app: AppHandle,
    project_id: String,
    session_id: String,
    options: ExportOptions,
    destination: ExportDestination,
) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        // The dialog only needs a real session title when it is actually going to show one
        // (`Clipboard` never opens a save dialog at all); `Session::title()` isn't cheap enough to
        // call unconditionally just to build a filename nothing will use.
        let picked = match destination {
            ExportDestination::File => {
                let title = load_full_session(&state, &project_id, &session_id)
                    .map(|s| s.title())
                    .unwrap_or_default();
                pick_export_path(&app, &format!("{}.md", sanitize_filename(&title)), "md")
            }
            ExportDestination::Clipboard => None,
        };
        export_session_inner(&app, &state, &project_id, &session_id, &options, destination, picked)
    })
    .await
}

fn export_edit_patch_inner(
    state: &AppState,
    project_id: &str,
    session_id: &str,
    block_id: &str,
    picked: Option<std::path::PathBuf>,
) -> Result<WriteOutcome> {
    let Some(path) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let session = load_full_session(state, project_id, session_id)?;
    let block = find_tool_block(&session, block_id)?;
    let file_path = block
        .file_path
        .as_deref()
        .ok_or_else(|| AppError::not_found("this tool call has no file path"))?;
    let edit = block
        .edit
        .as_ref()
        .ok_or_else(|| AppError::invalid_input("this tool block is not an edit with both strings"))?;
    let patch = heimdall_core::export::patch::to_patch(&edit.old_string, &edit.new_string, file_path);
    std::fs::write(&path, patch).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(WriteOutcome::Written)
}

#[tauri::command]
#[specta::specta]
pub async fn export_edit_patch(
    app: AppHandle,
    project_id: String,
    session_id: String,
    block_id: String,
) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        // Same reasoning as `export_session`: the default filename (the edited file's own last
        // path component, `SessionExporter.presentPatchSavePanel`'s own convention) needs the
        // block resolved once just to name the dialog -- resolved again inside `_inner` rather
        // than threaded through, since a cancelled dialog should cost nothing more than that.
        let default_name = load_full_session(&state, &project_id, &session_id)
            .ok()
            .and_then(|s| find_tool_block(&s, &block_id).ok().cloned())
            .and_then(|b| b.file_path.clone())
            .and_then(|p| {
                std::path::Path::new(&p)
                    .file_name()
                    .map(|n| format!("{}.patch", n.to_string_lossy()))
            })
            .unwrap_or_else(|| "edit.patch".to_string());
        let picked = pick_export_path(&app, &default_name, "patch");
        export_edit_patch_inner(&state, &project_id, &session_id, &block_id, picked)
    })
    .await
}

fn export_analytics_csv_inner(
    state: &AppState,
    kind: CsvKind,
    picked: Option<std::path::PathBuf>,
) -> Result<WriteOutcome> {
    let Some(path) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let claude_home = &state.claude_home;
    let stats =
        heimdall_core::data::stats_cache::load_analytics(&claude_home.stats_cache_file(), &claude_home.projects_dir())
            .ok_or_else(|| AppError::not_found("no stats cache and nothing to compute from (no sessions yet)"))?;
    let csv = analytics_csv::export(&stats.stats, kind);
    std::fs::write(&path, csv).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(WriteOutcome::Written)
}

#[tauri::command]
#[specta::specta]
pub async fn export_analytics_csv(app: AppHandle, kind: CsvKind) -> Result<WriteOutcome> {
    blocking(move || {
        let picked = pick_export_path(&app, analytics_csv::suggested_file_name(kind), "csv");
        export_analytics_csv_inner(&app.state::<AppState>(), kind, picked)
    })
    .await
}

/// `PngKind` has no Swift equivalent filename convention to port (`presentImageSavePanel` took its
/// suggested name from its own caller, which the IPC contract's bare `save_png(kind, bytes)` has
/// no room for) -- a plain kebab-case name matching `analytics_csv::suggested_file_name`'s own
/// style for the other export kind in this file.
fn png_file_name(kind: PngKind) -> &'static str {
    match kind {
        PngKind::Analytics => "analytics.png",
        PngKind::EditDiff => "edit-diff.png",
    }
}

fn save_png_inner(bytes: &[u8], picked: Option<std::path::PathBuf>) -> Result<WriteOutcome> {
    heimdall_core::export::validate_png(bytes)?;
    let Some(path) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    std::fs::write(&path, bytes).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(WriteOutcome::Written)
}

#[tauri::command]
#[specta::specta]
pub async fn save_png(app: AppHandle, kind: PngKind, bytes: Vec<u8>) -> Result<WriteOutcome> {
    blocking(move || {
        let picked = pick_export_path(&app, png_file_name(kind), "png");
        save_png_inner(&bytes, picked)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;

    #[test]
    fn sanitize_filename_replaces_path_separators_and_truncates() {
        assert_eq!(sanitize_filename("fix: a/b\\c:d"), "fix- a-b-c-d");
        assert_eq!(sanitize_filename(&"x".repeat(100)).chars().count(), 80);
    }

    /// A fresh tempdir project with one session, one message carrying one tool block (an `Edit`
    /// with both strings, or a `Read` with none) -- never the shared fixture tree, since every
    /// test below either writes a file or would otherwise need a shape the real fixtures don't
    /// happen to carry (a tool block with `old_string`/`new_string` both present).
    fn state_with_session(project_id: &str, session_id: &str, tool_json: serde_json::Value) -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        let project_dir = root.join("projects").join(project_id);
        std::fs::create_dir_all(&project_dir).unwrap();
        let jsonl = format!(
            "{}\n{}",
            serde_json::json!({"type":"user","uuid":"u1","timestamp":"2026-01-01T00:00:00.000Z","isSidechain":false,"message":{"role":"user","content":"hi"}}),
            serde_json::json!({"type":"assistant","uuid":"a1","timestamp":"2026-01-01T00:00:01.000Z","isSidechain":false,"message":{"id":"msg1","role":"assistant","model":"claude-opus-4-6","content":[tool_json],"usage":{"input_tokens":1,"output_tokens":1}}}),
        );
        std::fs::write(project_dir.join(format!("{session_id}.jsonl")), jsonl).unwrap();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    fn block_id_of(state: &AppState, project_id: &str, session_id: &str) -> String {
        let session = load_full_session(state, project_id, session_id).unwrap();
        format!("{}:0", session.messages[1].id)
    }

    // MARK: - export_session (File destination only -- Clipboard would write to this machine's
    // real OS clipboard even under `tauri::test::MockRuntime`, since clipboard access isn't
    // virtualised by Tauri's own test harness the way window creation is; the same real-side-
    // effect boundary this session's own M3-53 entry already drew around `open_terminal_inner`).

    fn state_with_markdown_session() -> AppState {
        state_with_session(
            "proj",
            "11111111-1111-1111-1111-111111111111",
            serde_json::json!({"type":"text","text":"hello"}),
        )
    }

    #[test]
    fn export_session_no_path_picked_is_cancelled_and_writes_nothing() {
        let state = state_with_markdown_session();
        let options = ExportOptions::default();
        assert_eq!(
            export_session_inner(
                &app_handle(),
                &state,
                "proj",
                "11111111-1111-1111-1111-111111111111",
                &options,
                ExportDestination::File,
                None
            )
            .unwrap(),
            WriteOutcome::Cancelled
        );
    }

    #[test]
    fn export_session_confirmed_writes_real_markdown_to_the_picked_path() {
        let state = state_with_markdown_session();
        let out = state.claude_home.root().join("export.md");
        let options = ExportOptions::default();
        assert_eq!(
            export_session_inner(
                &app_handle(),
                &state,
                "proj",
                "11111111-1111-1111-1111-111111111111",
                &options,
                ExportDestination::File,
                Some(out.clone())
            )
            .unwrap(),
            WriteOutcome::Written
        );
        let written = std::fs::read_to_string(&out).unwrap();
        assert!(written.contains("hello"));
    }

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        tauri::test::mock_builder()
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .expect("build mock app")
            .handle()
            .clone()
    }

    // MARK: - export_edit_patch

    fn edit_tool(old: &str, new: &str) -> serde_json::Value {
        serde_json::json!({"type":"tool_use","id":"tu1","name":"Edit","input":{"file_path":"/abs/target.rs","old_string":old,"new_string":new}})
    }

    #[test]
    fn export_edit_patch_no_path_picked_is_cancelled() {
        let state = state_with_session("proj", "11111111-1111-1111-1111-111111111111", edit_tool("old", "new"));
        let block_id = block_id_of(&state, "proj", "11111111-1111-1111-1111-111111111111");
        assert_eq!(
            export_edit_patch_inner(&state, "proj", "11111111-1111-1111-1111-111111111111", &block_id, None).unwrap(),
            WriteOutcome::Cancelled
        );
    }

    #[test]
    fn export_edit_patch_confirmed_writes_a_real_unified_diff() {
        let state = state_with_session(
            "proj",
            "11111111-1111-1111-1111-111111111111",
            edit_tool("let a = 1;", "let a = 2;"),
        );
        let block_id = block_id_of(&state, "proj", "11111111-1111-1111-1111-111111111111");
        let out = state.claude_home.root().join("edit.patch");
        assert_eq!(
            export_edit_patch_inner(
                &state,
                "proj",
                "11111111-1111-1111-1111-111111111111",
                &block_id,
                Some(out.clone())
            )
            .unwrap(),
            WriteOutcome::Written
        );
        let patch = std::fs::read_to_string(&out).unwrap();
        assert!(patch.contains("-let a = 1;") && patch.contains("+let a = 2;"));
    }

    #[test]
    fn export_edit_patch_rejects_a_non_edit_tool_block() {
        let state = state_with_session(
            "proj",
            "11111111-1111-1111-1111-111111111111",
            serde_json::json!({"type":"tool_use","id":"tu1","name":"Read","input":{"file_path":"/abs/target.rs"}}),
        );
        let block_id = block_id_of(&state, "proj", "11111111-1111-1111-1111-111111111111");
        let out = state.claude_home.root().join("edit.patch");
        assert!(matches!(
            export_edit_patch_inner(
                &state,
                "proj",
                "11111111-1111-1111-1111-111111111111",
                &block_id,
                Some(out)
            )
            .unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    // MARK: - export_analytics_csv

    fn state_from_fixture(fixture_root: &str) -> AppState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures")
            .join(fixture_root);
        AppState {
            claude_home: ClaudeHome::new(root.join("claude-home"), root.join("claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn export_analytics_csv_no_path_picked_is_cancelled() {
        let state = state_from_fixture("screenshot");
        assert_eq!(
            export_analytics_csv_inner(&state, CsvKind::DailyActivity, None).unwrap(),
            WriteOutcome::Cancelled
        );
    }

    #[test]
    fn export_analytics_csv_confirmed_writes_a_real_csv_table() {
        // `fixtures/windows` has real transcripts, real UUID-named session files (`stats::
        // session_files` only recognises those -- `fixtures/screenshot`'s readable `conv-001`-style
        // names don't qualify, confirmed directly) and no `stats-cache.json`, so this exercises a
        // real recompute end to end, the same path `analytics.rs`'s own tests already confirmed
        // works against this fixture.
        let state = state_from_fixture("windows");
        let out = tempfile::tempdir().unwrap().keep().join("daily-activity.csv");
        assert_eq!(
            export_analytics_csv_inner(&state, CsvKind::DailyActivity, Some(out.clone())).unwrap(),
            WriteOutcome::Written
        );
        let csv = std::fs::read_to_string(&out).unwrap();
        assert!(csv.starts_with("Date,Messages,Sessions,Tool Calls\n"));
    }

    // MARK: - save_png

    const REAL_PNG_PREFIX: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];

    #[test]
    fn save_png_rejects_bytes_without_the_png_signature() {
        let out = tempfile::tempdir().unwrap().keep().join("x.png");
        assert!(matches!(
            save_png_inner(b"not a png", Some(out)).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn save_png_no_path_picked_is_cancelled() {
        let mut bytes = REAL_PNG_PREFIX.to_vec();
        bytes.extend_from_slice(b"rest of the file");
        assert_eq!(save_png_inner(&bytes, None).unwrap(), WriteOutcome::Cancelled);
    }

    #[test]
    fn save_png_confirmed_writes_the_real_bytes() {
        let mut bytes = REAL_PNG_PREFIX.to_vec();
        bytes.extend_from_slice(b"rest of the file");
        let out = tempfile::tempdir().unwrap().keep().join("x.png");
        assert_eq!(
            save_png_inner(&bytes, Some(out.clone())).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(std::fs::read(&out).unwrap(), bytes);
    }

    #[test]
    fn png_file_name_is_kebab_case_per_kind() {
        assert_eq!(png_file_name(PngKind::Analytics), "analytics.png");
        assert_eq!(png_file_name(PngKind::EditDiff), "edit-diff.png");
    }
}
