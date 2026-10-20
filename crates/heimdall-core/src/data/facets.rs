//! Facets loader (Services/FacetsLoader.swift): AI session summaries in
//! `~/.claude/usage-data/facets/<sessionId>.json`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::error::{AppError, Loaded, Result};
use crate::models::session_facets::SessionFacets;
use crate::paths::plain_file_name;

/// The session's facets; `None` when there is no file or it does not parse.
pub fn load(facets_dir: &Path, session_id: &str) -> Result<Option<SessionFacets>> {
    let path = facets_dir.join(format!("{}.json", plain_file_name(session_id)?));
    Ok(read(&path))
}

/// All facets keyed by session id. `skipped` counts `.json` files that do not parse.
pub fn load_all(facets_dir: &Path) -> Result<Loaded<BTreeMap<String, SessionFacets>>> {
    let entries = match std::fs::read_dir(facets_dir) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Loaded::new(BTreeMap::new(), 0)),
        Err(e) => return Err(AppError::from_io(&e, facets_dir)),
    };
    let mut facets = BTreeMap::new();
    let mut skipped = 0;
    for path in entries.flatten().map(|e| e.path()) {
        let Some(stem) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .filter(|s| !s.starts_with('.'))
        else {
            continue;
        };
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        match read(&path) {
            // Keyed by the id inside the file, as the Swift reference app does; the file name when that is missing.
            Some(f) => {
                let key = if f.session_id.is_empty() {
                    stem.to_string()
                } else {
                    f.session_id.clone()
                };
                facets.insert(key, f);
            }
            None => skipped += 1,
        }
    }
    Ok(Loaded::new(facets, skipped))
}

fn read(path: &Path) -> Option<SessionFacets> {
    serde_json::from_slice(&std::fs::read(path).ok()?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(dir: &Path, session_id: &str, goal: &str) {
        let json = format!(
            r#"{{"underlying_goal": "{goal}", "goal_categories": {{"testing": 1}}, "outcome": "success", "user_satisfaction_counts": {{}},
                "claude_helpfulness": "very_helpful", "session_type": "single_task", "friction_counts": {{}}, "friction_detail": "",
                "primary_success": "none", "brief_summary": "Test summary", "session_id": "{session_id}"}}"#
        );
        std::fs::write(dir.join(format!("{session_id}.json")), json).unwrap();
    }
    const MISSING: &str = "/nonexistent/path/8e2b4c1d";

    // Ported from PoirotTests/Services/FacetsLoaderTests.swift.
    #[test]
    fn load_facets_non_existent_path_returns_nil() {
        assert!(load(Path::new(MISSING), "some-session-id").unwrap().is_none());
    }
    #[test]
    fn load_facets_existing_file_returns_facets() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "abc-1234", "Refactor auth");
        let f = load(d.path(), "abc-1234").unwrap().unwrap();
        assert_eq!(
            (f.session_id.as_str(), f.underlying_goal.as_str()),
            ("abc-1234", "Refactor auth")
        );
    }
    #[test]
    fn load_facets_missing_file_returns_nil() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "other-session", "g");
        assert!(load(d.path(), "nonexistent-session").unwrap().is_none());
    }
    #[test]
    fn load_facets_invalid_json_returns_nil() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("bad-session.json"), "not valid json").unwrap();
        assert!(load(d.path(), "bad-session").unwrap().is_none());
    }
    #[test]
    fn load_all_facets_non_existent_path_returns_empty() {
        assert!(load_all(Path::new(MISSING)).unwrap().value.is_empty());
    }
    #[test]
    fn load_all_facets_empty_directory_returns_empty() {
        let d = tempfile::tempdir().unwrap();
        assert!(load_all(d.path()).unwrap().value.is_empty());
    }
    #[test]
    fn load_all_facets_multiple_files_returns_all() {
        let d = tempfile::tempdir().unwrap();
        for i in 1..=3 {
            write(d.path(), &format!("session-{i}"), &format!("Goal {i}"));
        }
        let all = load_all(d.path()).unwrap().value;
        assert_eq!(all.len(), 3);
        assert!((1..=3).all(|i| all[&format!("session-{i}")].underlying_goal == format!("Goal {i}")));
    }
    #[test]
    fn load_all_facets_skips_invalid_files() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "good-session", "Good goal");
        std::fs::write(d.path().join("bad-session.json"), "not json").unwrap();
        let loaded = load_all(d.path()).unwrap();
        assert_eq!(
            (
                loaded.value.len(),
                loaded.value.contains_key("good-session"),
                loaded.skipped
            ),
            (1, true, 1)
        );
    }
    #[test]
    fn load_all_facets_skips_non_json_files() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "good-session", "g");
        std::fs::write(d.path().join("readme.txt"), "not a facets file").unwrap();
        assert_eq!(load_all(d.path()).unwrap().value.len(), 1);
    }
    #[test]
    fn load_all_facets_keyed_by_session_id() {
        let d = tempfile::tempdir().unwrap();
        write(d.path(), "00163b58-3b30-49fa-af28-052f06724f50", "UUID session");
        assert_eq!(
            load_all(d.path()).unwrap().value["00163b58-3b30-49fa-af28-052f06724f50"].underlying_goal,
            "UUID session"
        );
    }

    #[test]
    fn a_file_without_session_id_is_keyed_by_its_name_and_ids_stay_in_the_folder() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("s9.json"), r#"{"underlying_goal": "g"}"#).unwrap();
        assert!(load_all(d.path()).unwrap().value.contains_key("s9"));
        assert!(load(d.path(), "../s9").is_err());
    }
}
