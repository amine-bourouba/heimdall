//! `~/.claude/history.jsonl` reader (Services/HistoryLoader.swift): every prompt typed into
//! Claude Code, newest first, in pages.
//!
//! The Swift reference app's ids were `<timestamp>-<line index>`, so deleting after Claude Code appended a line
//! removed the wrong one (port-plan C-23). Here an id is derived from the line itself
//! ([`entry_id`]), so the delete command (M3-46) can find the same line again.

use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::Path;

use chrono::DateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use specta::Type;

use crate::error::{AppError, Loaded, Result};
use crate::models::history_entry::HistoryEntry;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HistoryPage {
    pub entries: Vec<HistoryEntry>,
    pub offset: u32,
    pub total: u32,
    /// Lines that did not parse as history entries.
    pub skipped: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawEntry {
    display: String,
    #[serde(default)]
    pasted_contents: Option<BTreeMap<String, Value>>,
    timestamp: f64,
    project: String,
}

/// `<timestamp ms>-<hash of the line>`, plus `-<n>` for the n-th repeat of an identical line.
pub fn entry_id(line: &str, timestamp_ms: i64, occurrence: u32) -> String {
    let hash = crate::stable_hash(line.as_bytes());
    match occurrence {
        0 => format!("{timestamp_ms}-{hash:016x}"),
        n => format!("{timestamp_ms}-{hash:016x}-{n}"),
    }
}

/// Every entry, newest first. A missing file is an empty history.
pub fn load_all(path: &Path) -> Result<Loaded<Vec<HistoryEntry>>> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(Loaded::new(Vec::new(), 0)),
        Err(e) => return Err(AppError::from_io(&e, path)),
    };
    let mut entries = Vec::new();
    let mut skipped = 0;
    let mut repeats: HashMap<u64, u32> = HashMap::new();
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = match line {
            Ok(line) => line,
            Err(e) if e.kind() == ErrorKind::InvalidData => {
                skipped += 1; // not UTF-8
                continue;
            }
            Err(e) => return Err(AppError::from_io(&e, path)),
        };
        let line = if i == 0 {
            line.strip_prefix('\u{feff}').unwrap_or(&line)
        } else {
            &line
        };
        if line.trim().is_empty() {
            continue;
        }
        let occurrence = repeats.entry(crate::stable_hash(line.as_bytes())).or_default();
        match parse_line(line, *occurrence) {
            Some(entry) => entries.push(entry),
            None => skipped += 1,
        }
        *occurrence += 1;
    }
    entries.sort_by_key(|e| std::cmp::Reverse(e.timestamp));
    Ok(Loaded::new(entries, skipped))
}

/// `limit` entries starting at `offset`, newest first.
pub fn page(path: &Path, offset: u32, limit: u32) -> Result<HistoryPage> {
    let loaded = load_all(path)?;
    let total = loaded.value.len() as u32;
    let entries = loaded
        .value
        .into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    Ok(HistoryPage {
        entries,
        offset,
        total,
        skipped: loaded.skipped,
    })
}

/// Non-empty lines, counted without parsing (HistoryLoader.entryCount), for the sidebar badge.
pub fn entry_count(path: &Path) -> Result<u32> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(0),
        Err(e) => return Err(AppError::from_io(&e, path)),
    };
    let mut reader = BufReader::new(file);
    let mut buf = Vec::new();
    let mut count = 0;
    loop {
        buf.clear();
        if reader
            .read_until(b'\n', &mut buf)
            .map_err(|e| AppError::from_io(&e, path))?
            == 0
        {
            return Ok(count);
        }
        if !buf.iter().all(u8::is_ascii_whitespace) {
            count += 1;
        }
    }
}

/// Distinct project paths, in order of their newest entry.
pub fn unique_projects(path: &Path) -> Result<Vec<String>> {
    let mut seen = std::collections::HashSet::new();
    Ok(load_all(path)?
        .value
        .into_iter()
        .map(|e| e.project)
        .filter(|p| seen.insert(p.clone()))
        .collect())
}

