//! Cross-OS fixture conformance (task M1-54): every loader reads the four committed fixture
//! trees (`fixtures/{synthetic,screenshot,windows,linux}/claude-home/`) with no unexpected
//! skipped lines, and project-path resolution never falls back to decoding a folder name where a
//! transcript's own `cwd` was actually available (ADR 0003's resolution order).
//!
//! Runs unconditionally on every OS (no `#[cfg(target_os)]` anywhere in this file) — ADR 0003:
//! "every OS branch of every ... path rule runs in CI on every host, because core never uses
//! `#[cfg]` for decisions." `TargetOs` is passed as data instead, one value per fixture set,
//! matching the platform each set was actually captured on.

use std::path::{Path, PathBuf};

use heimdall_core::config::read::{agents, commands, hooks, mcp, output_styles, plans, plugins, skills};
use heimdall_core::data::{facets, file_history, history, todos};
use heimdall_core::paths::{ClaudeHome, TargetOs};
use heimdall_core::projects::discovery;
use heimdall_core::transcript::full::{self, ParseContext};

fn fixtures_root() -> PathBuf {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures")).to_path_buf()
}

/// One fixture set's `claude-home/` root, plus the OS it was actually captured on (or modelled
/// after, for the hand-built sets) — this is what governs case-sensitivity and drive-letter
/// handling in project-path resolution, not a fixed choice.
struct Set {
    name: &'static str,
    claude_home: PathBuf,
    os: TargetOs,
}

fn sets() -> Vec<Set> {
    [
        ("synthetic", TargetOs::Macos),
        ("screenshot", TargetOs::Macos),
        ("windows", TargetOs::Windows),
        ("linux", TargetOs::Linux),
    ]
    .into_iter()
    .map(|(name, os)| Set {
        name,
        claude_home: fixtures_root().join(name).join("claude-home"),
        os,
    })
    .collect()
}

/// Every session id under a set's `projects/`, discovered once and reused across the several
/// per-session loaders (file history, debug log) so each fixture is only walked a handful of
/// times, not once per assertion.
fn discovered(set: &Set) -> Vec<heimdall_core::models::project::Project> {
    let loaded = discovery::discover(&set.claude_home.join("projects"), set.os).unwrap();
    assert_eq!(
        loaded.skipped, 0,
        "{}: project/session discovery skipped {} lines/files",
        set.name, loaded.skipped
    );
    loaded.value
}

#[test]
fn every_fixture_set_discovers_projects_with_no_skips_and_no_unnecessary_decoding() {
    for set in sets() {
        let projects = discovered(&set);
        for p in &projects {
            let has_cwd_data = p.sessions.iter().any(|s| !s.cwds.is_empty());
            if has_cwd_data {
                assert_ne!(
                    p.path_source,
                    Some(heimdall_core::models::project::PathSource::Decoded),
                    "{}: project {} had real cwd data but still fell back to decoding its folder name",
                    set.name,
                    p.id
                );
            }
        }
    }
}

/// AC2, stated at this cross-fixture suite's own level (in addition to the existing
/// `projects::discovery` unit test that first established it): every Windows fixture project
/// that has any sessions resolves via a transcript `cwd`, never the sessions index or decoding —
/// fixtures/windows has no `sessions-index.json` files at all (M0-14's own MANIFEST), so
/// `TranscriptCwd` is the only source that should ever fire there.
#[test]
fn windows_projects_report_path_source_transcript_cwd() {
    let windows = Set {
        name: "windows",
        claude_home: fixtures_root().join("windows").join("claude-home"),
        os: TargetOs::Windows,
    };
    let projects = discovered(&windows);
    let with_sessions: Vec<_> = projects.iter().filter(|p| !p.sessions.is_empty()).collect();
    assert!(
        !with_sessions.is_empty(),
        "windows fixture has no projects with sessions to assert against"
    );
    for p in with_sessions {
        assert_eq!(
            p.path_source,
            Some(heimdall_core::models::project::PathSource::TranscriptCwd),
            "project {}",
            p.id
        );
    }
}

