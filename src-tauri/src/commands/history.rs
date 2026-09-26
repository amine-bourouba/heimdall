//! Commands in commands/history.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M1-43 (MVP) and M3-46 (delete/clear).
#![allow(unused_variables)]

use heimdall_core::data::history::HistoryPage;
use heimdall_core::{AppError, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn list_history(offset: u32, limit: u32) -> Result<HistoryPage> {
    Err(AppError::not_implemented("list_history"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(entry_id: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("delete_history_entry"))
}

#[tauri::command]
#[specta::specta]
pub async fn clear_history_older_than(days: u32) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("clear_history_older_than"))
}
