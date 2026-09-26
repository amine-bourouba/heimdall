//! Commands in commands/tray.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37; bodies in M3-30.
#![allow(unused_variables)]

use heimdall_core::data::tray::TraySnapshot;
use heimdall_core::search::NavTarget;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn get_tray_snapshot(query: Option<String>) -> Result<TraySnapshot> {
    Err(AppError::not_implemented("get_tray_snapshot"))
}

#[tauri::command]
#[specta::specta]
pub async fn open_in_main_window(target: NavTarget) -> Result<()> {
    Err(AppError::not_implemented("open_in_main_window"))
}
