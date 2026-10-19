//! Commands in commands/history.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! `list_history`'s real body landed in M1-43; `delete_history_entry`/`clear_history_older_than`
//! land here (M3-46). Every body runs through `state::blocking` (contract §1).

use std::time::{SystemTime, UNIX_EPOCH};

use heimdall_core::data::history;
use heimdall_core::data::history::HistoryPage;
use heimdall_core::data::history_write;
use heimdall_core::{Result, WriteOutcome};
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking, confirm};

fn list_history_inner(state: &AppState, offset: u32, limit: u32) -> Result<HistoryPage> {
    history::page(&state.claude_home.history_file(), offset, limit)
}

#[tauri::command]
#[specta::specta]
pub async fn list_history(app: AppHandle, offset: u32, limit: u32) -> Result<HistoryPage> {
    blocking(move || list_history_inner(&app.state::<AppState>(), offset, limit)).await
}

fn delete_history_entry_inner(state: &AppState, entry_id: &str, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    history_write::delete_entry(&state.claude_home.history_file(), entry_id)
}

#[tauri::command]
#[specta::specta]
pub async fn delete_history_entry(app: AppHandle, entry_id: String) -> Result<WriteOutcome> {
    blocking(move || {
        let confirmed = confirm(
            &app,
            "Confirm Delete",
            "Delete this prompt from history? This cannot be undone.",
            "Delete",
        );
        delete_history_entry_inner(&app.state::<AppState>(), &entry_id, confirmed)
    })
    .await
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn clear_history_older_than_inner(state: &AppState, days: u32, confirmed: bool) -> Result<WriteOutcome> {
    if !confirmed {
        return Ok(WriteOutcome::Cancelled);
    }
    history_write::clear_older_than(&state.claude_home.history_file(), days, now_ms())
}

#[tauri::command]
#[specta::specta]
pub async fn clear_history_older_than(app: AppHandle, days: u32) -> Result<WriteOutcome> {
    blocking(move || {
        let confirmed = confirm(
            &app,
            "Confirm Delete",
            &format!("Delete every history entry older than {days} days? This cannot be undone."),
            "Delete",
        );
        clear_history_older_than_inner(&app.state::<AppState>(), days, confirmed)
    })
    .await
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

    /// A fresh tempdir with its own `history.jsonl` -- never the shared `fixtures/synthetic` tree
    /// read above, since these tests delete from it.
    fn state_with_history(content: &str) -> AppState {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("history.jsonl"), content).unwrap();
        // `ClaudeHome::new`'s `root` leaks for the test's lifetime by boxing the tempdir away --
        // simplest is to just forget the guard so the file outlives this function.
        let root = dir.keep();
        AppState {
            claude_home: ClaudeHome::new(&root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    fn line(display: &str, ts: i64) -> String {
        format!(r#"{{"display":"{display}","pastedContents":{{}},"timestamp":{ts},"project":"/x"}}"#)
    }

    #[test]
    fn delete_history_entry_declined_is_cancelled_and_writes_nothing() {
        let state = state_with_history(&line("a", 1000));
        let before = std::fs::read(state.claude_home.history_file()).unwrap();
        assert_eq!(
            delete_history_entry_inner(&state, "anything", false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert_eq!(std::fs::read(state.claude_home.history_file()).unwrap(), before);
    }

    #[test]
    fn delete_history_entry_confirmed_removes_the_matching_line() {
        let state = state_with_history(&line("a", 1000));
        let id = list_history_inner(&state, 0, 10).unwrap().entries[0].id.clone();
        assert_eq!(
            delete_history_entry_inner(&state, &id, true).unwrap(),
            WriteOutcome::Written
        );
        assert!(list_history_inner(&state, 0, 10).unwrap().entries.is_empty());
    }

    #[test]
    fn clear_history_older_than_declined_is_cancelled_and_writes_nothing() {
        let state = state_with_history(&line("a", 1000));
        let before = std::fs::read(state.claude_home.history_file()).unwrap();
        assert_eq!(
            clear_history_older_than_inner(&state, 0, false).unwrap(),
            WriteOutcome::Cancelled
        );
        assert_eq!(std::fs::read(state.claude_home.history_file()).unwrap(), before);
    }

    #[test]
    fn clear_history_older_than_confirmed_removes_entries_before_the_cutoff() {
        let day_ms: i64 = 86_400_000;
        let now = now_ms();
        let state = state_with_history(&[line("old", now - 10 * day_ms), line("new", now)].join("\n"));
        assert_eq!(
            clear_history_older_than_inner(&state, 1, true).unwrap(),
            WriteOutcome::Written
        );
        let remaining = list_history_inner(&state, 0, 10).unwrap().entries;
        assert_eq!((remaining.len(), remaining[0].display.as_str()), (1, "new"));
    }
}
