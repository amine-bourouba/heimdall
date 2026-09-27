//! Debug log loader (Services/DebugLogLoader.swift): `~/.claude/debug/<sessionId>.txt`, one
//! `<ISO 8601> [LEVEL] message` entry per line. Files are streamed, since they can be large;
//! a page keeps only the entries it returns.

use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

use crate::error::{AppError, Result};
use crate::models::debug_log_entry::{DebugLogEntry, LogLevel};
use crate::paths::plain_file_name;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct DebugLogPage {
    pub entries: Vec<DebugLogEntry>,
    pub offset: u32,
    /// Entries in the file, or of the requested level when filtered.
    pub total: u32,
}

/// `2026-02-06T16:01:34.872Z [DEBUG] message`: the timestamp is the text before the first `[`,
/// the level the text inside it (unknown levels read as debug), the message the rest. Lines
/// without a timestamp and a bracketed level are not entries.
pub fn parse_line(line: &str, index: u32) -> Option<DebugLogEntry> {
    let open = line.find('[')?;
    let timestamp: DateTime<Utc> = DateTime::parse_from_rfc3339(line[..open].trim()).ok()?.to_utc();
    let close = open + line[open..].find(']')?;
    Some(DebugLogEntry {
        id: index,
        timestamp,
        level: LogLevel::from_raw(&line[open + 1..close]).unwrap_or(LogLevel::Debug),
        message: line[close + 1..].trim().to_string(),
    })
}

/// Every entry of `text`, numbered from 0 in file order.
pub fn parse(text: &str) -> Vec<DebugLogEntry> {
    page_from(text.as_bytes(), 0, u32::MAX, None).entries
}

/// Entries `offset..offset + limit`, counting only `level` entries when a level is given. Ids stay
/// the entry's position in the whole file, so they do not change with the filter.
pub fn page_from(reader: impl BufRead, offset: u32, limit: u32, level: Option<LogLevel>) -> DebugLogPage {
    let mut entries = Vec::new();
    let mut index = 0;
    let mut matched = 0;
    for line in split_lines(reader) {
        let Some(entry) = parse_line(line.trim(), index) else {
            continue;
        };
        index += 1;
        if level.is_some_and(|l| l != entry.level) {
            continue;
        }
        if matched >= offset && matched - offset < limit {
            entries.push(entry);
        }
        matched += 1;
    }
    DebugLogPage {
        entries,
        offset,
        total: matched,
    }
}

/// A page of the session's log; a session without a log has an empty one.
pub fn page(
    debug_dir: &Path,
    session_id: &str,
    offset: u32,
    limit: u32,
    level: Option<LogLevel>,
) -> Result<DebugLogPage> {
    let path = log_path(debug_dir, session_id)?;
    match File::open(&path) {
        Ok(f) => Ok(page_from(BufReader::new(f), offset, limit, level)),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(DebugLogPage {
            entries: Vec::new(),
            offset,
            total: 0,
        }),
        Err(e) => Err(AppError::from_io(&e, &path)),
    }
}

/// Every entry of the session's log.
pub fn load(debug_dir: &Path, session_id: &str) -> Result<Vec<DebugLogEntry>> {
    Ok(page(debug_dir, session_id, 0, u32::MAX, None)?.entries)
}

pub fn has_log(debug_dir: &Path, session_id: &str) -> bool {
    log_path(debug_dir, session_id).is_ok_and(|p| p.is_file())
}

/// Ids of the sessions that have a log, sorted.
pub fn all_session_ids(debug_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(debug_dir) else {
        return Vec::new();
    };
    let mut ids: Vec<String> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .filter_map(|p| p.file_stem()?.to_str().map(String::from))
        .filter(|s| !s.starts_with('.'))
        .collect();
    ids.sort();
    ids
}

fn log_path(debug_dir: &Path, session_id: &str) -> Result<PathBuf> {
    Ok(debug_dir.join(format!("{}.txt", plain_file_name(session_id)?)))
}

