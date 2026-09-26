//! Commands in commands/export.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M3-09 (MVP) and M3-36 (CSV/PNG).
#![allow(unused_variables)]

use heimdall_core::export::session_markdown::ExportOptions;
use heimdall_core::export::{CsvKind, ExportDestination, PngKind};
use heimdall_core::{AppError, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn export_session(
    project_id: String,
    session_id: String,
    options: ExportOptions,
    destination: ExportDestination,
) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("export_session"))
}

#[tauri::command]
#[specta::specta]
pub async fn export_edit_patch(project_id: String, session_id: String, block_id: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("export_edit_patch"))
}

#[tauri::command]
#[specta::specta]
pub async fn export_analytics_csv(kind: CsvKind) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("export_analytics_csv"))
}

#[tauri::command]
#[specta::specta]
pub async fn save_png(kind: PngKind, bytes: Vec<u8>) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("save_png"))
}
