//! Commands in commands/config_write.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M3-44.
//!
//! `delete_config_item`'s contract argument name `ref` is spelled `config_ref` here (see config_read.rs).
#![allow(unused_variables)]

use heimdall_core::models::config::hook::HookGroupInput;
use heimdall_core::models::config::scope::{ConfigKind, ConfigRef, ProjectContext};
use heimdall_core::models::config::sub_agent::AgentInput;
use heimdall_core::{AppError, Result, WriteOutcome};

#[tauri::command]
#[specta::specta]
pub async fn set_plugin_enabled(key: String, enabled: bool) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("set_plugin_enabled"))
}

#[tauri::command]
#[specta::specta]
pub async fn remove_plugin(key: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("remove_plugin"))
}

#[tauri::command]
#[specta::specta]
pub async fn set_default_model(model: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("set_default_model"))
}

#[tauri::command]
#[specta::specta]
pub async fn set_project_model(project: ProjectContext, model: Option<String>) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("set_project_model"))
}

#[tauri::command]
#[specta::specta]
pub async fn save_hook(
    project: Option<ProjectContext>,
    event: String,
    group: HookGroupInput,
    existing_index: Option<u32>,
) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("save_hook"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_hook(project: Option<ProjectContext>, event: String, index: u32) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("delete_hook"))
}

#[tauri::command]
#[specta::specta]
pub async fn import_hooks(project: Option<ProjectContext>) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("import_hooks"))
}

#[tauri::command]
#[specta::specta]
pub async fn export_hooks(project: Option<ProjectContext>) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("export_hooks"))
}

#[tauri::command]
#[specta::specta]
pub async fn save_agent(input: AgentInput, original_slug: Option<String>) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("save_agent"))
}

#[tauri::command]
#[specta::specta]
pub async fn import_agent() -> Result<WriteOutcome> {
    Err(AppError::not_implemented("import_agent"))
}

#[tauri::command]
#[specta::specta]
pub async fn export_agent(slug: String) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("export_agent"))
}

#[tauri::command]
#[specta::specta]
pub async fn create_config_template(kind: ConfigKind, project: Option<ProjectContext>) -> Result<ConfigRef> {
    Err(AppError::not_implemented("create_config_template"))
}

#[tauri::command]
#[specta::specta]
pub async fn delete_config_item(config_ref: ConfigRef) -> Result<WriteOutcome> {
    Err(AppError::not_implemented("delete_config_item"))
}
