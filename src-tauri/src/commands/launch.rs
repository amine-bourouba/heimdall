//! Commands in commands/launch.rs (docs/ipc-contract.md §6). Typed stubs land in M1-37;
//! bodies in M2-11 (MVP) and M3-53 (later).
#![allow(unused_variables)]

use heimdall_core::os_integration::launch::{EditorTarget, RevealTarget};
use heimdall_core::os_integration::links;
use heimdall_core::{AppError, Result};

#[tauri::command]
#[specta::specta]
pub async fn open_external_url(url: String) -> Result<()> {
    // The allowlist check and the opener call both happen inside `links::open_external_url`
    // (heimdall-core, tauri-free and directly unit-tested there, ADR 0010 control 2) -- this is
    // only the real opener plugin wired in as the injected closure.
    links::open_external_url(&url, |u| {
        tauri_plugin_opener::open_url(u, None::<&str>).map_err(std::io::Error::other)
    })
}

#[tauri::command]
#[specta::specta]
pub async fn open_in_editor(target: EditorTarget) -> Result<()> {
    Err(AppError::not_implemented("open_in_editor"))
}

#[tauri::command]
#[specta::specta]
pub async fn open_terminal(project_id: String, session_id: Option<String>) -> Result<()> {
    Err(AppError::not_implemented("open_terminal"))
}

#[tauri::command]
#[specta::specta]
pub async fn reveal_in_file_manager(target: RevealTarget) -> Result<()> {
    Err(AppError::not_implemented("reveal_in_file_manager"))
}
