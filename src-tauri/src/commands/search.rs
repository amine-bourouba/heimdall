//! Commands in commands/search.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37; bodies in M1-46.
#![allow(unused_variables)]

use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::models::prefs::SearchMode;
use heimdall_core::search::SearchResults;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn search(query: String, mode: SearchMode, project: Option<ProjectContext>) -> Result<SearchResults> {
    Err(AppError::not_implemented("search"))
}
