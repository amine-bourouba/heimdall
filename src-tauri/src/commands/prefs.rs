//! Commands in commands/prefs.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37; bodies in M1-39.
#![allow(unused_variables)]

use heimdall_core::models::prefs::{Prefs, PrefsPatch};
use heimdall_core::{AppError, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn get_prefs() -> Result<Prefs> {
    Err(AppError::not_implemented("get_prefs"))
}

#[tauri::command]
#[specta::specta]
pub async fn update_prefs(patch: PrefsPatch) -> Result<Prefs> {
    Err(AppError::not_implemented("update_prefs"))
}

#[tauri::command]
#[specta::specta]
pub async fn add_memory_folder() -> Result<WriteOutcome> {
    Err(AppError::not_implemented("add_memory_folder"))
}

#[tauri::command]
#[specta::specta]
pub async fn remove_memory_folder(source_id: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("remove_memory_folder"))
}

#[tauri::command]
#[specta::specta]
pub async fn pick_config_project() -> Result<WriteOutcome> {
    Err(AppError::not_implemented("pick_config_project"))
}
