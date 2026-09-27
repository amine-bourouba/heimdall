//! Maps a changed path under `~/.claude` to a data area and, where the contract calls for one, a
//! key naming which project, session or memory source it belongs to (ADR 0006, contract §7).
//!
//! Paths are plain strings compared through [`paths::normalize`] and an OS-aware case rule, the
//! same convention `paths::same_file` uses, so Windows- and Unix-shaped paths both test correctly
//! from any host — the watcher that calls this (M1-47) always runs on one real OS, but this
//! function's own test suite does not.
//!
//! `classify` never returns [`Area::Prefs`] (preferences live outside `~/.claude`; `prefs.rs`,
//! M1-39, emits that area itself when it saves) and only returns [`Area::All`] for the Claude home
//! root itself, a rare, drastic event; the watcher's own overflow handling (M1-47) is the usual
//! source of `all`.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::data::todos;
use crate::paths::{self, TargetOs};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum Area {
    Projects,
    Sessions,
    Transcript,
    Todos,
    History,
    DebugLogs,
    Facets,
    FileHistory,
    Stats,
    Commands,
    Skills,
    Plans,
    OutputStyles,
    Agents,
    Hooks,
    Settings,
    Mcp,
    Plugins,
    Memory,
    Prefs,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DataChange {
    pub area: Area,
    pub key: Option<String>,
}

fn change(area: Area, key: Option<String>) -> DataChange {
    DataChange { area, key }
}

/// `path`, `claude_home_root` and `config_file` (`~/.claude.json`) are plain path strings; `os`
/// picks the separator and case rule. Only two paths ever classify to more than one area:
/// `~/.claude` itself (`all`) and `settings.json` (`settings` and `hooks`, since one file holds
/// both and each has its own store to notify).
pub fn classify(path: &str, claude_home_root: &str, config_file: &str, os: TargetOs) -> Vec<DataChange> {
    if paths::same_path(path, config_file, os) {
        return vec![change(Area::Mcp, None)];
    }
    let Some(rest) = strip_root(path, claude_home_root, os) else {
        return Vec::new();
    };
    let parts: Vec<&str> = rest.split('/').filter(|s| !s.is_empty()).collect();
    match parts.as_slice() {
        [] => vec![change(Area::All, None)],
        ["projects"] => vec![change(Area::Projects, None)],
        ["projects", _name] => vec![change(Area::Projects, None)],
        ["projects", name, "sessions-index.json"] => vec![change(Area::Sessions, Some((*name).to_string()))],
        ["projects", name, "memory", ..] => vec![change(Area::Memory, Some(format!("project:{name}")))],
        // A session's transcript appearing or changing can add or rename a row in its project's
        // own session list (M1-41's `list_sessions`), not just the transcript's own content — both
        // areas change together, the same reasoning `settings.json` already gets two areas for.
        ["projects", name, file] if file.ends_with(".jsonl") => {
            vec![
                change(Area::Transcript, file.strip_suffix(".jsonl").map(String::from)),
                change(Area::Sessions, Some((*name).to_string())),
            ]
        }
        ["projects", name, _session, "subagents", file] => {
            let key = file.strip_suffix(".jsonl").or_else(|| file.strip_suffix(".meta.json"));
            key.map_or_else(Vec::new, |k| {
                vec![
                    change(Area::Transcript, Some(k.to_string())),
                    change(Area::Sessions, Some((*name).to_string())),
                ]
            })
        }
        ["projects", _name, _session, "tool-results", ..] => Vec::new(), // persisted tool output; Heimdall does not read it
        ["todos", file] => vec![change(
            Area::Todos,
            Some(todos::session_id_of(file.trim_end_matches(".json")).to_string()),
        )],
        ["debug", file] => vec![change(Area::DebugLogs, Some(file.trim_end_matches(".txt").to_string()))],
        ["file-history", session, ..] => vec![change(Area::FileHistory, Some((*session).to_string()))],
        ["usage-data", "facets", file] => vec![change(Area::Facets, Some(file.trim_end_matches(".json").to_string()))],
        ["usage-data", ..] => Vec::new(),
        ["history.jsonl"] => vec![change(Area::History, None)],
        ["stats-cache.json"] => vec![change(Area::Stats, None)],
        ["commands", ..] => vec![change(Area::Commands, None)],
        ["skills", "synced", ..] => Vec::new(), // Claude Code's synced-skills layout; not read (parity E12)
        ["skills", ..] => vec![change(Area::Skills, None)],
        ["plans", ..] => vec![change(Area::Plans, None)],
        ["output-styles", ..] => vec![change(Area::OutputStyles, None)],
        ["agents", ..] => vec![change(Area::Agents, None)],
        ["settings.json"] => vec![change(Area::Settings, None), change(Area::Hooks, None)],
        ["mcp-needs-auth-cache.json"] => vec![change(Area::Mcp, None)],
        ["plugins", "installed_plugins.json"] => vec![change(Area::Plugins, None)],
        _ => Vec::new(),
    }
}

/// `path` relative to `root` as `/`-joined components, or `None` when `path` is not under `root`.
/// Case-insensitive on Windows and macOS ([`TargetOs::case_insensitive`]); the remainder keeps its
/// original case regardless, since project folder names and session ids are meaningful data.
fn strip_root(path: &str, root: &str, os: TargetOs) -> Option<String> {
    let norm_path = paths::normalize(path);
    let norm_root = paths::normalize(root);
    if norm_path.len() < norm_root.len() {
        return None;
    }
    let (head, rest) = norm_path.split_at(norm_root.len());
    let matches = if os.case_insensitive() {
        head.eq_ignore_ascii_case(&norm_root)
    } else {
        head == norm_root
    };
    if !matches {
        return None;
    }
    match rest.strip_prefix('/') {
        Some(r) => Some(r.to_string()),
        None if rest.is_empty() => Some(String::new()),
        None => None, // head matched a longer sibling name, e.g. root "/a/b" against "/a/bc"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn area_names_match_the_contract_exactly() {
        let pairs = [
            (Area::DebugLogs, "debugLogs"),
            (Area::FileHistory, "fileHistory"),
            (Area::OutputStyles, "outputStyles"),
            (Area::Mcp, "mcp"),
            (Area::Prefs, "prefs"),
            (Area::All, "all"),
            (Area::Projects, "projects"),
        ];
        for (area, name) in pairs {
            assert_eq!(serde_json::to_value(area).unwrap(), name);
        }
    }

    const WIN_HOME: &str = r"C:\Users\me\.claude";
    const WIN_CONFIG: &str = r"C:\Users\me\.claude.json";
    const UNIX_HOME: &str = "/home/me/.claude";
    const UNIX_CONFIG: &str = "/home/me/.claude.json";

    fn win(path: &str) -> Vec<DataChange> {
        classify(path, WIN_HOME, WIN_CONFIG, TargetOs::Windows)
    }
    fn unix(path: &str) -> Vec<DataChange> {
        classify(path, UNIX_HOME, UNIX_CONFIG, TargetOs::Linux)
    }
    fn one(area: Area, key: Option<&str>) -> Vec<DataChange> {
        vec![change(area, key.map(String::from))]
    }

    /// One table, run against both a Windows-shaped and a Unix-shaped Claude home, covering every
    /// area `classify` can produce (`prefs` is emitted by `prefs.rs`, M1-39, never from a path).
    #[test]
    fn every_reachable_area_classifies_from_both_windows_and_unix_paths() {
        let cases: &[(&str, Vec<DataChange>)] = &[
            (r"\projects\-Users-me-app", one(Area::Projects, None)),
            (
                r"\projects\-Users-me-app\sessions-index.json",
                one(Area::Sessions, Some("-Users-me-app")),
            ),
            (
                r"\projects\-Users-me-app\11111111-1111-4111-8111-111111111111.jsonl",
                vec![
                    change(Area::Transcript, Some("11111111-1111-4111-8111-111111111111".into())),
                    change(Area::Sessions, Some("-Users-me-app".into())),
                ],
            ),
            (
                r"\projects\-Users-me-app\11111111-1111-4111-8111-111111111111\subagents\agent-a9b1c2d3.jsonl",
                vec![
                    change(Area::Transcript, Some("agent-a9b1c2d3".into())),
                    change(Area::Sessions, Some("-Users-me-app".into())),
                ],
            ),
            (
                r"\projects\-Users-me-app\11111111-1111-4111-8111-111111111111\subagents\agent-a9b1c2d3.meta.json",
                vec![
                    change(Area::Transcript, Some("agent-a9b1c2d3".into())),
                    change(Area::Sessions, Some("-Users-me-app".into())),
                ],
            ),
            (
                r"\projects\-Users-me-app\memory\MEMORY.md",
                one(Area::Memory, Some("project:-Users-me-app")),
            ),
            (
                r"\todos\11111111-1111-4111-8111-111111111111-agent-a9b1c2d3.json",
                one(Area::Todos, Some("11111111-1111-4111-8111-111111111111")),
            ),
            (
                r"\todos\11111111-1111-4111-8111-111111111111.json",
                one(Area::Todos, Some("11111111-1111-4111-8111-111111111111")),
            ),
            (r"\history.jsonl", one(Area::History, None)),
            (
                r"\debug\11111111-1111-4111-8111-111111111111.txt",
                one(Area::DebugLogs, Some("11111111-1111-4111-8111-111111111111")),
            ),
            (
                r"\usage-data\facets\11111111-1111-4111-8111-111111111111.json",
                one(Area::Facets, Some("11111111-1111-4111-8111-111111111111")),
            ),
            (
                r"\file-history\11111111-1111-4111-8111-111111111111\3f2504e5f2a14f9f@v1",
                one(Area::FileHistory, Some("11111111-1111-4111-8111-111111111111")),
            ),
            (r"\stats-cache.json", one(Area::Stats, None)),
            (r"\commands\review.md", one(Area::Commands, None)),
            (r"\skills\pdf-tools\SKILL.md", one(Area::Skills, None)),
            (r"\plans\refactor-auth-module.md", one(Area::Plans, None)),
            (r"\output-styles\concise.md", one(Area::OutputStyles, None)),
            (r"\agents\reviewer.md", one(Area::Agents, None)),
            (r"\plugins\installed_plugins.json", one(Area::Plugins, None)),
            (r"\mcp-needs-auth-cache.json", one(Area::Mcp, None)),
        ];
        for (suffix, expected) in cases {
            let win_path = format!(r"{WIN_HOME}\{}", suffix.trim_start_matches('\\'));
            assert_eq!(win(&win_path), *expected, "windows: {suffix}");
            let unix_path = format!("{UNIX_HOME}{}", suffix.replace('\\', "/"));
            assert_eq!(unix(&unix_path), *expected, "unix: {suffix}");
        }
    }

    #[test]
    fn the_claude_home_root_itself_classifies_as_all() {
        assert_eq!(win(WIN_HOME), one(Area::All, None));
        assert_eq!(unix(UNIX_HOME), one(Area::All, None));
    }

    // Correction found while building M1-47 (the watcher): a session transcript appearing,
    // changing or being renamed over can add or rename a row in `list_sessions`, so it must
    // invalidate `sessions` (keyed by the project id) as well as `transcript` (keyed by the
    // session id) — a plain `.jsonl` path classifying to `transcript` alone would leave a stale
    // session list cached after exactly the kind of change the watcher exists to catch.
    #[test]
    fn a_session_transcript_classifies_to_both_transcript_and_its_projects_sessions_area() {
        let expected = vec![
            change(Area::Transcript, Some("11111111-1111-4111-8111-111111111111".into())),
            change(Area::Sessions, Some("-Users-me-app".into())),
        ];
        assert_eq!(
            win(r"C:\Users\me\.claude\projects\-Users-me-app\11111111-1111-4111-8111-111111111111.jsonl"),
            expected
        );
    }

    #[test]
    fn settings_json_classifies_as_both_settings_and_hooks() {
        let expected = vec![change(Area::Settings, None), change(Area::Hooks, None)];
        assert_eq!(win(r"C:\Users\me\.claude\settings.json"), expected);
        assert_eq!(unix("/home/me/.claude/settings.json"), expected);
    }

    #[test]
    fn the_config_file_classifies_as_mcp_even_though_it_sits_outside_the_home_root() {
        assert_eq!(win(WIN_CONFIG), one(Area::Mcp, None));
        assert_eq!(unix(UNIX_CONFIG), one(Area::Mcp, None));
    }

    #[test]
    fn synced_skills_and_persisted_tool_output_are_not_read() {
        assert!(win(r"C:\Users\me\.claude\skills\synced\abc\tidy\SKILL.md").is_empty());
        assert!(
            win(r"C:\Users\me\.claude\projects\p\11111111-1111-4111-8111-111111111111\tool-results\x.txt").is_empty()
        );
        assert!(win(r"C:\Users\me\.claude\usage-data\something-else.json").is_empty());
    }

    #[test]
    fn unrelated_paths_return_nothing() {
        for p in [
            r"C:\Users\me\Desktop\notes.txt",
            r"C:\Users\other\.claude\settings.json",
            "/etc/passwd",
            r"C:\Users\me\.claude-backup\settings.json",
        ] {
            assert!(
                win(p).is_empty() && classify(p, UNIX_HOME, UNIX_CONFIG, TargetOs::Linux).is_empty(),
                "{p}"
            );
        }
    }

    #[test]
    fn windows_and_macos_compare_the_root_case_insensitively_but_keep_the_remainder_as_written() {
        let hit = classify(
            r"c:\users\ME\.CLAUDE\commands\Review.md",
            WIN_HOME,
            WIN_CONFIG,
            TargetOs::Windows,
        );
        assert_eq!(hit, one(Area::Commands, None));
        // On Linux the same casing mismatch is a different, unrelated path.
        assert!(
            classify(
                r"c:\users\ME\.CLAUDE\commands\Review.md",
                WIN_HOME,
                WIN_CONFIG,
                TargetOs::Linux
            )
            .is_empty()
        );
    }

    #[test]
    fn a_sibling_folder_that_only_shares_a_prefix_is_not_mistaken_for_the_root() {
        // "/home/me/.claude-extra" must not be treated as "/home/me/.claude" plus "-extra".
        assert!(unix("/home/me/.claude-extra/settings.json").is_empty());
    }
}