/// AC1: a transcript captured with CRLF line endings parses to exactly the same messages as its
/// LF original — everything except the file path itself (which legitimately differs) and the
/// project path (supplied externally by the caller, not derived from file content).
#[test]
fn crlf_and_lf_variants_of_the_same_transcript_parse_identically() {
    let session_id = "7f33ce70-8d2c-4c0c-adc5-64f1aee23b5e";
    let project_dir = "c--Users-fixture-Desktop-tech-proj-11";
    let lf_path = fixtures_root()
        .join("windows/claude-home/projects")
        .join(project_dir)
        .join(format!("{session_id}.jsonl"));
    let crlf_path = fixtures_root()
        .join("windows/crlf/claude-home/projects")
        .join(project_dir)
        .join(format!("{session_id}.jsonl"));
    assert!(
        lf_path.is_file() && crlf_path.is_file(),
        "fixture paths moved — update this test"
    );

    let ctx = ParseContext {
        project_path: "/same/for/both",
        session_id,
        index_started_at: None,
        is_sidechain: false,
    };
    let lf = full::parse(&lf_path, ctx)
        .unwrap()
        .value
        .expect("LF transcript failed to parse");
    let crlf = full::parse(&crlf_path, ctx)
        .unwrap()
        .value
        .expect("CRLF transcript failed to parse");

    assert_eq!(lf.messages, crlf.messages);
    assert_eq!(
        (
            lf.started_at,
            lf.ended_at,
            &lf.model,
            lf.total_tokens,
            lf.tokens,
            lf.is_sidechain,
            &lf.searchable_text
        ),
        (
            crlf.started_at,
            crlf.ended_at,
            &crlf.model,
            crlf.total_tokens,
            crlf.tokens,
            crlf.is_sidechain,
            &crlf.searchable_text
        )
    );
}

/// AC1 ("no unexpected skipped lines"), for every loader that tracks skips via `Loaded<T>`, across
/// all four sets. Loaders with no fixture data for a set (e.g. `fixtures/linux` has no `todos/` at
/// all) still return `Loaded` with `skipped: 0` for a missing directory — that is the documented,
/// correct behaviour (config/read/mod.rs and data/*.rs both treat "not found" as empty, not an
/// error), not something this test needs to special-case per set.
#[test]
fn every_loaded_returning_loader_has_zero_skips_on_every_set() {
    for set in sets() {
        let home = ClaudeHome::new(
            set.claude_home.clone(),
            set.claude_home.parent().unwrap().join("claude.json"),
        );
        let projects = discovered(&set);

        let check = |what: &str, skipped: u32| assert_eq!(skipped, 0, "{}: {what} skipped {skipped}", set.name);

        check(
            "history",
            history::load_all(&set.claude_home.join("history.jsonl"))
                .unwrap()
                .skipped,
        );
        check(
            "todos (flat)",
            todos::load_all(&set.claude_home.join("todos")).unwrap().skipped,
        );
        check(
            "todos (lists)",
            todos::load_all_lists(&set.claude_home.join("todos")).unwrap().skipped,
        );
        check(
            "facets",
            facets::load_all(&set.claude_home.join("usage-data/facets"))
                .unwrap()
                .skipped,
        );
        check("agents", agents::load(&set.claude_home).skipped);
        check("commands", commands::load(&set.claude_home, None).skipped);
        check("output_styles", output_styles::load(&set.claude_home, None).skipped);
        check("plans", plans::load(&set.claude_home).skipped);
        check("skills", skills::load(&set.claude_home, None).skipped);
        check("hooks", hooks::load(&set.claude_home, None).skipped);
        check("plugins", plugins::load(&home).skipped);
        check("mcp", mcp::load(&home, None, set.os).skipped);

        for p in &projects {
            for s in &p.sessions {
                check(
                    &format!("file_history({})", s.id),
                    file_history::load(&set.claude_home.join("projects"), &s.id, set.os)
                        .unwrap()
                        .skipped,
                );
            }
        }
    }
}

/// `settings.json` and the pre-computed `stats-cache.json` don't track skips (a settings parse
/// failure is a hard error by design, and the stats cache is a single JSON document, not a
/// collection of independently-skippable records) — checked here only for "does not error /
/// panic", which is still real conformance coverage distinct from the `Loaded`-based check above.
#[test]
fn settings_and_stats_cache_load_without_error_on_every_set() {
    for set in sets() {
        let settings_result = heimdall_core::config::read::settings::load(&set.claude_home.join("settings.json"));
        assert!(
            settings_result.is_ok(),
            "{}: settings.json failed to load: {:?}",
            set.name,
            settings_result.err()
        );

        // A missing/absent cache is `None`, not an error — only a genuinely malformed file would
        // be a real conformance problem, and none of these fixtures ship one under this name.
        let _ = heimdall_core::data::stats_cache::load(&set.claude_home.join("stats-cache.json"));
    }
}
