//! Commands in commands/session_data.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M1-42.
#![allow(unused_variables)]

use heimdall_core::diff::DiffResult;
use heimdall_core::data::debug_log::DebugLogPage;
use heimdall_core::models::file_version::{FileHistory, FileVersionContent};
use heimdall_core::models::session_facets::SessionFacets;
use heimdall_core::models::session_todo::{TodoGroup, TodoList};
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn get_session_todos(session_id: String) -> Result<Vec<TodoList>> {
    Err(AppError::not_implemented("get_session_todos"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_all_todos() -> Result<Vec<TodoGroup>> {
    Err(AppError::not_implemented("list_all_todos"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_file_history(project_id: String, session_id: String) -> Result<FileHistory> {
    Err(AppError::not_implemented("get_file_history"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_file_version(session_id: String, version_id: String) -> Result<FileVersionContent> {
    Err(AppError::not_implemented("get_file_version"))
}

#[tauri::command]
#[specta::specta]
pub async fn diff_file_versions(session_id: String, from_version_id: String, to_version_id: String) -> Result<DiffResult> {
    Err(AppError::not_implemented("diff_file_versions"))
}

#[tauri::command]
#[specta::specta]
pub async fn diff_lines(old_text: String, new_text: String) -> Result<DiffResult> {
    Err(AppError::not_implemented("diff_lines"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_debug_log_page(session_id: String, offset: u32, limit: u32, level: Option<String>) -> Result<DebugLogPage> {
    Err(AppError::not_implemented("get_debug_log_page"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_facets(session_id: String) -> Result<Option<SessionFacets>> {
    Err(AppError::not_implemented("get_session_facets"))
}