fn parse_line(line: &str, occurrence: u32) -> Option<HistoryEntry> {
    let raw: RawEntry = serde_json::from_str(line).ok()?;
    let ms = raw.timestamp as i64;
    Some(HistoryEntry {
        id: entry_id(line, ms, occurrence),
        display: raw.display,
        // The Swift reference app expected string values and dropped the whole line otherwise; pasted items stored
        // as objects keep their `content`.
        pasted_contents: raw
            .pasted_contents
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                let text = match &v {
                    Value::String(s) => s.clone(),
                    other => other
                        .get("content")
                        .and_then(Value::as_str)
                        .map_or_else(|| other.to_string(), String::from),
                };
                (k, text)
            })
            .collect(),
        timestamp: DateTime::from_timestamp_millis(ms)?,
        project: raw.project,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn file(content: &str) -> (tempfile::TempDir, std::path::PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.jsonl");
        std::fs::write(&path, content).unwrap();
        (dir, path)
    }
    fn line(display: &str, ts: u64, project: &str) -> String {
        format!(r#"{{"display":"{display}","pastedContents":{{}},"timestamp":{ts},"project":"{project}"}}"#)
    }
    fn displays(entries: &[HistoryEntry]) -> Vec<&str> {
        entries.iter().map(|e| e.display.as_str()).collect()
    }
    const MISSING: &str = "/nonexistent/7f0c6a1e/history.jsonl";

    // Ported from the Swift reference's own test suite (Services/HistoryLoaderTests.swift).
    #[test]
    fn load_all_non_existent_file_returns_empty() {
        assert!(load_all(Path::new(MISSING)).unwrap().value.is_empty());
    }
    #[test]
    fn load_all_empty_file_returns_empty() {
        let (_d, p) = file("");
        assert!(load_all(&p).unwrap().value.is_empty());
    }
    #[test]
    fn load_all_single_entry_parses_correctly() {
        let (_d, p) = file(&line("hello world", 1_700_000_000_000, "/Users/test/my-project"));
        let e = load_all(&p).unwrap().value;
        assert_eq!(
            (
                e.len(),
                e[0].display.as_str(),
                e[0].project.as_str(),
                e[0].project_name()
            ),
            (1, "hello world", "/Users/test/my-project", "my-project")
        );
    }
    #[test]
    fn load_all_multiple_entries_sorted_by_timestamp_descending() {
        let (_d, p) = file(
            &[
                line("first", 1_700_000_000_000, "/a"),
                line("second", 1_700_000_002_000, "/b"),
                line("third", 1_700_000_001_000, "/c"),
            ]
            .join("\n"),
        );
        assert_eq!(displays(&load_all(&p).unwrap().value), ["second", "third", "first"]);
    }
    #[test]
    fn load_all_skips_invalid_lines() {
        let (_d, p) = file(
            &[
                line("valid", 1_700_000_000_000, "/a"),
                "not valid json".into(),
                line("also valid", 1_700_000_001_000, "/b"),
            ]
            .join("\n"),
        );
        let loaded = load_all(&p).unwrap();
        assert_eq!((loaded.value.len(), loaded.skipped), (2, 1));
    }
    #[test]
    fn load_all_handles_pasted_contents() {
        let (_d, p) = file(
            r#"{"display":"test","pastedContents":{"file.txt":"contents here"},"timestamp":1700000000000,"project":"/a"}"#,
        );
        assert_eq!(
            load_all(&p).unwrap().value[0].pasted_contents["file.txt"],
            "contents here"
        );
    }
    #[test]
    fn load_all_handles_null_pasted_contents() {
        let (_d, p) = file(r#"{"display":"test","pastedContents":null,"timestamp":1700000000000,"project":"/a"}"#);
        let e = load_all(&p).unwrap().value;
        assert!(e.len() == 1 && e[0].pasted_contents.is_empty());
    }
    #[test]
    fn load_all_timestamp_converted_from_milliseconds() {
        let (_d, p) = file(&line("test", 1_700_000_000_000, "/a"));
        assert_eq!(load_all(&p).unwrap().value[0].timestamp.timestamp(), 1_700_000_000);
    }
    #[test]
    fn entry_count_non_existent_file_returns_zero() {
        assert_eq!(entry_count(Path::new(MISSING)).unwrap(), 0);
    }
    #[test]
    fn entry_count_matches_line_count() {
        let (_d, p) = file(&[line("a", 1, "/a"), line("b", 2, "/b"), line("c", 3, "/c")].join("\n"));
        assert_eq!(entry_count(&p).unwrap(), 3);
    }
    #[test]
    fn unique_projects_returns_distinct_paths() {
        let (_d, p) = file(
            &[
                line("a", 1_700_000_002_000, "/project-a"),
                line("b", 1_700_000_001_000, "/project-b"),
                line("c", 1_700_000_000_000, "/project-a"),
            ]
            .join("\n"),
        );
        assert_eq!(unique_projects(&p).unwrap(), ["/project-a", "/project-b"]);
    }
    #[test]
    fn unique_projects_non_existent_file_returns_empty() {
        assert!(unique_projects(Path::new(MISSING)).unwrap().is_empty());
    }

    #[test]
    fn ids_follow_the_line_not_its_position() {
        let a = line("a", 1_700_000_000_000, "/a");
        let b = line("b", 1_700_000_001_000, "/b");
        let (_d, p) = file(&format!("{a}\n{b}\n"));
        let before: Vec<_> = load_all(&p).unwrap().value.into_iter().map(|e| e.id).collect();
        // A line inserted before them moves their positions but not their ids.
        std::fs::write(&p, format!("{}\n{a}\n{b}\n", line("new", 1_600_000_000_000, "/n"))).unwrap();
        let after: Vec<_> = load_all(&p)
            .unwrap()
            .value
            .into_iter()
            .filter(|e| e.display != "new")
            .map(|e| e.id)
            .collect();
        assert_eq!(before, after);
        assert!(before[0].starts_with("1700000001000-"));
    }

    #[test]
    fn identical_lines_get_distinct_ids() {
        let a = line("same", 1_700_000_000_000, "/a");
        let (_d, p) = file(&format!("{a}\r\n{a}\r\n"));
        let ids: Vec<_> = load_all(&p).unwrap().value.into_iter().map(|e| e.id).collect();
        assert_eq!(ids.len(), 2);
        assert_ne!(ids[0], ids[1]);
    }

    #[test]
    fn object_pasted_contents_and_extra_keys_are_accepted() {
        let (_d, p) = file(
            r#"{"display":"x","pastedContents":{"1":{"id":1,"type":"text","content":"pasted"},"2":{"id":2,"type":"image"}},"timestamp":1700000000000,"project":"/a","sessionId":"s"}"#,
        );
        let e = &load_all(&p).unwrap().value[0];
        assert_eq!(e.pasted_contents["1"], "pasted");
        assert_eq!(e.pasted_contents["2"], r#"{"id":2,"type":"image"}"#);
    }

    #[test]
    fn pages_are_slices_of_the_newest_first_list() {
        let lines: Vec<_> = (0..5)
            .map(|i| line(&format!("e{i}"), 1_700_000_000_000 + i * 1000, "/p"))
            .collect();
        let (_d, p) = file(&lines.join("\n"));
        let pg = page(&p, 1, 2).unwrap();
        assert_eq!(
            (displays(&pg.entries), pg.offset, pg.total, pg.skipped),
            (vec!["e3", "e2"], 1, 5, 0)
        );
    }

    #[test]
    fn windows_fixture_history_parses_fully() {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows/claude-home/history.jsonl");
        let loaded = load_all(&p).unwrap();
        assert_eq!(
            (loaded.value.len() as u32, loaded.skipped),
            (entry_count(&p).unwrap(), 0)
        );
    }
}
