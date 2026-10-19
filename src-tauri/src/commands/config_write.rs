//! Commands in commands/config_write.rs (docs/ipc-contract.md §6). Typed stubs landed in M1-37;
//! real bodies land here (M3-44). Every body runs through `state::blocking` (contract §1).
//!
//! `delete_config_item`'s contract argument name `ref` is spelled `config_ref` here (see config_read.rs).
//!
//! ADR 0010 control 6 ("writing a hook command or URL... first shows a native (Rust-side)
//! confirmation with the exact values") applies to `save_hook` and `import_hooks` only, per this
//! task's own AC1 -- neither `delete_hook` nor any of the plugin/model/template/delete commands
//! below need one. Every command that shows *any* dialog (confirm or file picker) follows the same
//! "inject the dialog's own result" shape `prefs.rs` already established: the thin
//! `#[tauri::command]` wrapper calls the real native dialog, and passes its plain result into a
//! separately-testable `*_inner` function that never touches `AppHandle` or a dialog at all (AC3).

use std::path::{Path, PathBuf};

use heimdall_core::config::read::hooks;
use heimdall_core::config::write::{files, settings};
use heimdall_core::models::config::hook::{HookGroupInput, HookHandlerInput, HookHandlerType};
use heimdall_core::models::config::scope::{ConfigKind, ConfigRef, ConfigScope, ProjectContext};
use heimdall_core::models::config::sub_agent::AgentInput;
use heimdall_core::{AppError, Result, WriteOutcome};
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking, confirm, pick_export_path, pick_import_path};

use super::config_read::resolve_project_scope;

/// The directory `config::write::files`'s own functions call `claude_dir` -- `~/.claude` for
/// global, `<project>/.claude` for project-scoped, matching `config::read`'s own `scopes()`
/// convention (`config/read/mod.rs`) and `config::write::settings::resolve_scoped_path`'s own
/// `root.join(".claude")` for a project.
fn write_dir(claude_home_root: &Path, project_root: Option<&Path>) -> PathBuf {
    project_root.map_or_else(|| claude_home_root.to_path_buf(), |r| r.join(".claude"))
}

fn scope_of(project_root: &Option<PathBuf>) -> ConfigScope {
    if project_root.is_some() {
        ConfigScope::Project
    } else {
        ConfigScope::Global
    }
}

// MARK: - Plugins

fn set_plugin_enabled_inner(state: &AppState, key: &str, enabled: bool) -> Result<WriteOutcome> {
    settings::set_plugin_enabled(state.claude_home.root(), key, enabled)
}

#[tauri::command]
#[specta::specta]
pub async fn set_plugin_enabled(app: AppHandle, key: String, enabled: bool) -> Result<WriteOutcome> {
    blocking(move || set_plugin_enabled_inner(&app.state::<AppState>(), &key, enabled)).await
}

fn remove_plugin_inner(state: &AppState, key: &str) -> Result<WriteOutcome> {
    settings::remove_plugin(state.claude_home.root(), key)
}

#[tauri::command]
#[specta::specta]
pub async fn remove_plugin(app: AppHandle, key: String) -> Result<WriteOutcome> {
    blocking(move || remove_plugin_inner(&app.state::<AppState>(), &key)).await
}

// MARK: - Model

fn set_default_model_inner(state: &AppState, model: &str) -> Result<WriteOutcome> {
    settings::set_default_model(state.claude_home.root(), model)
}

#[tauri::command]
#[specta::specta]
pub async fn set_default_model(app: AppHandle, model: String) -> Result<WriteOutcome> {
    blocking(move || set_default_model_inner(&app.state::<AppState>(), &model)).await
}

#[tauri::command]
#[specta::specta]
pub async fn set_project_model(app: AppHandle, project: ProjectContext, model: Option<String>) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let project_root = resolve_project_scope(&app, &state, &project)?;
        settings::set_project_model(&project_root, model.as_deref())
    })
    .await
}

// MARK: - Hooks

