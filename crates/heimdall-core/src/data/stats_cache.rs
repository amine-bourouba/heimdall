//! `~/.claude/stats-cache.json` and when to trust it (Services/StatsCacheLoader.swift). Claude
//! Code's cache can stop advancing while transcripts keep growing, so it is used only when it
//! covers the latest local session day; otherwise stats are recomputed ([`super::stats`]).

use std::fmt::Display;
use std::path::Path;

use chrono::{Local, TimeZone};

use super::stats;
use crate::models::stats_cache::{StatsCache, StatsSource, StatsView};

/// The cache, or `None` when it is missing or does not parse.
pub fn load(path: &Path) -> Option<StatsCache> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

/// True when token series stop before activity does, or either series ends before
/// `latest_session_day` (dates compare as `YYYY-MM-DD` text).
pub fn is_stale(cache: &StatsCache, latest_session_day: Option<&str>) -> bool {
    let last_activity = cache.daily_activity.iter().map(|d| d.date.as_str()).max().unwrap_or("");
    let last_tokens = cache
        .daily_model_tokens
        .iter()
        .map(|d| d.date.as_str())
        .max()
        .unwrap_or("");
    if !last_activity.is_empty() && last_tokens < last_activity {
        return true;
    }
    latest_session_day.is_some_and(|day| last_tokens < day || last_activity < day)
}

/// The analytics source: the cache when it is fresh, else a recomputation, else the stale cache
/// (still reported as [`StatsSource::Cache`] -- it did not come from a recomputation, there was
/// just nothing to recompute from, e.g. no transcripts at all).
pub fn load_analytics(cache_path: &Path, projects_dir: &Path) -> Option<StatsView> {
    load_analytics_in(cache_path, projects_dir, &Local)
}

pub fn load_analytics_in<Tz: TimeZone>(cache_path: &Path, projects_dir: &Path, tz: &Tz) -> Option<StatsView>
where
    Tz::Offset: Display,
{
    let cache = load(cache_path);
    let latest = stats::latest_session_day_in(projects_dir, tz);
    match cache {
        Some(c) if !is_stale(&c, latest.as_deref()) => Some(StatsView {
            stats: c,
            source: StatsSource::Cache,
        }),
        cache => match stats::compute_in(projects_dir, tz) {
            Some(computed) => Some(StatsView {
                stats: computed,
                source: StatsSource::Computed,
            }),
            None => cache.map(|stats| StatsView {
                stats,
                source: StatsSource::Cache,
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::stats::tests::utc;
    use crate::models::stats_cache::{DailyActivity, DailyModelTokens};

    fn cache(activity: &[(&str, u64)], tokens: &[(&str, u64)]) -> StatsCache {
        StatsCache {
            daily_activity: activity
                .iter()
                .map(|(d, n)| DailyActivity {
                    date: d.to_string(),
                    message_count: *n,
                    session_count: 1,
                    tool_call_count: 0,
                })
                .collect(),
            daily_model_tokens: tokens
                .iter()
                .map(|(d, t)| DailyModelTokens {
                    date: d.to_string(),
                    tokens_by_model: [("claude-opus-4-6".to_string(), *t)].into(),
                })
                .collect(),
            ..Default::default()
        }
    }
    /// `<root>/projects/proj/<session>.jsonl` and the cache path `<root>/stats-cache.json`.
    fn root_with(jsonl: &str) -> tempfile::TempDir {
        let root = tempfile::tempdir().unwrap();
        let dir = root.path().join("projects").join("proj");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("5f0c2d9e-1a3b-4c5d-8e7f-9a0b1c2d3e4f.jsonl"), jsonl).unwrap();
        root
    }

    // Ported from the loader cases of PoirotTests/Models/StatsCacheTests.swift.
    #[test]
    fn loader_missing_file_returns_nil() {
        assert!(load(Path::new("/nonexistent/path/stats-cache.json")).is_none());
    }
    #[test]
    fn is_stale_tokens_lag_activity_is_true() {
        let c = cache(&[("2026-03-01", 1), ("2026-08-28", 2)], &[("2026-03-01", 100)]);
        assert!(is_stale(&c, Some("2026-08-28")));
        assert!(is_stale(&c, None));
    }
    #[test]
    fn is_stale_last_computed_before_latest_session_is_true() {
        let c = cache(&[("2026-03-15", 10)], &[("2026-03-15", 50)]);
        assert!(is_stale(&c, Some("2026-08-28")));
        assert!(!is_stale(&c, Some("2026-03-15")));
        assert!(!is_stale(&c, None));
    }
    #[test]
    fn load_analytics_stale_cache_recomputes_from_transcripts() {
        let root = root_with(concat!(
            r#"{"type":"user","timestamp":"2026-08-20T10:00:00.000Z","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
            "\n",
            r#"{"type":"assistant","timestamp":"2026-08-20T10:00:05.000Z","message":{"id":"a1","model":"claude-opus-4-6","usage":{"input_tokens":7,"output_tokens":3},"content":[]}}"#,
        ));
        let cache_path = root.path().join("stats-cache.json");
        std::fs::write(
            &cache_path,
            serde_json::to_vec(&cache(&[("2026-03-15", 1)], &[("2026-03-15", 999)])).unwrap(),
        )
        .unwrap();
        let v = load_analytics_in(&cache_path, &root.path().join("projects"), &utc()).unwrap();
        assert_eq!(v.source, StatsSource::Computed);
        let s = &v.stats;
        assert!(s.daily_model_tokens.iter().any(|d| d.date == "2026-08-20"));
        assert!(
            !s.daily_model_tokens
                .iter()
                .any(|d| d.tokens_by_model.get("claude-opus-4-6") == Some(&999))
        );
        let m = &s.model_usage["claude-opus-4-6"];
        assert_eq!((m.input_tokens, m.output_tokens), (7, 3));
    }
    #[test]
    fn load_analytics_fresh_cache_is_kept() {
        let root = root_with(
            r#"{"type":"user","timestamp":"2026-08-20T10:00:00.000Z","uuid":"u1","message":{"role":"user","content":"hi"}}"#,
        );
        let projects = root.path().join("projects");
        let latest = stats::latest_session_day_in(&projects, &utc()).unwrap();
        let mut fresh = cache(&[(&latest, 42)], &[(&latest, 1234)]);
        fresh.daily_model_tokens[0].tokens_by_model = [("kept-model".to_string(), 1234)].into();
        let cache_path = root.path().join("stats-cache.json");
        std::fs::write(&cache_path, serde_json::to_vec(&fresh).unwrap()).unwrap();
        let v = load_analytics_in(&cache_path, &projects, &utc()).unwrap();
        assert_eq!(v.source, StatsSource::Cache);
        assert_eq!(
            (
                v.stats.daily_activity[0].message_count,
                v.stats.daily_model_tokens[0].tokens_by_model.get("kept-model")
            ),
            (42, Some(&1234))
        );
    }

    #[test]
    fn a_stale_cache_is_kept_when_there_is_nothing_to_recompute() {
        let root = tempfile::tempdir().unwrap();
        let cache_path = root.path().join("stats-cache.json");
        std::fs::write(
            &cache_path,
            serde_json::to_vec(&cache(&[("2026-03-15", 1)], &[])).unwrap(),
        )
        .unwrap();
        let v = load_analytics_in(&cache_path, &root.path().join("projects"), &utc()).unwrap();
        assert_eq!((v.stats.daily_activity.len(), v.source), (1, StatsSource::Cache));
    }
}
