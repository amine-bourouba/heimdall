//! Commands in commands/sessions.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M1-41 (MVP) and M3-46 (delete).
#![allow(unused_variables)]

use heimdall_core::data::tray::RecentSession;
use heimdall_core::models::message::MessagePage;
use heimdall_core::models::project::ProjectSummary;
use heimdall_core::models::session::{SessionDetail, SessionSummary};
use heimdall_core::{AppError, Loaded, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn list_projects() -> Result<Loaded<Vec<ProjectSummary>>> {
    Err(AppError::not_implemented("list_projects"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_sessions(project_id: String, include_agents: bool) -> Result<Loaded<Vec<SessionSummary>>> {
    Err(AppError::not_implemented("list_sessions"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session(project_id: String, session_id: String) -> Result<SessionDetail> {
    Err(AppError::not_implemented("get_session"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_session_messages(project_id: String, session_id: String, start: u32, count: u32) -> Result<MessagePage> {
    Err(AppError::not_implemented("get_session_messages"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_recent_sessions(limit: u32) -> Result<Vec<RecentSession>> {
    Err(AppError::not_implemented("list_recent_sessions"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_session(project_id: String, session_id: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("delete_session"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_project(project_id: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("delete_project"))
}
