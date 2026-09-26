//! Commands in commands/update.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37; bodies in M4-04.
#![allow(unused_variables)]

use heimdall_core::app_shell::UpdateStatus;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn check_for_update() -> Result<UpdateStatus> {
    Err(AppError::not_implemented("check_for_update"))
}

#[tauri::command]
#[specta::specta]
pub async fn install_update() -> Result<()> {
    Err(AppError::not_implemented("install_update"))
}
