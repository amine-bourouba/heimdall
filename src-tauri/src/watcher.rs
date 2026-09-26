//! One recursive filesystem watcher over `~/.claude`, `~/.claude.json` (through its parent, with a
//! file filter) and the custom memory folders, debounced and classified into `data-changed`
//! events (ADR 0006). Poirot's `FileWatcher.swift` watches one folder at a time, non-recursively,
//! with a hand-rolled 1 s debounce, started from 13 call sites across 8 files
//! (`docs/spec/claude-home-inventory.md` §4) — this is a from-scratch design with no Swift
//! equivalent to port; ADR 0006 is the spec.

use std::path::Path;
use std::sync::Mutex;
use std::time::Duration;

use heimdall_core::classify::{classify, Area, DataChange};
use heimdall_core::paths::{compare_key, ClaudeHome, TargetOs};
use notify_debouncer_full::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use tauri::{AppHandle, Manager};
use tauri_specta::Event;

use crate::events::{DataChanged, WatcherMode, WatcherStatus};
use crate::state::AppState;

/// Matches `notify-debouncer-full`'s own recommended timeout (its docs' own example uses 2s; ADR
/// 0006 says "starting at 1s" for Poirot parity) — long enough to coalesce a burst of saves from
/// one edit, short enough that the 2.5s acceptance budget still has headroom.
const DEBOUNCE: Duration = Duration::from_millis(1000);

type AppDebouncer = Debouncer<RecommendedWatcher, RecommendedCache>;

/// Held in Tauri's managed state so preference commands can re-register the custom memory
/// folders when they change, without tearing down the whole watcher.
pub struct WatcherHandle {
    debouncer: Mutex<AppDebouncer>,
    custom_folders: Mutex<Vec<String>>,
}

impl WatcherHandle {
    /// Unwatches every previously registered custom folder and watches `folders` instead. Best
    /// effort: a folder that fails to watch (for example, one the user removed from disk) is
    /// simply skipped, the same as Poirot silently doing nothing for a folder `open(2)` cannot see.
    pub fn set_custom_folders(&self, folders: &[String]) {
        let mut debouncer = self.debouncer.lock().expect("watcher lock");
        let mut current = self.custom_folders.lock().expect("watcher lock");
        for folder in current.iter() {
            let _ = debouncer.unwatch(Path::new(folder));
        }
        for folder in folders {
            let _ = debouncer.watch(Path::new(folder), RecursiveMode::NonRecursive);
        }
        *current = folders.to_vec();
    }
}

/// Classifies one changed path: a custom memory folder's own contents never fall under
/// `claude_home_root`, so `classify` (M1-34) cannot see them — checked first, against the folder
/// list as it stood on the last registration, before falling through to `classify` for everything
/// under `~/.claude` and `~/.claude.json` itself.
fn classify_path(path: &Path, claude_home: &ClaudeHome, custom_folders: &[String], os: TargetOs) -> Vec<DataChange> {
    let path_str = path.display().to_string();
    let path_key = compare_key(&path_str, os);
    if let Some(index) = custom_folders.iter().position(|f| {
        let folder_key = compare_key(f, os);
        path_key == folder_key || path_key.starts_with(&format!("{folder_key}/"))
    }) {
        return vec![DataChange { area: Area::Memory, key: Some(format!("folder:{index}")) }];
    }
    classify(&path_str, &claude_home.root().display().to_string(), &claude_home.config_file().display().to_string(), os)
}

fn emit_status<R: tauri::Runtime>(app: &AppHandle<R>, mode: WatcherMode, detail: Option<String>) {
    let _ = WatcherStatus { mode, detail }.emit(app);
}

fn handle_events<R: tauri::Runtime>(app: &AppHandle<R>, claude_home: &ClaudeHome, custom_folders: &Mutex<Vec<String>>, os: TargetOs, result: DebounceEventResult) {
    // A per-path watch error (a file vanished between the event firing and the debounce window
    // closing, say) is not actionable here; there is nothing to invalidate or tell the UI.
    let Ok(events) = result else { return };

    // A rescan notice means events since the last one may be incomplete; treat the whole home as
    // changed rather than trust a partial batch (ADR 0006 decision 6, the Windows/overflow half).
    if events.iter().any(|e| e.flag() == Some(notify_debouncer_full::notify::event::Flag::Rescan)) {
        let _ = DataChanged(vec![DataChange { area: Area::All, key: None }]).emit(app);
        return;
    }

    let folders = custom_folders.lock().expect("watcher lock").clone();
    let mut changes: Vec<DataChange> = Vec::new();
    for event in &events {
        for path in &event.paths {
            for change in classify_path(path, claude_home, &folders, os) {
                if !changes.contains(&change) {
                    changes.push(change);
                }
            }
        }
    }
    if !changes.is_empty() {
        let state = app.state::<AppState>();
        for change in &changes {
            state.cache.invalidate_area(change.area);
        }
        let _ = DataChanged(changes).emit(app);
    }
}

