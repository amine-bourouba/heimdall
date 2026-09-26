//! App state: the resolved Claude home plus a path-keyed cache validated by (mtime, size) on every
//! read (ADR 0006 decision 4) — a missed watcher event (M1-47) makes cached data late, never
//! wrong, since a changed file's own metadata always disagrees with what was cached for it.
//!
//! Poirot's own transcript cache (`AppState.swift:303`) is never invalidated at all — `clearCache()`
//! has no callers outside its tests (ADR 0006's own Context section calls this out as a known bug).
//! `AppStateCacheTests`' three cases (store-and-retrieve, miss on an unknown id, clear-removes-all)
//! are ported below as the hit/miss/clear-everything cases; the staleness and per-area invalidation
//! cases have no Swift equivalent, since Heimdall's is a materially richer cache.

use std::any::Any;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

use heimdall_core::classify::Area;
use heimdall_core::error::{AppError, Result};
use heimdall_core::paths::ClaudeHome;

/// `HEIMDALL_CLAUDE_HOME` lets a debug build point at a fixture tree instead of the real
/// `~/.claude`. Reading it in a release build would let an attacker-controlled environment
/// variable redirect a shipped app at arbitrary data (ADR 0010), so the release build never even
/// contains the code path that reads it.
#[cfg(debug_assertions)]
fn heimdall_claude_home_override() -> Option<PathBuf> {
    std::env::var_os("HEIMDALL_CLAUDE_HOME").map(PathBuf::from).filter(|p| !p.as_os_str().is_empty())
}

#[cfg(not(debug_assertions))]
fn heimdall_claude_home_override() -> Option<PathBuf> {
    None
}

/// Resolves the Claude home for this run: `HEIMDALL_CLAUDE_HOME` (debug builds only), then
/// `CLAUDE_CONFIG_DIR`, then the real user home — `ClaudeHome::from_env`'s own order.
pub fn resolve_claude_home() -> Result<ClaudeHome> {
    if let Some(dir) = heimdall_claude_home_override() {
        return if dir.is_absolute() {
            Ok(ClaudeHome::new(&dir, dir.join(".claude.json")))
        } else {
            Err(AppError::invalid_input("HEIMDALL_CLAUDE_HOME must be an absolute path"))
        };
    }
    ClaudeHome::from_env()
}

struct CacheEntry {
    mtime: SystemTime,
    size: u64,
    area: Area,
    value: Box<dyn Any + Send + Sync>,
}

/// A path-keyed cache of arbitrary loaded values (`Project`s, a session's messages, …), each
/// stamped with the (mtime, size) of the file it was read from and the [`Area`] it belongs to.
/// Freshness is checked by the caller passing in the file's *current* metadata — the cache itself
/// never touches disk, so every read stays off the async runtime's own thread for free and stays
/// trivial to test without a real filesystem.
#[derive(Default)]
pub struct Cache {
    entries: Mutex<HashMap<PathBuf, CacheEntry>>,
}

impl Cache {
    /// The cached value for `path`, if one exists and `mtime`/`size` still match what was cached —
    /// a change to the file (or a metadata read that could not be taken) is a miss, never a stale hit.
    pub fn get<T: Clone + Send + Sync + 'static>(&self, path: &Path, mtime: SystemTime, size: u64) -> Option<T> {
        let entries = self.entries.lock().expect("cache lock");
        let entry = entries.get(path)?;
        if entry.mtime == mtime && entry.size == size { entry.value.downcast_ref::<T>().cloned() } else { None }
    }

    pub fn put<T: Send + Sync + 'static>(&self, path: impl Into<PathBuf>, mtime: SystemTime, size: u64, area: Area, value: T) {
        let mut entries = self.entries.lock().expect("cache lock");
        entries.insert(path.into(), CacheEntry { mtime, size, area, value: Box::new(value) });
    }

    /// Drops every entry tagged with `area` (called with each `DataChange` the watcher emits, M1-47).
    #[allow(dead_code)] // no caller until the watcher (M1-47) lands
    pub fn invalidate_area(&self, area: Area) {
        self.entries.lock().expect("cache lock").retain(|_, e| e.area != area);
    }

    /// Drops every entry, regardless of area (a full rescan, or area `all`).
    #[allow(dead_code)] // no caller until the watcher (M1-47) lands
    pub fn clear(&self) {
        self.entries.lock().expect("cache lock").clear();
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.entries.lock().expect("cache lock").len()
    }
}

pub struct AppState {
    pub claude_home: ClaudeHome,
    pub cache: Cache,
}

impl AppState {
    pub fn new() -> Result<Self> {
        Ok(Self { claude_home: resolve_claude_home()?, cache: Cache::default() })
    }
}

