//! Commands in commands/mcp.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M1-45 (MVP) and M3-45 (writes).
#![allow(unused_variables)]

use std::collections::BTreeMap;

use heimdall_core::models::catalog::mcp_catalog::McpCatalogEntry;
use heimdall_core::models::config::mcp::{McpServer, McpServerInput, McpServerStatus};
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::{AppError, Loaded, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn list_mcp_servers(project: Option<ProjectContext>) -> Result<Loaded<Vec<McpServer>>> {
    Err(AppError::not_implemented("list_mcp_servers"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_statuses(project: Option<ProjectContext>) -> Result<BTreeMap<String, McpServerStatus>> {
    Err(AppError::not_implemented("get_mcp_statuses"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_mcp_catalog() -> Result<Vec<McpCatalogEntry>> {
    Err(AppError::not_implemented("get_mcp_catalog"))
}

#[tauri::command]
#[specta::specta]
pub async fn add_mcp_server(input: McpServerInput) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("add_mcp_server"))
}

#[tauri::command]
#[specta::specta]
pub async fn update_mcp_server(original_name: String, input: McpServerInput) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("update_mcp_server"))
}

#[tauri::command]
#[specta::specta]
pub async fn remove_mcp_server(name: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("remove_mcp_server"))
}
