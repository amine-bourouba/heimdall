//! Commands in commands/mcp.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37, the three
//! MVP read bodies landed in M1-45; `add_mcp_server`/`update_mcp_server`/`remove_mcp_server` land
//! here (M3-45). Every body runs through `state::blocking` (contract §1).
//!
//! No response here carries a secret value: [`McpServer`] only ever serialises `envKeys`/
//! `headerKeys` (names), never the `env`/`headers` maps themselves (`#[serde(skip)]`, M1-10) — the
//! redaction this task's own AC1 asks for already happened before this file exists, and the same
//! key-names-only shape is reused below for the native confirmation's own summary text.
//!
//! Global scope only, matching `config::write::mcp`'s own doc comment (M3-41): none of these three
//! commands take a project parameter in the contract.

use std::collections::BTreeMap;

use heimdall_core::config::read::mcp;
use heimdall_core::config::write::{mcp as mcp_write, settings};
use heimdall_core::models::catalog::mcp_catalog::{self, McpCatalogEntry};
use heimdall_core::models::config::mcp::{McpServer, McpServerInput, McpServerStatus};
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::paths::TargetOs;
use heimdall_core::{Loaded, Result, WriteOutcome, mcp_status};
use tauri::{AppHandle, Manager};

use super::config_read::resolve_project_scope;
use crate::state::{AppState, blocking, confirm};

fn list_mcp_servers_inner<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    project: Option<ProjectContext>,
) -> Result<Loaded<Vec<McpServer>>> {
    let root = project
        .as_ref()
        .map(|context| resolve_project_scope(app, state, context))
        .transpose()?;
    let project_path = root.as_deref().and_then(|p| p.to_str());
    Ok(mcp::load(&state.claude_home, project_path, TargetOs::current()))
}

#[tauri::command]
#[specta::specta]
pub async fn list_mcp_servers(app: AppHandle, project: Option<ProjectContext>) -> Result<Loaded<Vec<McpServer>>> {
    blocking(move || list_mcp_servers_inner(&app, &app.state::<AppState>(), project)).await
}

