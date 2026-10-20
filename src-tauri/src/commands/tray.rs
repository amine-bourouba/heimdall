//! Commands in commands/tray.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37, both
//! bodies land here (M3-30). Every body runs through `state::blocking` (contract §1), except
//! window focus/emit, which must run on the main thread (`AppHandle` methods are thread-safe
//! dispatches to it regardless of which thread calls them, so no special handling is needed).

use heimdall_core::data::{stats_cache, tray};
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::discover;
use heimdall_core::search::NavTarget;
use heimdall_core::{AppError, Result};
use tauri::{AppHandle, Manager, Runtime};
use tauri_specta::Event;

use crate::events::Navigate;
use crate::state::{AppState, blocking};

fn get_tray_snapshot_inner(state: &AppState, query: Option<&str>) -> Result<tray::TraySnapshot> {
    let claude_home = &state.claude_home;
    let projects = discover(&claude_home.projects_dir(), TargetOs::current())?.value;
    let stats = stats_cache::load_analytics(&claude_home.stats_cache_file(), &claude_home.projects_dir());
    Ok(tray::snapshot(&projects, stats.as_ref().map(|v| &v.stats), query))
}

#[tauri::command]
#[specta::specta]
pub async fn get_tray_snapshot(app: AppHandle, query: Option<String>) -> Result<tray::TraySnapshot> {
    blocking(move || get_tray_snapshot_inner(&app.state::<AppState>(), query.as_deref())).await
}

/// Shows and focuses the main window (creating it is `window::init`'s own job at startup; this
/// never happens before that has run), then emits `Navigate` for the router to act on
/// (`src-tauri/src/events.rs`'s own doc comment already named this as `Navigate`'s one caller).
/// `pub(crate)`: `crate::tray`'s own Linux menu (M3-56) reuses this exact logic for its "Open
/// Heimdall" and per-session items, rather than duplicating show/focus/emit a second time.
pub(crate) fn open_in_main_window_inner<R: Runtime>(app: &AppHandle<R>, target: NavTarget) -> Result<()> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| AppError::not_found("no main window"))?;
    window.show().map_err(|e| AppError::io(e.to_string()))?;
    window.set_focus().map_err(|e| AppError::io(e.to_string()))?;
    Navigate { target }.emit(app).map_err(|e| AppError::io(e.to_string()))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn open_in_main_window(app: AppHandle, target: NavTarget) -> Result<()> {
    blocking(move || open_in_main_window_inner(&app, target)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    use tauri::webview::WebviewWindowBuilder;

    fn state(fixture_root: &str) -> AppState {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../fixtures")
            .join(fixture_root);
        AppState {
            claude_home: ClaudeHome::new(root.join("claude-home"), root.join("claude.json")),
            cache: crate::state::Cache::default(),
        }
    }

    #[test]
    fn get_tray_snapshot_covers_recent_sessions_and_todays_messages() {
        // `total_sessions` comes from `stats_cache::load_analytics`, which (unlike project/session
        // discovery, via `sessions-index.json`) requires real UUID-named session files to compute
        // anything at all -- `fixtures/screenshot`'s readable `conv-001`-style names don't qualify
        // (confirmed directly), so `fixtures/windows` is used here instead.
        let snapshot = get_tray_snapshot_inner(&state("windows"), None).unwrap();
        assert!(!snapshot.recent.is_empty());
        assert!(snapshot.total_sessions > 0);
    }

    #[test]
    fn get_tray_snapshot_filters_by_query() {
        let all = get_tray_snapshot_inner(&state("screenshot"), None).unwrap();
        let filtered = get_tray_snapshot_inner(&state("screenshot"), Some("no-such-session-xyz")).unwrap();
        assert!(filtered.recent.is_empty() && !all.recent.is_empty());
    }

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        // open_in_main_window_inner emits a typed Navigate event; tauri-specta panics on any
        // emit() ("EventRegistry not found... forget to call Builder::mount_events?") unless this
        // has run first, the same mount_events(app) call lib.rs's own real setup() makes.
        tauri_specta::Builder::<tauri::test::MockRuntime>::new()
            .events(tauri_specta::collect_events![crate::events::Navigate])
            .mount_events(&app);
        app.handle().clone()
    }

    #[test]
    fn open_in_main_window_with_no_main_window_is_not_found() {
        let app = app_handle();
        let target = NavTarget::Screen {
            screen: heimdall_core::search::NavScreen::History,
        };
        assert!(matches!(
            open_in_main_window_inner(&app, target).unwrap_err(),
            AppError::NotFound { .. }
        ));
    }

    #[test]
    fn open_in_main_window_shows_focuses_and_emits_navigate_when_the_window_exists() {
        let app = app_handle();
        WebviewWindowBuilder::new(&app, "main", tauri::WebviewUrl::App("index.html".into()))
            .build()
            .expect("build mock main window");
        let target = NavTarget::Session {
            project_id: "proj".to_string(),
            session_id: "sess".to_string(),
        };
        open_in_main_window_inner(&app, target).unwrap();
    }
}
