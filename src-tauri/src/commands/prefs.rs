//! Commands in commands/prefs.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37, real
//! bodies land in M1-39. Every body runs through `state::blocking` (contract §1) since loading
//! and saving the preferences store, and opening a native folder picker, are all disk/OS work.

use heimdall_core::models::prefs::{Prefs, PrefsPatch};
use heimdall_core::{Result, WriteOutcome};
use tauri::AppHandle;

use crate::state::blocking;

#[tauri::command]
#[specta::specta]
pub async fn get_prefs(app: AppHandle) -> Result<Prefs> {
    blocking(move || crate::prefs::load(&app)).await
}

#[tauri::command]
#[specta::specta]
pub async fn update_prefs(app: AppHandle, patch: PrefsPatch) -> Result<Prefs> {
    blocking(move || crate::prefs::update(&app, patch)).await
}

#[tauri::command]
#[specta::specta]
pub async fn add_memory_folder(app: AppHandle) -> Result<WriteOutcome> {
    blocking(move || {
        let picked = crate::prefs::pick_folder(&app);
        crate::prefs::add_memory_folder(&app, picked)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_memory_folder(app: AppHandle, source_id: String) -> Result<WriteOutcome> {
    blocking(move || crate::prefs::remove_memory_folder(&app, &source_id)).await
}

#[tauri::command]
#[specta::specta]
pub async fn pick_config_project(app: AppHandle) -> Result<WriteOutcome> {
    blocking(move || {
        let picked = crate::prefs::pick_folder(&app);
        crate::prefs::pick_config_project(&app, picked)
    })
    .await
}