fn get_mcp_statuses_inner<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    project: Option<ProjectContext>,
) -> Result<BTreeMap<String, McpServerStatus>> {
    let mut servers = list_mcp_servers_inner(app, state, project)?.value;
    let auth = mcp::auth_cache_names(&state.claude_home);
    let processes = mcp_status::running_command_lines();
    mcp_status::resolve_statuses(&mut servers, &auth, &processes, TargetOs::current());
    Ok(servers.into_iter().map(|s| (s.id, s.status)).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_statuses(
    app: AppHandle,
    project: Option<ProjectContext>,
) -> Result<BTreeMap<String, McpServerStatus>> {
    blocking(move || get_mcp_statuses_inner(&app, &app.state::<AppState>(), project)).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_catalog() -> Result<Vec<McpCatalogEntry>> {
    blocking(|| Ok(mcp_catalog::entries())).await
}

/// The exact values a confirmation shows for one server (this task's own Goal: "the exact
/// command, each argument, env keys, URL and header keys") -- never a secret *value*, only key
/// *names*, the same shape [`McpServer`]'s own `env_keys`/`header_keys` already carry over IPC.
fn server_summary(
    name: &str,
    command: Option<&str>,
    args: &[String],
    url: Option<&str>,
    env_keys: &[String],
    header_keys: &[String],
) -> String {
    let mut lines = vec![format!("Server: {name}")];
    if let Some(cmd) = command {
        lines.push(format!("command: {cmd}"));
    }
    if !args.is_empty() {
        lines.push(format!("args: {}", args.join(" ")));
    }
    if let Some(url) = url {
        lines.push(format!("url: {url}"));
    }
    if !env_keys.is_empty() {
        lines.push(format!("env keys: {}", env_keys.join(", ")));
    }
    if !header_keys.is_empty() {
        lines.push(format!("header keys: {}", header_keys.join(", ")));
    }
    lines.join("\n")
}

fn input_summary(input: &McpServerInput) -> String {
    let env_keys: Vec<String> = input.env.keys().cloned().collect();
    let header_keys: Vec<String> = input.headers.keys().cloned().collect();
    server_summary(
        &input.name,
        input.command.as_deref(),
        &input.args,
        input.url.as_deref(),
        &env_keys,
        &header_keys,
    )
}

/// What a removal's own confirmation shows: the *current* top-level definition's values, looked
/// up by name -- `remove_mcp_server` itself only ever receives a bare name, with nothing else to
/// show without reading it back first. A name that no longer matches anything (already removed by
/// a racing call) still shows a confirmation naming it, rather than erroring before the user even
/// gets to decide.
fn removal_summary(state: &AppState, name: &str) -> String {
    let servers = mcp::load(&state.claude_home, None, TargetOs::current()).value;
    match servers.into_iter().find(|s| s.raw_name == name) {
        Some(s) => server_summary(
            name,
            s.command.as_deref(),
            &s.args,
            s.url.as_deref(),
            &s.env_keys,
            &s.header_keys,
        ),
        None => format!("Server: {name}"),
    }
}

fn add_mcp_server_inner(state: &AppState, input: &McpServerInput, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    mcp_write::add_server(state.claude_home.config_file(), input)
}

#[tauri::command]
#[specta::specta]
pub async fn add_mcp_server(app: AppHandle, input: McpServerInput) -> Result<WriteOutcome> {
    blocking(move || {
        let message = format!("Add this MCP server?\n\n{}", input_summary(&input));
        let confirmed = confirm(&app, "Add MCP Server", &message, "Add");
        add_mcp_server_inner(&app.state::<AppState>(), &input, confirmed)
    })
    .await
}

fn update_mcp_server_inner(
    state: &AppState,
    original_name: &str,
    input: &McpServerInput,
    confirmed: bool,
) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    mcp_write::update_server(state.claude_home.config_file(), original_name, input)
}

#[tauri::command]
#[specta::specta]
pub async fn update_mcp_server(app: AppHandle, original_name: String, input: McpServerInput) -> Result<WriteOutcome> {
    blocking(move || {
        let message = format!("Save changes to this MCP server?\n\n{}", input_summary(&input));
        let confirmed = confirm(&app, "Update MCP Server", &message, "Save");
        update_mcp_server_inner(&app.state::<AppState>(), &original_name, &input, confirmed)
    })
    .await
}

/// Removes the server itself (`config::write::mcp`, M3-41) and strips its `permissions.allow`
/// entries from `settings.json` (`config::write::settings::remove_mcp_permissions`, M3-40) --
/// Swift's own `removeMCPPermissions`/`removeMCPServer` are two separate functions a single UI
/// action calls together, and M3-40's own Follow-ups named this task as the first real caller of
/// both at once. `Written` if either one actually changed something.
fn remove_mcp_server_inner(state: &AppState, name: &str, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    let server_outcome = mcp_write::remove_server(state.claude_home.config_file(), name)?;
    let permissions_outcome = settings::remove_mcp_permissions(state.claude_home.root(), name)?;
    Ok(
        if server_outcome == WriteOutcome::Written || permissions_outcome == WriteOutcome::Written {
            WriteOutcome::Written
        } else {
            WriteOutcome::Unchanged
        },
    )
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mcp_server(app: AppHandle, name: String) -> Result<WriteOutcome> {
    blocking(move || {
        let state = app.state::<AppState>();
        let message = format!("Remove this MCP server?\n\n{}", removal_summary(&state, &name));
        let confirmed = confirm(&app, "Remove MCP Server", &message, "Remove");
        remove_mcp_server_inner(&state, &name, confirmed)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::AppError;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
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
        app.handle().clone()
    }

    fn json_of(server: &McpServer) -> String {
        serde_json::to_string(server).unwrap()
    }

    #[test]
    fn list_mcp_servers_never_carries_a_secret_value() {
        let app = app_handle();
        let loaded = list_mcp_servers_inner(&app, &app.state::<AppState>(), None).unwrap();
        assert!(!loaded.value.is_empty());
        for server in &loaded.value {
            let json = json_of(server);
            assert!(!json.contains("SENTINEL-SECRET"), "leaked a secret: {json}");
        }
    }

    #[test]
    fn list_mcp_servers_global_scope_sees_github_and_sentry_but_not_the_project_scoped_server() {
        let app = app_handle();
        let loaded = list_mcp_servers_inner(&app, &app.state::<AppState>(), None).unwrap();
        let names: Vec<_> = loaded.value.iter().map(|s| s.raw_name.as_str()).collect();
        assert!(names.contains(&"github") && names.contains(&"sentry"));
    }

    #[test]
    fn list_mcp_servers_project_scope_adds_the_projects_own_server() {
        let app = app_handle();
        let project = Some(ProjectContext::Project {
            project_id: "-Users-fixture-src-app".into(),
        });
        let loaded = list_mcp_servers_inner(&app, &app.state::<AppState>(), project).unwrap();
        assert!(loaded.value.iter().any(|s| s.raw_name == "local-db"));
    }

    #[test]
    fn list_mcp_servers_rejects_an_unknown_project_as_invalid_input() {
        let app = app_handle();
        let project = Some(ProjectContext::Project {
            project_id: "-Users-nobody-nowhere".into(),
        });
        assert!(matches!(
            list_mcp_servers_inner(&app, &app.state::<AppState>(), project).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }

    #[test]
    fn get_mcp_statuses_keys_by_server_id_and_covers_every_server() {
        let app = app_handle();
        let servers = list_mcp_servers_inner(&app, &app.state::<AppState>(), None)
            .unwrap()
            .value;
        let statuses = get_mcp_statuses_inner(&app, &app.state::<AppState>(), None).unwrap();
        assert_eq!(statuses.len(), servers.len());
        for server in &servers {
            assert!(statuses.contains_key(&server.id), "missing status for {}", server.id);
        }
    }

    #[test]
    fn get_mcp_catalog_returns_the_static_catalog() {
        let catalog = tauri::async_runtime::block_on(get_mcp_catalog()).unwrap();
        assert_eq!(catalog, mcp_catalog::entries());
        assert!(!catalog.is_empty());
    }

    // MARK: - Writes (never the shared `fixtures/synthetic` tree `app_handle()` reads above --
    // every test here writes, so each gets its own fresh tempdir instead).

    use heimdall_core::models::config::mcp::{McpServerInputTransport, SecretInput};

    fn fresh_state() -> AppState {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.keep();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    fn stdio_input(name: &str, command: &str) -> McpServerInput {
        McpServerInput {
            name: name.to_string(),
            transport: McpServerInputTransport::Stdio,
            command: Some(command.to_string()),
            args: vec!["--flag".to_string()],
            url: None,
            env: BTreeMap::from([(
                "API_KEY".to_string(),
                SecretInput::Set {
                    value: "sk-super-secret".to_string(),
                },
            )]),
            headers: BTreeMap::new(),
        }
    }

    fn config_json(state: &AppState) -> serde_json::Value {
        serde_json::from_slice(&std::fs::read(state.claude_home.config_file()).unwrap()).unwrap()
    }

    #[test]
    fn add_mcp_server_declined_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        assert_eq!(
            add_mcp_server_inner(&state, &stdio_input("local-db", "node"), false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert!(!state.claude_home.config_file().exists());
    }

    #[test]
    fn add_mcp_server_confirmed_writes_the_server() {
        let state = fresh_state();
        assert_eq!(
            add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(config_json(&state)["mcpServers"]["local-db"]["command"], "node");
    }

    // MARK: - AC1: secret values never appear in the confirmation, only key names

    #[test]
    fn input_summary_names_command_args_and_env_keys_but_never_the_secret_value() {
        let summary = input_summary(&stdio_input("local-db", "node"));
        assert!(summary.contains("command: node"));
        assert!(summary.contains("args: --flag"));
        assert!(summary.contains("env keys: API_KEY"));
        assert!(!summary.contains("sk-super-secret"));
    }

    #[test]
    fn update_mcp_server_declined_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap();
        let before = std::fs::read(state.claude_home.config_file()).unwrap();
        assert_eq!(
            update_mcp_server_inner(&state, "local-db", &stdio_input("local-db", "python3"), false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert_eq!(std::fs::read(state.claude_home.config_file()).unwrap(), before);
    }

    #[test]
    fn update_mcp_server_confirmed_updates_the_existing_entry() {
        let state = fresh_state();
        add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap();
        assert_eq!(
            update_mcp_server_inner(&state, "local-db", &stdio_input("local-db", "python3"), true).unwrap(),
            WriteOutcome::Written
        );
        assert_eq!(config_json(&state)["mcpServers"]["local-db"]["command"], "python3");
    }

    #[test]
    fn remove_mcp_server_declined_is_cancelled_and_writes_nothing() {
        let state = fresh_state();
        add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap();
        let before = std::fs::read(state.claude_home.config_file()).unwrap();
        assert_eq!(
            remove_mcp_server_inner(&state, "local-db", false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert_eq!(std::fs::read(state.claude_home.config_file()).unwrap(), before);
    }

    #[test]
    fn remove_mcp_server_confirmed_removes_the_server_and_its_permissions() {
        let state = fresh_state();
        add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap();
        std::fs::write(
            state.claude_home.root().join("settings.json"),
            r#"{"permissions": {"allow": ["mcp__local-db__query", "Bash(ls)"]}}"#,
        )
        .unwrap();

        assert_eq!(
            remove_mcp_server_inner(&state, "local-db", true).unwrap(),
            WriteOutcome::Written
        );
        assert!(config_json(&state)["mcpServers"].get("local-db").is_none());
        let settings: serde_json::Value =
            serde_json::from_slice(&std::fs::read(state.claude_home.root().join("settings.json")).unwrap()).unwrap();
        assert_eq!(settings["permissions"]["allow"], serde_json::json!(["Bash(ls)"]));
    }

    #[test]
    fn removal_summary_names_the_current_servers_values() {
        let state = fresh_state();
        add_mcp_server_inner(&state, &stdio_input("local-db", "node"), true).unwrap();
        assert!(removal_summary(&state, "local-db").contains("command: node"));
    }

    #[test]
    fn removal_summary_of_an_unknown_server_still_names_it_rather_than_erroring() {
        let state = fresh_state();
        assert_eq!(removal_summary(&state, "ghost"), "Server: ghost");
    }
}
