//! Commands in commands/config_read.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M1-44.
//!
//! `get_config_item`'s and `delete_config_item`'s contract argument name `ref` is spelled
//! `config_ref` here: `ref` is a reserved word, so it cannot be a Rust parameter name.
#![allow(unused_variables)]

use heimdall_core::config::read::memory::MemorySource;
use heimdall_core::config::read::models::ModelsView;
use heimdall_core::models::config::hook::HookEntry;
use heimdall_core::models::config::memory_file::{MemoryFileDetail, MemoryFileSummary};
use heimdall_core::models::config::plugin::PluginInfo;
use heimdall_core::models::config::scope::{ConfigItemDetail, ConfigItemSummary, ConfigKind, ConfigRef, ProjectContext};
use heimdall_core::{AppError, Loaded, Result};

#[tauri::command]
#[specta::specta]
pub async fn list_config_items(kind: ConfigKind, project: Option<ProjectContext>) -> Result<Loaded<Vec<ConfigItemSummary>>> {
    Err(AppError::not_implemented("list_config_items"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_config_item(config_ref: ConfigRef) -> Result<ConfigItemDetail> {
    Err(AppError::not_implemented("get_config_item"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_hooks(project: Option<ProjectContext>) -> Result<Loaded<Vec<HookEntry>>> {
    Err(AppError::not_implemented("list_hooks"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_plugins() -> Result<Vec<PluginInfo>> {
    Err(AppError::not_implemented("list_plugins"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_models(project: Option<ProjectContext>) -> Result<ModelsView> {
    Err(AppError::not_implemented("list_models"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_memory_sources() -> Result<Vec<MemorySource>> {
    Err(AppError::not_implemented("list_memory_sources"))
}

#[tauri::command]
#[specta::specta]
pub async fn list_memory_files(source_id: String) -> Result<Vec<MemoryFileSummary>> {
    Err(AppError::not_implemented("list_memory_files"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_memory_file(source_id: String, file_id: String) -> Result<MemoryFileDetail> {
    Err(AppError::not_implemented("get_memory_file"))
}