/// The value a save/import confirmation shows for one handler -- its command or URL, the one
/// field ADR 0010 control 6 names explicitly.
fn handler_value_summary(h: &HookHandlerInput) -> String {
    match h.handler_type {
        HookHandlerType::Command => format!("command: {}", h.command.as_deref().unwrap_or("(none)")),
        HookHandlerType::Http => format!("url: {}", h.url.as_deref().unwrap_or("(none)")),
    }
}

fn hook_save_message(event: &str, group: &HookGroupInput) -> String {
    let values: Vec<String> = group.handlers.iter().map(handler_value_summary).collect();
    format!("Save this {event} hook?\n\n{}", values.join("\n"))
}

fn save_hook_inner(
    state: &AppState,
    scope: ConfigScope,
    project_root: Option<&Path>,
    event: &str,
    group: &HookGroupInput,
    existing_index: Option<usize>,
    confirmed: bool,
) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    settings::save_hook(
        state.claude_home.root(),
        scope,
        project_root,
        event,
        group,
        existing_index,
    )
}

#[tauri::command]
#[specta::specta]
pub async fn save_hook(
    app: AppHandle,
    project: Option<ProjectContext>,
    event: String,
    group: HookGroupInput,
    existing_index: Option<u32>,
) -> Result<WriteOutcome> {
    blocking(move || {
        let confirmed = confirm(&app, "Confirm Save", &hook_save_message(&event, &group), "Save");
        let state = app.state::<AppState>();
        let project_root = project.map(|c| resolve_project_scope(&app, &state, &c)).transpose()?;
        save_hook_inner(
            &state,
            scope_of(&project_root),
            project_root.as_deref(),
            &event,
            &group,
            existing_index.map(|i| i as usize),
            confirmed,
        )
    })
    .await
}

fn delete_hook_inner(
    state: &AppState,
    scope: ConfigScope,
    project_root: Option<&Path>,
    event: &str,
    index: usize,
) -> Result<WriteOutcome> {
    settings::delete_hook(state.claude_home.root(), scope, project_root, event, index)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_hook(
    app: AppHandle,
    project: Option<ProjectContext>,
    event: String,
    index: u32,
) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let project_root = project.map(|c| resolve_project_scope(&app, &state, &c)).transpose()?;
        delete_hook_inner(
            &state,
            scope_of(&project_root),
            project_root.as_deref(),
            &event,
            index as usize,
        )
    })
    .await
}

/// What `import_hooks`'s own confirmation shows: every handler the picked file would add, by
/// event. Reads the same `{"hooks": {<event>: [<group>, ...]}}` wrapper `settings::import_hooks`
/// does, but only to describe it -- never mutates anything, so a user can see exactly what an
/// untrusted file contains before any of it reaches `settings.json`.
fn hook_import_preview(bytes: &[u8]) -> Result<String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|e| AppError::parse(format!("not a valid hooks export: {e}")))?;
    let mut lines = Vec::new();
    if let Some(hooks) = value.get("hooks").and_then(Value::as_object) {
        for (event, groups) in hooks {
            for group in groups.as_array().into_iter().flatten() {
                for handler in group.get("hooks").and_then(Value::as_array).into_iter().flatten() {
                    let value = handler
                        .get("command")
                        .or_else(|| handler.get("url"))
                        .and_then(Value::as_str)
                        .unwrap_or("?");
                    lines.push(format!("{event}: {value}"));
                }
            }
        }
    }
    Ok(if lines.is_empty() {
        "This file has no hooks to import.".to_string()
    } else {
        format!("Import {} hook handler(s)?\n\n{}", lines.len(), lines.join("\n"))
    })
}

fn import_hooks_inner(
    state: &AppState,
    scope: ConfigScope,
    project_root: Option<&Path>,
    json_bytes: Option<&[u8]>,
    confirmed: bool,
) -> Result<WriteOutcome> {
    let Some(bytes) = json_bytes else {
        return Ok(WriteOutcome::Cancelled);
    };
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    let count = settings::import_hooks(state.claude_home.root(), scope, project_root, bytes)?;
    Ok(if count > 0 {
        WriteOutcome::Written
    } else {
        WriteOutcome::Unchanged
    })
}