/// Runs `f` on the blocking-task pool so disk work never runs on the async event loop (contract §1).
pub async fn blocking<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(f).await.expect("blocking task panicked")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn stamp(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    // Ported from PoirotTests/App/AppStateCacheTests.swift.
    #[test]
    fn cache_session_stores_and_retrieves() {
        let cache = Cache::default();
        cache.put("s.json", stamp(1), 10, Area::Sessions, "abc-123".to_string());
        assert_eq!(cache.get::<String>(Path::new("s.json"), stamp(1), 10), Some("abc-123".to_string()));
    }

    #[test]
    fn cached_session_returns_none_for_an_unknown_path() {
        let cache = Cache::default();
        assert_eq!(cache.get::<String>(Path::new("nonexistent"), stamp(1), 10), None);
    }

    #[test]
    fn clear_removes_all_entries() {
        let cache = Cache::default();
        cache.put("s1.json", stamp(1), 1, Area::Sessions, 1u32);
        cache.put("s2.json", stamp(1), 1, Area::Sessions, 2u32);
        cache.clear();
        assert_eq!((cache.get::<u32>(Path::new("s1.json"), stamp(1), 1), cache.get::<u32>(Path::new("s2.json"), stamp(1), 1)), (None, None));
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn a_changed_size_at_the_same_mtime_is_a_miss() {
        let cache = Cache::default();
        cache.put("f", stamp(5), 100, Area::Transcript, "old".to_string());
        assert_eq!(cache.get::<String>(Path::new("f"), stamp(5), 200), None);
    }

    #[test]
    fn a_changed_mtime_at_the_same_size_is_a_miss() {
        let cache = Cache::default();
        cache.put("f", stamp(5), 100, Area::Transcript, "old".to_string());
        assert_eq!(cache.get::<String>(Path::new("f"), stamp(6), 100), None);
    }

    #[test]
    fn a_stale_entry_stays_in_the_map_until_overwritten_or_invalidated() {
        // A miss on read never evicts by itself: the next `put` for the same path, once the caller
        // has reloaded, simply replaces it.
        let cache = Cache::default();
        cache.put("f", stamp(1), 1, Area::Transcript, "v1".to_string());
        assert_eq!(cache.get::<String>(Path::new("f"), stamp(2), 1), None);
        cache.put("f", stamp(2), 1, Area::Transcript, "v2".to_string());
        assert_eq!(cache.get::<String>(Path::new("f"), stamp(2), 1), Some("v2".to_string()));
    }

    #[test]
    fn invalidate_area_drops_only_matching_entries() {
        let cache = Cache::default();
        cache.put("sessions/a", stamp(1), 1, Area::Sessions, "a".to_string());
        cache.put("todos/b", stamp(1), 1, Area::Todos, "b".to_string());
        cache.invalidate_area(Area::Sessions);
        assert_eq!(cache.get::<String>(Path::new("sessions/a"), stamp(1), 1), None);
        assert_eq!(cache.get::<String>(Path::new("todos/b"), stamp(1), 1), Some("b".to_string()));
    }

    #[test]
    fn get_with_the_wrong_type_does_not_panic() {
        let cache = Cache::default();
        cache.put("f", stamp(1), 1, Area::Sessions, 42u32);
        assert_eq!(cache.get::<String>(Path::new("f"), stamp(1), 1), None);
    }

    #[test]
    fn blocking_runs_the_closure_and_returns_its_value() {
        assert_eq!(tauri::async_runtime::block_on(blocking(|| 1 + 1)), 2);
    }

    #[test]
    fn the_override_only_takes_effect_in_a_debug_build() {
        // `cfg!(debug_assertions)` reflects whichever profile actually compiled this test binary,
        // so running this under `cargo test --release` (debug-assertions off by default) is what
        // proves the release arm — not just this test's own inspection of the source.
        unsafe { std::env::set_var("HEIMDALL_CLAUDE_HOME", "/tmp/fixture-home") };
        let found = heimdall_claude_home_override();
        unsafe { std::env::remove_var("HEIMDALL_CLAUDE_HOME") };
        if cfg!(debug_assertions) {
            assert_eq!(found, Some(PathBuf::from("/tmp/fixture-home")));
        } else {
            assert_eq!(found, None, "a release build must never read HEIMDALL_CLAUDE_HOME");
        }
    }

    // Only meaningful in a debug build: in release, the override is never read at all, so this
    // path falls through to `ClaudeHome::from_env` instead of ever seeing this value.
    #[cfg(debug_assertions)]
    #[test]
    fn resolve_claude_home_rejects_a_relative_override() {
        unsafe { std::env::set_var("HEIMDALL_CLAUDE_HOME", "relative/dir") };
        let err = resolve_claude_home().unwrap_err();
        unsafe { std::env::remove_var("HEIMDALL_CLAUDE_HOME") };
        assert!(matches!(err, AppError::InvalidInput { .. }));
    }
}