/// Lines as text; invalid UTF-8 is replaced rather than ending the read.
fn split_lines(mut reader: impl BufRead) -> impl Iterator<Item = String> {
    let mut buf = Vec::new();
    std::iter::from_fn(move || {
        buf.clear();
        match reader.read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(String::from_utf8_lossy(&buf).into_owned()),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, Timelike};

    fn levels(entries: &[DebugLogEntry]) -> Vec<LogLevel> {
        entries.iter().map(|e| e.level).collect()
    }
    fn dir_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        for (name, content) in files {
            std::fs::write(dir.path().join(name), content).unwrap();
        }
        dir
    }
    fn ten(n: u32) -> String {
        (0..n)
            .map(|i| format!("2026-02-06T16:01:{i:02}.000Z [DEBUG] Message {i}"))
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn ids(p: &DebugLogPage) -> Vec<u32> {
        p.entries.iter().map(|e| e.id).collect()
    }
    const MISSING: &str = "/nonexistent/3c9d1e0f";

    // Ported from PoirotTests/Services/DebugLogLoaderTests.swift.
    #[test]
    fn parse_empty_string_returns_empty() {
        assert!(parse("").is_empty());
    }
    #[test]
    fn parse_single_debug_line() {
        let e = parse("2026-02-06T16:01:34.872Z [DEBUG] Loading MCP servers");
        assert_eq!(
            (e.len(), e[0].level, e[0].message.as_str(), e[0].id),
            (1, LogLevel::Debug, "Loading MCP servers", 0)
        );
    }
    #[test]
    fn parse_error_level() {
        let e = parse("2026-02-06T16:01:35.000Z [ERROR] MCP server failed");
        assert_eq!(
            (e[0].level, e[0].message.as_str()),
            (LogLevel::Error, "MCP server failed")
        );
    }
    #[test]
    fn parse_warn_level() {
        let e = parse("2026-02-06T16:01:35.000Z [WARN] Deprecated config");
        assert_eq!(
            (e[0].level, e[0].message.as_str()),
            (LogLevel::Warn, "Deprecated config")
        );
    }
    #[test]
    fn parse_multiple_lines() {
        let e = parse(
            "2026-02-06T16:01:34.872Z [DEBUG] First message\n2026-02-06T16:01:35.000Z [ERROR] Error message\n2026-02-06T16:01:36.123Z [WARN] Warning message",
        );
        assert_eq!(levels(&e), [LogLevel::Debug, LogLevel::Error, LogLevel::Warn]);
        assert_eq!(e.iter().map(|x| x.id).collect::<Vec<_>>(), [0, 1, 2]);
    }
    #[test]
    fn parse_skips_empty_lines() {
        assert_eq!(
            parse("2026-02-06T16:01:34.872Z [DEBUG] First\n\n\r\n2026-02-06T16:01:35.000Z [DEBUG] Second").len(),
            2
        );
    }
    #[test]
    fn parse_skips_invalid_lines() {
        assert_eq!(parse("2026-02-06T16:01:34.872Z [DEBUG] Valid line\nThis line has no timestamp or brackets\n2026-02-06T16:01:35.000Z [ERROR] Another valid line").len(), 2);
    }
    #[test]
    fn parse_message_with_brackets() {
        assert_eq!(
            parse("2026-02-06T16:01:34.872Z [DEBUG] [init] configureGlobalMTLS starting")[0].message,
            "[init] configureGlobalMTLS starting"
        );
    }
    #[test]
    fn parse_timestamp_is_parsed_correctly() {
        let t = parse("2026-02-06T16:01:34.872Z [DEBUG] Test")[0].timestamp;
        assert_eq!(
            (t.year(), t.month(), t.day(), t.hour(), t.minute(), t.second()),
            (2026, 2, 6, 16, 1, 34)
        );
    }
    #[test]
    fn parse_unknown_level_defaults_to_debug() {
        let e = parse("2026-02-06T16:01:34.872Z [INFO] Some info");
        assert_eq!(
            (e.len(), e[0].level, e[0].message.as_str()),
            (1, LogLevel::Debug, "Some info")
        );
    }
    #[test]
    fn load_entries_non_existent_session_returns_empty() {
        assert!(load(Path::new(MISSING), "nonexistent-session").unwrap().is_empty());
    }
    #[test]
    fn load_entries_valid_file_returns_entries() {
        let d = dir_with(&[(
            "test-session-123.txt",
            "2026-02-06T16:01:34.872Z [DEBUG] Loading MCP servers\n2026-02-06T16:01:35.000Z [ERROR] Server failed to start",
        )]);
        assert_eq!(
            levels(&load(d.path(), "test-session-123").unwrap()),
            [LogLevel::Debug, LogLevel::Error]
        );
    }
    #[test]
    fn has_log_no_file_returns_false() {
        assert!(!has_log(Path::new(MISSING), "no-session"));
    }
    #[test]
    fn has_log_file_exists_returns_true() {
        let d = dir_with(&[("my-session.txt", "log content")]);
        assert!(has_log(d.path(), "my-session"));
    }
    #[test]
    fn all_session_ids_non_existent_path_returns_empty() {
        assert!(all_session_ids(Path::new(MISSING)).is_empty());
    }
    #[test]
    fn all_session_ids_returns_filenames() {
        let d = dir_with(&[("sess-1.txt", "log"), ("sess-2.txt", "log"), ("not-a-log.json", "{}")]);
        assert_eq!(all_session_ids(d.path()), ["sess-1", "sess-2"]);
    }
    #[test]
    fn load_entries_paginated_first_page() {
        let d = dir_with(&[("paged-session.txt", &ten(10))]);
        let p = page(d.path(), "paged-session", 0, 3, None).unwrap();
        assert_eq!((p.total, ids(&p)), (10, vec![0, 1, 2]));
    }
    #[test]
    fn load_entries_paginated_second_page() {
        let d = dir_with(&[("paged-session.txt", &ten(10))]);
        let p = page(d.path(), "paged-session", 3, 3, None).unwrap();
        assert_eq!((p.total, ids(&p)), (10, vec![3, 4, 5]));
    }
    #[test]
    fn load_entries_paginated_last_page_partial_results() {
        let d = dir_with(&[("paged-session.txt", &ten(5))]);
        let p = page(d.path(), "paged-session", 3, 10, None).unwrap();
        assert_eq!((p.total, ids(&p)), (5, vec![3, 4]));
    }
    #[test]
    fn load_entries_paginated_offset_beyond_end_returns_empty() {
        let d = dir_with(&[("paged-session.txt", "2026-02-06T16:01:00.000Z [DEBUG] Only one")]);
        let p = page(d.path(), "paged-session", 10, 5, None).unwrap();
        assert_eq!((p.total, p.entries.len()), (1, 0));
    }
    #[test]
    fn load_entries_paginated_non_existent_session() {
        let p = page(Path::new(MISSING), "no-session", 0, 10, None).unwrap();
        assert_eq!((p.total, p.entries.len()), (0, 0));
    }
    #[test]
    fn parse_paged_returns_correct_total_count() {
        let text = "2026-02-06T16:01:00.000Z [DEBUG] A\n2026-02-06T16:01:01.000Z [WARN] B\n2026-02-06T16:01:02.000Z [ERROR] C\n2026-02-06T16:01:03.000Z [DEBUG] D";
        let p = page_from(text.as_bytes(), 0, 2, None);
        assert_eq!(
            (
                p.total,
                p.entries.iter().map(|e| e.message.as_str()).collect::<Vec<_>>()
            ),
            (4, vec!["A", "B"])
        );
    }

    #[test]
    fn a_level_filter_pages_matching_entries_and_keeps_file_ids() {
        let text = "2026-02-06T16:01:00.000Z [DEBUG] A\n2026-02-06T16:01:01.000Z [ERROR] B\n2026-02-06T16:01:02.000Z [DEBUG] C\n2026-02-06T16:01:03.000Z [ERROR] D";
        let p = page_from(text.as_bytes(), 1, 5, Some(LogLevel::Error));
        assert_eq!((p.total, ids(&p)), (2, vec![3]));
    }

    #[test]
    fn session_ids_cannot_leave_the_debug_folder() {
        let d = dir_with(&[]);
        assert!(page(d.path(), "../secrets", 0, 1, None).is_err());
        assert!(!has_log(d.path(), "../secrets"));
    }
}
