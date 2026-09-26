//! Commands in commands/analytics.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M3-30.
#![allow(unused_variables)]

use heimdall_core::models::stats_cache::StatsView;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn get_stats() -> Result<StatsView> {
    Err(AppError::not_implemented("get_stats"))
}
