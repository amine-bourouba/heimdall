//! IPC command registration through tauri-specta (ADR 0004).
//!
//! Placeholder created by M1-03 (port-plan §5.4); owned by M1-37, which registers every command
//! from docs/ipc-contract.md and exports src/ipc/bindings.ts.

pub mod analytics;
pub mod app;
pub mod config_read;
pub mod config_write;
pub mod export;
pub mod history;
pub mod launch;
pub mod mcp;
pub mod prefs;
pub mod search;
pub mod session_data;
pub mod sessions;
pub mod tray;
pub mod update;

pub fn builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
}
