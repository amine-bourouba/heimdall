//! Commands in commands/app.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37; bodies in M1-40.
#![allow(unused_variables)]

use heimdall_core::app_shell::{AppInfo, CliStatus, SidebarCounts};
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn get_app_info() -> Result<AppInfo> {
    Err(AppError::not_implemented("get_app_info"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_sidebar_counts(project: Option<ProjectContext>) -> Result<SidebarCounts> {
    Err(AppError::not_implemented("get_sidebar_counts"))
}

#[tauri::command]
#[specta::specta]
pub async fn detect_claude_cli() -> Result<CliStatus> {
    Err(AppError::not_implemented("detect_claude_cli"))
}
