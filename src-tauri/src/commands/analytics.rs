//! Commands in commands/analytics.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! `get_stats`'s real body lands here (M3-30). Every body runs through `state::blocking` (contract §1).

use heimdall_core::data::stats_cache;
use heimdall_core::models::stats_cache::StatsView;
use heimdall_core::{AppError, Result};
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking};

fn get_stats_inner(state: &AppState) -> Result<StatsView> {
    let claude_home = &state.claude_home;
    stats_cache::load_analytics(&claude_home.stats_cache_file(), &claude_home.projects_dir())
        .ok_or_else(|| AppError::not_found("no stats cache and nothing to compute from (no sessions yet)"))
}

#[tauri::command]
#[specta::specta]
pub async fn get_stats(app: AppHandle) -> Result<StatsView> {
    blocking(move || get_stats_inner(&app.state::<AppState>())).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::models::stats_cache::StatsSource;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;

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
    fn get_stats_reports_the_cache_source_when_the_fixtures_own_cache_is_fresh() {
        // `fixtures/synthetic`'s own `stats-cache.json` has `lastComputedDate: "2026-09-10"`,
        // matching its own transcripts' latest session day exactly -- not stale, so this is the
        // "serve the on-disk cache as-is" path, not a recomputation.
        let view = get_stats_inner(&state("synthetic")).unwrap();
        assert_eq!(view.source, StatsSource::Cache);
        assert!(view.stats.total_sessions > 0);
    }

    #[test]
    fn get_stats_recomputes_when_no_cache_file_exists() {
        // `fixtures/screenshot` has real session transcripts but (confirmed directly) no
        // `stats-cache.json` at all, so this exercises the opposite path from the test above.
        let view = get_stats_inner(&state("screenshot")).unwrap();
        assert_eq!(view.source, StatsSource::Computed);
        assert!(view.stats.total_sessions > 0);
    }

    #[test]
    fn get_stats_on_a_home_with_no_sessions_and_no_cache_is_not_found() {
        let dir = tempfile::tempdir().unwrap();
        let state = AppState {
            claude_home: ClaudeHome::new(dir.path(), dir.path().join(".claude.json")),
            cache: crate::state::Cache::default(),
        };
        assert!(matches!(
            get_stats_inner(&state).unwrap_err(),
            AppError::NotFound { .. }
        ));
    }
}