#[tauri::command]
#[specta::specta]
pub async fn import_hooks(app: AppHandle, project: Option<ProjectContext>) -> Result<WriteOutcome> {
    blocking(move || {
        let Some(path) = pick_import_path(&app, "json") else {
            return Ok(WriteOutcome::Cancelled);
        };
        let bytes = std::fs::read(&path).map_err(|e| AppError::from_io(&e, &path))?;
        let preview = hook_import_preview(&bytes)?;
        let confirmed = confirm(&app, "Import Hooks", &preview, "Import");
        let state = app.state::<AppState>();
        let project_root = project.map(|c| resolve_project_scope(&app, &state, &c)).transpose()?;
        import_hooks_inner(
            &state,
            scope_of(&project_root),
            project_root.as_deref(),
            Some(&bytes),
            confirmed,
        )
    })
    .await
}

fn export_hooks_inner(state: &AppState, project_root: Option<&Path>, picked: Option<PathBuf>) -> Result<WriteOutcome> {
    let Some(path) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let entries = hooks::load(state.claude_home.root(), project_root).value;
    let bytes = settings::export_hooks(&entries);
    std::fs::write(&path, &bytes).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(WriteOutcome::Written)
}

#[tauri::command]
#[specta::specta]
pub async fn export_hooks(app: AppHandle, project: Option<ProjectContext>) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let project_root = project.map(|c| resolve_project_scope(&app, &state, &c)).transpose()?;
        let picked = pick_export_path(&app, "hooks.json", "json");
        export_hooks_inner(&state, project_root.as_deref(), picked)
    })
    .await
}

// MARK: - Agents (global scope only -- `config::write::files`'s own doc comment: no project
// parameter anywhere in the Swift source this ported from).

fn save_agent_inner(state: &AppState, input: &AgentInput, original_slug: Option<&str>) -> Result<WriteOutcome> {
    files::save_agent(state.claude_home.root(), input, original_slug)
}

#[tauri::command]
#[specta::specta]
pub async fn save_agent(app: AppHandle, input: AgentInput, original_slug: Option<String>) -> Result<WriteOutcome> {
    blocking(move || save_agent_inner(&app.state::<AppState>(), &input, original_slug.as_deref())).await
}

fn import_agent_inner(state: &AppState, json_bytes: Option<&[u8]>) -> Result<WriteOutcome> {
    let Some(bytes) = json_bytes else {
        return Ok(WriteOutcome::Cancelled);
    };
    files::import_agent_json(state.claude_home.root(), bytes)
}

#[tauri::command]
#[specta::specta]
pub async fn import_agent(app: AppHandle) -> Result<WriteOutcome> {
    blocking(move || {
        let Some(path) = pick_import_path(&app, "json") else {
            return Ok(WriteOutcome::Cancelled);
        };
        let bytes = std::fs::read(&path).map_err(|e| AppError::from_io(&e, &path))?;
        import_agent_inner(&app.state::<AppState>(), Some(&bytes))
    })
    .await
}

fn export_agent_inner(state: &AppState, slug: &str, picked: Option<PathBuf>) -> Result<WriteOutcome> {
    let Some(path) = picked else {
        return Ok(WriteOutcome::Cancelled);
    };
    let bytes = files::export_agent_json(state.claude_home.root(), slug)?;
    std::fs::write(&path, &bytes).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(WriteOutcome::Written)
}

#[tauri::command]
#[specta::specta]
pub async fn export_agent(app: AppHandle, slug: String) -> Result<WriteOutcome> {
    blocking(move || {
        let picked = pick_export_path(&app, &format!("{slug}.json"), "json");
        export_agent_inner(&app.state::<AppState>(), &slug, picked)
    })
    .await
}

// MARK: - Templates / generic delete

