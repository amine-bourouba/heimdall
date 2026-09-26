//! Typed events: data-changed, watcher-status, menu-action, navigate (docs/ipc-contract.md §7).
//!
//! Event names are the struct name kebab-cased by `tauri_specta::Event`'s derive (`DataChanged` ->
//! `"data-changed"`), so the Rust names below double as the wire names in the contract table.

use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_specta::Event;

use heimdall_core::app_shell::AppAction;
use heimdall_core::classify::DataChange;
use heimdall_core::search::NavTarget;

/// Emitted whenever on-disk data the app has cached changes (watcher M1-47, prefs M1-39).
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct DataChanged(pub Vec<DataChange>);

/// Whether the file-system watcher is running normally, degraded to polling, or gave up.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum WatcherMode {
    Recursive,
    Degraded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct WatcherStatus {
    pub mode: WatcherMode,
    pub detail: Option<String>,
}

/// Emitted by the macOS menu (M2-13) for `useAppActions` to handle.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct MenuAction {
    pub action: AppAction,
}

/// Emitted by `open_in_main_window` (M3-30) for the router to act on.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
#[serde(rename_all = "camelCase")]
pub struct Navigate {
    pub target: NavTarget,
}
