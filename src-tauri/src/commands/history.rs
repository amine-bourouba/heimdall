//! Commands in commands/history.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! `list_history`'s real body lands here (M1-43); `delete_history_entry`/`clear_history_older_than`
//! stay stubs (M3-46). Every body runs through `state::blocking` (contract §1).

use heimdall_core::data::history;
use heimdall_core::data::history::HistoryPage;
use heimdall_core::{AppError, Result, WriteOutcome};
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking};

fn list_history_inner(state: &AppState, offset: u32, limit: u32) -> Result<HistoryPage> {
    history::page(&state.claude_home.history_file(), offset, limit)
}

#[tauri::command]
#[specta::specta]
pub async fn list_history(app: AppHandle, offset: u32, limit: u32) -> Result<HistoryPage> {
    blocking(move || list_history_inner(&app.state::<AppState>(), offset, limit)).await
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(entry_id: String) -> Result<WriteOutcome> {
    let _ = entry_id;
    Err(AppError::not_implemented("delete_history_entry"))
}

#[tauri::command]
#[specta::specta]
pub async fn clear_history_older_than(days: u32) -> Result<WriteOutcome> {
    let _ = days;
    Err(AppError::not_implemented("clear_history_older_than"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;

    fn state() -> AppState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        AppState {
            claude_home: ClaudeHome::new(root.join("claude-home"), root.join("claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn list_history_pages_the_real_fixture_file() {
        let all = list_history_inner(&state(), 0, 1000).unwrap();
        assert!(all.total >= 1);
        let first_page = list_history_inner(&state(), 0, 1).unwrap();
        assert_eq!(
            (first_page.entries.len(), first_page.offset, first_page.total),
            (1, 0, all.total)
        );
        assert_eq!(first_page.entries[0].id, all.entries[0].id);
    }

    #[test]
    fn list_history_offset_past_the_end_is_an_empty_page_not_an_error() {
        let page = list_history_inner(&state(), 1_000_000, 10).unwrap();
        assert!(page.entries.is_empty());
    }

    #[test]
    fn list_history_on_a_missing_file_is_an_empty_page() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            claude_home: ClaudeHome::new(dir.path(), dir.path().join(".claude.json")),
            cache: crate::state::Cache::default(),
        };
        let page = list_history_inner(&state, 0, 10).unwrap();
        assert_eq!((page.entries.len(), page.total, page.skipped), (0, 0, 0));
    }
}