/// Registers `root` recursively; if that fails (for example Linux's `fs.inotify.max_user_watches`),
/// falls back to watching each of `root`'s own immediate subfolders recursively instead (ADR 0006
/// decision 6) and reports the reason.
fn register_root(debouncer: &mut AppDebouncer, root: &Path) -> Result<(), String> {
    if let Err(e) = debouncer.watch(root, RecursiveMode::Recursive) {
        let mut watched_any = false;
        if let Ok(entries) = std::fs::read_dir(root) {
            for dir in entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
                watched_any |= debouncer.watch(&dir, RecursiveMode::Recursive).is_ok();
            }
        }
        if !watched_any {
            return Err(format!("could not register any watch under {}: {e}", root.display()));
        }
        return Err(format!("recursive registration failed ({e}); watching known subfolders individually"));
    }
    Ok(())
}

pub fn init<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    let app_handle = app.handle().clone();
    let claude_home = app.state::<AppState>().claude_home.clone();
    let os = TargetOs::current();
    let custom_folders = crate::prefs::load(&app_handle).map(|p| p.custom_memory_folders).unwrap_or_default();

    let debouncer_home = claude_home.clone();
    let debouncer_app = app_handle.clone();
    let custom_folders_cell: std::sync::Arc<Mutex<Vec<String>>> = std::sync::Arc::new(Mutex::new(custom_folders.clone()));
    let handler_folders = custom_folders_cell.clone();
    let mut debouncer = new_debouncer(DEBOUNCE, None, move |result: DebounceEventResult| {
        handle_events(&debouncer_app, &debouncer_home, &handler_folders, os, result);
    })
    .map_err(|e| std::io::Error::other(format!("failed to create the file watcher: {e}")))?;

    if let Err(detail) = register_root(&mut debouncer, claude_home.root()) {
        emit_status(&app_handle, WatcherMode::Degraded, Some(detail));
    } else {
        emit_status(&app_handle, WatcherMode::Recursive, None);
    }

    if let Some(parent) = claude_home.config_file().parent() {
        let _ = debouncer.watch(parent, RecursiveMode::NonRecursive);
    }
    for folder in &custom_folders {
        let _ = debouncer.watch(Path::new(folder), RecursiveMode::NonRecursive);
    }

    // `custom_folders_cell` (used by the debounce callback) and `WatcherHandle.custom_folders`
    // (used by `set_custom_folders`) are deliberately separate: the handle re-registers watches
    // *and* replaces its own list under one lock, then the callback's clone is refreshed on the
    // handle's next read — a race window of at most one debounce tick, acceptable since a missed
    // event only makes data late, never wrong (ADR 0006 decision 4).
    app.manage(WatcherHandle { debouncer: Mutex::new(debouncer), custom_folders: Mutex::new(custom_folders) });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_path_recognises_a_custom_memory_folder_before_falling_through_to_classify() {
        let dir = tempfile::tempdir().unwrap();
        let claude_home = ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
        let folder = dir.path().join("notes");
        let folders = vec![folder.display().to_string()];
        let changed = classify_path(&folder.join("MEMORY.md"), &claude_home, &folders, TargetOs::current());
        assert_eq!(changed, [DataChange { area: Area::Memory, key: Some("folder:0".to_string()) }]);
    }

    #[test]
    fn classify_path_falls_through_to_classify_for_paths_under_the_claude_home() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join(".claude");
        let claude_home = ClaudeHome::new(&root, dir.path().join(".claude.json"));
        let changed = classify_path(&root.join("plans").join("x.md"), &claude_home, &[], TargetOs::current());
        assert_eq!(changed, [DataChange { area: Area::Plans, key: None }]);
    }

    #[test]
    fn classify_path_matches_a_folder_prefix_not_just_an_unrelated_sibling_with_a_shared_prefix() {
        let dir = tempfile::tempdir().unwrap();
        let claude_home = ClaudeHome::new(dir.path().join(".claude"), dir.path().join(".claude.json"));
        let folders = vec![dir.path().join("notes").display().to_string()];
        // "notes-extra" starts with "notes" as a string, but is not inside it as a path.
        let sibling = dir.path().join("notes-extra").join("x.md");
        let changed = classify_path(&sibling, &claude_home, &folders, TargetOs::current());
        assert!(changed.is_empty());
    }
}
