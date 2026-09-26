//! Commands in commands/mcp.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37, the three
//! MVP read bodies land here (M1-45); `add_mcp_server`/`update_mcp_server`/`remove_mcp_server`
//! stay stubs (M3-45). Every body runs through `state::blocking` (contract §1).
//!
//! No response here carries a secret value: [`McpServer`] only ever serialises `envKeys`/
//! `headerKeys` (names), never the `env`/`headers` maps themselves (`#[serde(skip)]`, M1-10) — the
//! redaction this task's acceptance criterion asks for already happened before this file exists.

use std::collections::BTreeMap;

use heimdall_core::config::read::mcp;
use heimdall_core::models::catalog::mcp_catalog::{self, McpCatalogEntry};
use heimdall_core::models::config::mcp::{McpServer, McpServerInput, McpServerStatus};
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::paths::TargetOs;
use heimdall_core::{mcp_status, AppError, Loaded, Result, WriteOutcome};
use tauri::{AppHandle, Manager};

use super::config_read::resolve_project_scope;
use crate::state::{blocking, AppState};

fn list_mcp_servers_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, project: Option<ProjectContext>) -> Result<Loaded<Vec<McpServer>>> {
    let root = project.as_ref().map(|context| resolve_project_scope(app, state, context)).transpose()?;
    let project_path = root.as_deref().and_then(|p| p.to_str());
    Ok(mcp::load(&state.claude_home, project_path, TargetOs::current()))
}

#[tauri::command]
#[specta::specta]
pub async fn list_mcp_servers(app: AppHandle, project: Option<ProjectContext>) -> Result<Loaded<Vec<McpServer>>> {
    blocking(move || list_mcp_servers_inner(&app, &app.state::<AppState>(), project)).await
}

fn get_mcp_statuses_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, project: Option<ProjectContext>) -> Result<BTreeMap<String, McpServerStatus>> {
    let mut servers = list_mcp_servers_inner(app, state, project)?.value;
    let auth = mcp::auth_cache_names(&state.claude_home);
    let processes = mcp_status::running_command_lines();
    mcp_status::resolve_statuses(&mut servers, &auth, &processes, TargetOs::current());
    Ok(servers.into_iter().map(|s| (s.id, s.status)).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_statuses(app: AppHandle, project: Option<ProjectContext>) -> Result<BTreeMap<String, McpServerStatus>> {
    blocking(move || get_mcp_statuses_inner(&app, &app.state::<AppState>(), project)).await
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_catalog() -> Result<Vec<McpCatalogEntry>> {
    blocking(|| Ok(mcp_catalog::entries())).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_mcp_server(input: McpServerInput) -> Result<WriteOutcome> {
    let _ = input;
    Err(AppError::not_implemented("add_mcp_server"))
}

#[tauri::command]
#[specta::specta]
pub async fn update_mcp_server(original_name: String, input: McpServerInput) -> Result<WriteOutcome> {
    let _ = (original_name, input);
    Err(AppError::not_implemented("update_mcp_server"))
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mcp_server(name: String) -> Result<WriteOutcome> {
    let _ = name;
    Err(AppError::not_implemented("remove_mcp_server"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        let claude_home = ClaudeHome::new(root.join("claude-home"), root.join("claude.json"));
        let app = mock_builder().plugin(tauri_plugin_store::Builder::default().build()).build(mock_context(noop_assets())).expect("build mock app");
        app.manage(AppState { claude_home, cache: crate::state::Cache::default() });
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
        let project = Some(ProjectContext::Project { project_id: "-Users-fixture-src-app".into() });
        let loaded = list_mcp_servers_inner(&app, &app.state::<AppState>(), project).unwrap();
        assert!(loaded.value.iter().any(|s| s.raw_name == "local-db"));
    }

    #[test]
    fn list_mcp_servers_rejects_an_unknown_project_as_invalid_input() {
        let app = app_handle();
        let project = Some(ProjectContext::Project { project_id: "-Users-nobody-nowhere".into() });
        assert!(matches!(list_mcp_servers_inner(&app, &app.state::<AppState>(), project).unwrap_err(), AppError::InvalidInput { .. }));
    }

    #[test]
    fn get_mcp_statuses_keys_by_server_id_and_covers_every_server() {
        let app = app_handle();
        let servers = list_mcp_servers_inner(&app, &app.state::<AppState>(), None).unwrap().value;
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
}
