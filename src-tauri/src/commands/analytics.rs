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
        // `latest_session_day` (stats.rs) keys staleness off each transcript file's own mtime, not
        // anything inside it -- and a fresh git checkout gives every file "now" as its mtime, which
        // is always later than this fixture's fixed `lastComputedDate: "2026-09-10"`. Reading the
        // shared fixture tree directly would make this test fail on every fresh checkout (it did,
        // in CI, confirmed directly). Copying its two needed files into a tempdir and setting an
        // explicit, fixed mtime makes "fresh" independent of when or where this test runs.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic/claude-home");
        std::fs::copy(src.join("stats-cache.json"), root.join("stats-cache.json")).unwrap();
        let project_dir = root.join("projects").join("-Users-fixture-src-app");
        std::fs::create_dir_all(&project_dir).unwrap();
        let before_last_computed = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(946_684_800); // 2000-01-01T00:00:00Z
        for name in [
            "11111111-1111-4111-8111-111111111111.jsonl",
            "22222222-2222-4222-8222-222222222222.jsonl",
            "sessions-index.json",
        ] {
            let dest = project_dir.join(name);
            std::fs::copy(src.join("projects/-Users-fixture-src-app").join(name), &dest).unwrap();
            if name.ends_with(".jsonl") {
                std::fs::File::options()
                    .write(true)
                    .open(&dest)
                    .unwrap()
                    .set_modified(before_last_computed)
                    .unwrap();
            }
        }
        let state = AppState {
            claude_home: ClaudeHome::new(root, root.join(".claude.json")),
            cache: crate::state::Cache::default(),
        };
        let view = get_stats_inner(&state).unwrap();
        assert_eq!(view.source, StatsSource::Cache);
        assert!(view.stats.total_sessions > 0);
    }

    #[test]
    fn get_stats_recomputes_when_no_cache_file_exists() {
        // `fixtures/windows` has real session transcripts, real UUID-named files (`stats::
        // session_files` only recognises those, unlike `fixtures/screenshot`'s readable
        // `conv-001`-style names) and (confirmed directly) no `stats-cache.json` at all, so this
        // exercises the opposite path from the test above.
        let view = get_stats_inner(&state("windows")).unwrap();
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