fn create_config_template_inner(state: &AppState, kind: ConfigKind, project_root: Option<&Path>) -> Result<ConfigRef> {
    files::create_template(&write_dir(state.claude_home.root(), project_root), kind)
}

#[tauri::command]
#[specta::specta]
pub async fn create_config_template(
    app: AppHandle,
    kind: ConfigKind,
    project: Option<ProjectContext>,
) -> Result<ConfigRef> {
    blocking(move || {
        let state = app.state::<AppState>();
        let project_root = project.map(|c| resolve_project_scope(&app, &state, &c)).transpose()?;
        create_config_template_inner(&state, kind, project_root.as_deref())
    })
    .await
}

fn delete_config_item_inner(
    state: &AppState,
    config_ref: &ConfigRef,
    project_root: Option<&Path>,
) -> Result<WriteOutcome> {
    files::delete_item(&write_dir(state.claude_home.root(), project_root), config_ref)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_config_item(app: AppHandle, config_ref: ConfigRef) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let project_root = config_ref
            .project
            .clone()
            .map(|c| resolve_project_scope(&app, &state, &c))
            .transpose()?;
        delete_config_item_inner(&state, &config_ref, project_root.as_deref())
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::models::config::hook::HookEntry;
    use heimdall_core::models::config::sub_agent::AgentColor;
    use heimdall_core::paths::ClaudeHome;
    use tauri::test::{mock_builder, mock_context, noop_assets};

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

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        app().handle().clone()
    }

    /// A fresh tempdir `AppState` -- never the shared `fixtures/synthetic` tree `app()` reads,
    /// since every test below writes.
    fn fresh_state() -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    fn read(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    // MARK: - Plugins / model (no dialog at all)

    #[test]
    fn set_plugin_enabled_writes_settings_json() {
        let state = fresh_state();
        set_plugin_enabled_inner(&state, "a@b", true).unwrap();
        assert_eq!(
            read(&state.claude_home.root().join("settings.json"))["enabledPlugins"],
            serde_json::json!({"a@b": true})
        );
    }

    #[test]
    fn remove_plugin_drops_the_key() {
        let state = fresh_state();
        set_plugin_enabled_inner(&state, "a@b", true).unwrap();
        remove_plugin_inner(&state, "a@b").unwrap();
        assert!(
            read(&state.claude_home.root().join("settings.json"))["enabledPlugins"]
                .as_object()
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn set_default_model_writes_the_model_verbatim() {
        let state = fresh_state();
        set_default_model_inner(&state, "opus").unwrap();
        assert_eq!(read(&state.claude_home.root().join("settings.json"))["model"], "opus");
    }

    // MARK: - save_hook / delete_hook (save declines without an injected real dialog)

    fn group(command: &str) -> HookGroupInput {
        HookGroupInput {
            matcher: None,
            handlers: vec![HookHandlerInput {
                handler_type: HookHandlerType::Command,
                command: Some(command.to_string()),
                url: None,
                timeout: None,
                status_message: None,
            }],
        }
    }

    #[test]
    fn save_hook_declined_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        let g = group("a.sh");
        assert_eq!(
            save_hook_inner(&state, ConfigScope::Global, None, "PreToolUse", &g, None, false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert!(!state.claude_home.root().join("settings.json").exists());
    }

    #[test]
    fn save_hook_confirmed_writes_the_group() {
        let state = fresh_state();
        let g = group("a.sh");
        assert_eq!(
            save_hook_inner(&state, ConfigScope::Global, None, "PreToolUse", &g, None, true).unwrap(),
            WriteOutcome::Written
        );
        let v = read(&state.claude_home.root().join("settings.json"));
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "a.sh");
    }

    #[test]
    fn hook_save_message_names_the_handlers_exact_value() {
        let msg = hook_save_message("PreToolUse", &group("danger.sh"));
        assert!(msg.contains("command: danger.sh"));
    }

    #[test]
    fn delete_hook_removes_the_group() {
        let state = fresh_state();
        save_hook_inner(
            &state,
            ConfigScope::Global,
            None,
            "PreToolUse",
            &group("a.sh"),
            None,
            true,
        )
        .unwrap();
        delete_hook_inner(&state, ConfigScope::Global, None, "PreToolUse", 0).unwrap();
        assert!(
            read(&state.claude_home.root().join("settings.json"))
                .get("hooks")
                .is_none()
        );
    }

    // MARK: - import_hooks / export_hooks (both dialog steps injected)

    #[test]
    fn import_hooks_no_file_picked_is_cancelled() {
        let state = fresh_state();
        assert_eq!(
            import_hooks_inner(&state, ConfigScope::Global, None, None, true).unwrap(),
            WriteOutcome::Cancelled
        );
    }

    #[test]
    fn import_hooks_declined_after_the_values_preview_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        let bytes = br#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "a.sh"}]}]}}"#;
        assert_eq!(
            import_hooks_inner(&state, ConfigScope::Global, None, Some(bytes), false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert!(!state.claude_home.root().join("settings.json").exists());
    }

    #[test]
    fn import_hooks_confirmed_imports_the_handlers() {
        let state = fresh_state();
        let bytes = br#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "a.sh"}]}]}}"#;
        assert_eq!(
            import_hooks_inner(&state, ConfigScope::Global, None, Some(bytes), true).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(
            read(&state.claude_home.root().join("settings.json"))["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "a.sh"
        );
    }

    #[test]
    fn hook_import_preview_names_every_handlers_exact_value() {
        let bytes = br#"{"hooks": {"Stop": [{"hooks": [{"type": "http", "url": "https://evil.example"}]}]}}"#;
        assert!(hook_import_preview(bytes).unwrap().contains("https://evil.example"));
    }

    #[test]
    fn hook_import_preview_rejects_malformed_json() {
        assert!(matches!(
            hook_import_preview(b"not json").unwrap_err(),
            AppError::Parse { .. }
        ));
    }

    #[test]
    fn export_hooks_no_path_picked_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        assert_eq!(export_hooks_inner(&state, None, None).unwrap(), WriteOutcome::Cancelled);
    }

    #[test]
    fn export_hooks_writes_every_global_entry_to_the_picked_path() {
        let state = fresh_state();
        save_hook_inner(
            &state,
            ConfigScope::Global,
            None,
            "PreToolUse",
            &group("a.sh"),
            None,
            true,
        )
        .unwrap();
        let out = state.claude_home.root().join("exported.json");
        assert_eq!(
            export_hooks_inner(&state, None, Some(out.clone())).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(read(&out)["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "a.sh");
    }

    #[test]
    fn export_then_import_round_trips_through_real_command_layer_functions() {
        let entries = vec![HookEntry {
            event: "PreToolUse".to_string(),
            matcher_group_index: 0,
            matcher: None,
            handlers: vec![
                heimdall_core::models::config::hook::HookHandler::from_json(
                    serde_json::json!({"type": "command", "command": "round-trip.sh"})
                        .as_object()
                        .unwrap(),
                )
                .unwrap(),
            ],
            scope: ConfigScope::Global,
        }];
        let bytes = settings::export_hooks(&entries);
        let state = fresh_state();
        import_hooks_inner(&state, ConfigScope::Global, None, Some(&bytes), true).unwrap();
        assert_eq!(
            read(&state.claude_home.root().join("settings.json"))["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "round-trip.sh"
        );
    }

    // MARK: - Agents

    fn agent_input(name: &str) -> AgentInput {
        AgentInput {
            name: name.to_string(),
            icon: "person.crop.circle".to_string(),
            description: "A test agent".to_string(),
            tools: vec!["Read".to_string()],
            model: Some("sonnet".to_string()),
            color: Some(AgentColor::Blue),
            prompt: Some("Do the thing.".to_string()),
            memory: None,
        }
    }

    #[test]
    fn save_agent_writes_a_new_agent_file() {
        let state = fresh_state();
        assert_eq!(
            save_agent_inner(&state, &agent_input("Reviewer"), None).unwrap(),
            WriteOutcome::Written
        );
        assert!(state.claude_home.root().join("agents").join("reviewer.md").exists());
    }

    #[test]
    fn import_agent_no_file_picked_is_cancelled() {
        let state = fresh_state();
        assert_eq!(import_agent_inner(&state, None).unwrap(), WriteOutcome::Cancelled);
    }

    #[test]
    fn import_agent_reads_a_real_exported_agent() {
        let source = fresh_state();
        save_agent_inner(&source, &agent_input("Reviewer"), None).unwrap();
        let bytes = files::export_agent_json(source.claude_home.root(), "reviewer").unwrap();

        let target = fresh_state();
        assert_eq!(
            import_agent_inner(&target, Some(&bytes)).unwrap(),
            WriteOutcome::Written
        );
        assert!(target.claude_home.root().join("agents").join("reviewer.md").exists());
    }

    #[test]
    fn export_agent_no_path_picked_is_cancelled() {
        let state = fresh_state();
        save_agent_inner(&state, &agent_input("Reviewer"), None).unwrap();
        assert_eq!(
            export_agent_inner(&state, "reviewer", None).unwrap(),
            WriteOutcome::Cancelled
        );
    }

    #[test]
    fn export_agent_writes_the_json_to_the_picked_path() {
        let state = fresh_state();
        save_agent_inner(&state, &agent_input("Reviewer"), None).unwrap();
        let out = state.claude_home.root().join("reviewer-export.json");
        assert_eq!(
            export_agent_inner(&state, "reviewer", Some(out.clone())).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(read(&out)["name"], "Reviewer");
    }

    // MARK: - Templates / delete (need `resolve_project_scope`'s real `AppHandle`, like config_read.rs)

    #[test]
    fn create_config_template_global_scope_lands_under_claude_home() {
        let state = fresh_state();
        let config_ref = create_config_template_inner(&state, ConfigKind::Agent, None).unwrap();
        assert_eq!(config_ref.slug, "new-agent");
        assert!(state.claude_home.root().join("agents").join("new-agent.md").exists());
    }

    #[test]
    fn delete_config_item_removes_the_created_template() {
        let state = fresh_state();
        let config_ref = create_config_template_inner(&state, ConfigKind::Command, None).unwrap();
        assert_eq!(
            delete_config_item_inner(&state, &config_ref, None).unwrap(),
            WriteOutcome::Written
        );
        assert!(
            !state
                .claude_home
                .root()
                .join("commands")
                .join("new-command.md")
                .exists()
        );
    }

    #[test]
    fn delete_config_item_unknown_slug_is_unchanged() {
        let state = fresh_state();
        let config_ref = ConfigRef {
            kind: ConfigKind::Command,
            project: None,
            slug: "nonexistent".to_string(),
        };
        assert_eq!(
            delete_config_item_inner(&state, &config_ref, None).unwrap(),
            WriteOutcome::Unchanged
        );
    }

    #[test]
    fn set_project_model_writes_the_projects_own_settings_file() {
        // `PickedFolder` + a real tempdir, never `Project { project_id }` against the shared
        // `fixtures/synthetic` tree: that project's own `sessions-index.json` decodes to
        // `/Users/fixture/src/app`, which has no drive letter and so is not `is_absolute()` on
        // Windows -- `set_project_model` would resolve and write it against this machine's own
        // current drive root for real, well outside any tempdir. Caught before ever running it.
        let app = app_handle();
        let state = app.state::<AppState>();
        let project_dir = tempfile::tempdir().unwrap();
        crate::prefs::pick_config_project(&app, Some(project_dir.path().display().to_string())).unwrap();
        let project_root = resolve_project_scope(&app, &state, &ProjectContext::PickedFolder).unwrap();
        settings::set_project_model(&project_root, Some("sonnet")).unwrap();
        assert_eq!(
            read(&project_root.join(".claude").join("settings.json"))["model"],
            "sonnet"
        );
    }
}
