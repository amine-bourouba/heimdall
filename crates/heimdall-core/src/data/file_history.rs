//! File history (Services/FileHistoryLoader.swift). A session's transcript records which files
//! Claude Code backed up, in `file-history-snapshot` records (all tracked files) and, in newer
//! versions, `file-history-delta` records (one backup each, often the only place a v1 appears).
//! The backups themselves are `~/.claude/file-history/<sessionId>/<hash>@v<n>`.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use chrono::DateTime;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{AppError, Loaded, Result};
use crate::jsonl;
use crate::models::file_version::{FileHistoryEntry, FileVersion};
use crate::paths::{TargetOs, plain_file_name, same_file};
use crate::transcript::blocks::int_value;

/// Only the fields file history needs; everything else in a transcript line is skipped unread.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Record {
    #[serde(rename = "type")]
    kind: Option<String>,
    snapshot: Option<Snapshot>,
    tracking_path: Option<String>,
    backup: Option<Backup>,
}

#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct Snapshot {
    tracked_file_backups: BTreeMap<String, Backup>,
}

#[derive(Deserialize, Default, Clone)]
#[serde(default, rename_all = "camelCase")]
struct Backup {
    backup_file_name: Option<String>,
    version: Option<Value>,
    backup_time: Option<String>,
}

/// The session's files and their versions, sorted by file name. Loads nothing when the session has
/// no transcript under `projects_dir`.
pub fn load(projects_dir: &Path, session_id: &str, os: TargetOs) -> Result<Loaded<Vec<FileHistoryEntry>>> {
    match find_transcript(projects_dir, session_id)? {
        Some(transcript) => load_from_transcript(&transcript, session_id, os),
        None => Ok(Loaded::new(Vec::new(), 0)),
    }
}

/// Whether `~/.claude/file-history/<sessionId>/` holds any backup at all — cheap existence check
/// for a sidebar/list badge, without reading or parsing a single one.
pub fn has_any(file_history_dir: &Path, session_id: &str) -> bool {
    let Ok(id) = plain_file_name(session_id) else {
        return false;
    };
    std::fs::read_dir(file_history_dir.join(id)).is_ok_and(|mut d| d.next().is_some())
}

/// `<project>/<sessionId>.jsonl` in whichever project folder holds it (folders in name order).
pub fn find_transcript(projects_dir: &Path, session_id: &str) -> Result<Option<PathBuf>> {
    let file = format!("{}.jsonl", plain_file_name(session_id)?);
    let Ok(entries) = std::fs::read_dir(projects_dir) else {
        return Ok(None);
    };
    let mut dirs: Vec<PathBuf> = entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    Ok(dirs.into_iter().map(|d| d.join(&file)).find(|p| p.is_file()))
}

/// Versions grouped per file. Snapshot keys are often absolute while delta paths are relative (or
/// the reverse); names that [`same_file`] matches are one file, named by the longest form.
pub fn load_from_transcript(
    transcript: &Path,
    session_id: &str,
    os: TargetOs,
) -> Result<Loaded<Vec<FileHistoryEntry>>> {
    let mut records = jsonl::open::<Record>(transcript)?;
    let mut groups: Vec<(String, Vec<FileVersion>)> = Vec::new();
    let mut add = |file_name: &str, backup: &Backup| {
        let Some(version) = to_version(file_name, backup, session_id) else {
            return;
        };
        match groups.iter_mut().find(|(name, _)| same_file(name, file_name, os)) {
            Some((name, versions)) => {
                if file_name.len() > name.len() {
                    *name = file_name.to_string();
                }
                versions.push(version);
            }
            None => groups.push((file_name.to_string(), vec![version])),
        }
    };
    for record in records.by_ref() {
        match record.kind.as_deref() {
            Some("file-history-snapshot") => {
                for (name, backup) in record.snapshot.map(|s| s.tracked_file_backups).unwrap_or_default() {
                    add(&name, &backup);
                }
            }
            Some("file-history-delta") => {
                if let (Some(name), Some(backup)) = (&record.tracking_path, &record.backup) {
                    add(name, backup);
                }
            }
            _ => {}
        }
    }
    if let Some(e) = records.take_io_error() {
        return Err(AppError::from_io(&e, transcript));
    }

    let mut entries: Vec<FileHistoryEntry> = groups
        .into_iter()
        .map(|(name, versions)| {
            let mut seen = HashSet::new();
            let mut versions: Vec<FileVersion> = FileHistoryEntry::new(name.clone(), versions).versions;
            versions.retain(|v| seen.insert(v.backup_file_name.clone()));
            for v in &mut versions {
                v.file_name.clone_from(&name);
            }
            FileHistoryEntry {
                file_name: name,
                versions,
            }
        })
        .collect();
    entries.sort_by_key(|e| e.file_name.to_lowercase());
    Ok(Loaded::new(entries, records.skipped()))
}

/// A backup's text; `None` when the backup is missing or not UTF-8.
pub fn load_content(file_history_dir: &Path, session_id: &str, backup_file_name: &str) -> Result<Option<String>> {
    let path = file_history_dir
        .join(plain_file_name(session_id)?)
        .join(plain_file_name(backup_file_name)?);
    Ok(std::fs::read(path).ok().and_then(|b| String::from_utf8(b).ok()))
}

/// A usable backup has a file name, a version above zero and a backup time.
fn to_version(file_name: &str, backup: &Backup, session_id: &str) -> Option<FileVersion> {
    let backup_file_name = backup.backup_file_name.as_deref().filter(|n| !n.is_empty())?;
    let version = u32::try_from(int_value(backup.version.as_ref()))
        .ok()
        .filter(|v| *v > 0)?;
    let backup_time = DateTime::parse_from_rfc3339(backup.backup_time.as_deref()?)
        .ok()?
        .to_utc();
    let content_hash = backup_file_name.split('@').next().unwrap_or(backup_file_name);
    Some(FileVersion::new(
        file_name,
        session_id,
        version,
        backup_time,
        content_hash,
        backup_file_name,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Harness {
        _root: tempfile::TempDir,
        projects: PathBuf,
        history: PathBuf,
        session: String,
    }

    fn harness(jsonl: &str) -> Harness {
        let root = tempfile::tempdir().unwrap();
        let (projects, history) = (root.path().join("projects"), root.path().join("file-history"));
        let session = "4333913a-24ab-5adb-859c-b5c0575ef51b".to_string();
        std::fs::create_dir_all(projects.join("proj")).unwrap();
        std::fs::create_dir_all(history.join(&session)).unwrap();
        std::fs::write(projects.join("proj").join(format!("{session}.jsonl")), jsonl).unwrap();
        Harness {
            _root: root,
            projects,
            history,
            session,
        }
    }
    fn load_h(h: &Harness, os: TargetOs) -> Vec<FileHistoryEntry> {
        load(&h.projects, &h.session, os).unwrap().value
    }
    fn names(e: &[FileHistoryEntry]) -> Vec<&str> {
        e.iter().map(|x| x.file_name.as_str()).collect()
    }
    fn versions(e: &FileHistoryEntry) -> Vec<u32> {
        e.versions.iter().map(|v| v.version).collect()
    }

    // Ported from the Swift reference's own test suite (Services/FileHistoryLoaderTests.swift).
    #[test]
    fn load_includes_delta_versions_missing_from_snapshots() {
        let h = harness(concat!(
            r#"{"type":"file-history-snapshot","messageId":"s1","snapshot":{"trackedFileBackups":{"src/a.swift":{"backupFileName":"aaa@v2","version":2,"backupTime":"2026-08-26T12:11:06.199Z"}}}}"#,
            "\n",
            r#"{"type":"file-history-delta","messageId":"d1","trackingPath":"src/a.swift","backup":{"backupFileName":"aaa@v1","version":1,"backupTime":"2026-08-26T12:00:00.000Z"}}"#,
            "\n",
            r#"{"type":"file-history-delta","messageId":"d2","trackingPath":"src/b.swift","backup":{"backupFileName":"bbb@v1","version":1,"backupTime":"2026-08-26T13:00:00Z"}}"#,
            "\n",
            r#"{"type":"file-history-delta","messageId":"d3","trackingPath":"src/c.swift","backup":{"backupFileName":null,"version":1,"backupTime":"2026-08-26T14:00:00.000Z"}}"#,
        ));
        let e = load_h(&h, TargetOs::Macos);
        assert_eq!(names(&e), ["src/a.swift", "src/b.swift"]);
        assert_eq!(versions(&e[0]), [1, 2]);
        assert_eq!(
            e[0].versions
                .iter()
                .map(|v| v.backup_file_name.as_str())
                .collect::<Vec<_>>(),
            ["aaa@v1", "aaa@v2"]
        );
        assert_eq!(versions(&e[1]), [1]);
    }
    #[test]
    fn load_delta_only_session_still_returns_files() {
        let h = harness(concat!(
            r#"{"type":"file-history-snapshot","messageId":"s1","snapshot":{"trackedFileBackups":{}}}"#,
            "\n",
            r#"{"type":"file-history-delta","messageId":"d1","trackingPath":"index.html","backup":{"backupFileName":"abc@v1","version":1,"backupTime":"2026-08-27T18:54:00.854Z"}}"#,
        ));
        let e = load_h(&h, TargetOs::Macos);
        assert_eq!(
            (names(&e), e[0].versions[0].backup_file_name.as_str()),
            (vec!["index.html"], "abc@v1")
        );
    }
    #[test]
    fn same_file_treats_relative_as_suffix_of_absolute() {
        let os = TargetOs::Macos;
        assert!(same_file("src/a.swift", "/Users/me/proj/src/a.swift", os));
        assert!(same_file("/Users/me/proj/src/a.swift", "src/a.swift", os));
        assert!(!same_file("src/a.swift", "lib/a.swift", os));
        assert!(!same_file("src/a.swift", "src/b.swift", os));
    }
    #[test]
    fn load_merges_relative_and_absolute_paths_for_same_file() {
        let h = harness(concat!(
            r#"{"type":"file-history-snapshot","messageId":"s1","snapshot":{"trackedFileBackups":{"/Users/me/proj/src/a.swift":{"backupFileName":"aaa@v2","version":2,"backupTime":"2026-08-26T12:11:06.199Z"}}}}"#,
            "\n",
            r#"{"type":"file-history-delta","messageId":"d1","trackingPath":"src/a.swift","backup":{"backupFileName":"aaa@v1","version":1,"backupTime":"2026-08-26T12:00:00.000Z"}}"#,
        ));
        let e = load_h(&h, TargetOs::Macos);
        assert_eq!(
            (names(&e), versions(&e[0])),
            (vec!["/Users/me/proj/src/a.swift"], vec![1, 2])
        );
    }
    #[test]
    fn load_file_content_reads_backup() {
        let h = harness("");
        std::fs::write(h.history.join(&h.session).join("abc@v1"), "hello v1").unwrap();
        assert_eq!(
            load_content(&h.history, &h.session, "abc@v1").unwrap().as_deref(),
            Some("hello v1")
        );
        assert_eq!(load_content(&h.history, &h.session, "missing").unwrap(), None);
    }

    #[test]
    fn windows_paths_merge_with_relative_tracked_names() {
        let h = harness(concat!(
            r#"{"type":"file-history-snapshot","snapshot":{"trackedFileBackups":{"C:\\Users\\me\\proj\\src\\Main.rs":{"backupFileName":"m@v2","version":2,"backupTime":"2026-08-26T12:11:06Z"}}}}"#,
            "\n",
            r#"{"type":"file-history-delta","trackingPath":"src\\main.rs","backup":{"backupFileName":"m@v1","version":1,"backupTime":"2026-08-26T12:00:00Z"}}"#,
            "\n",
            r#"{"type":"file-history-delta","trackingPath":"src\\main.rs","backup":{"backupFileName":"m@v1","version":1,"backupTime":"2026-08-26T12:00:00Z"}}"#,
        ));
        // NTFS ignores case, so `Main.rs` and `main.rs` are one file on Windows, and the repeated v1 counts once.
        let e = load_h(&h, TargetOs::Windows);
        assert_eq!(
            (names(&e), versions(&e[0])),
            (vec![r"C:\Users\me\proj\src\Main.rs"], vec![1, 2])
        );
        assert!(e[0].versions.iter().all(|v| v.file_name == e[0].file_name));
        // Linux compares case, so they stay apart.
        assert_eq!(load_h(&h, TargetOs::Linux).len(), 2);
    }

    #[test]
    fn ids_cannot_leave_the_history_folder() {
        let h = harness("");
        assert!(load_content(&h.history, "../x", "abc@v1").is_err());
        assert!(load_content(&h.history, &h.session, "..").is_err());
        assert!(find_transcript(&h.projects, "../proj/x").is_err());
    }

    #[test]
    fn windows_fixture_versions_resolve_to_backups_on_disk() {
        let home = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows/claude-home");
        let mut checked = 0;
        for session in [
            "54a79a0b-db3f-46cf-abbe-91ea1056d8bb",
            "ab8bf07e-7a7b-463c-a4f0-cd251b1634ce",
        ] {
            let loaded = load(&home.join("projects"), session, TargetOs::Windows).unwrap();
            assert_eq!(loaded.skipped, 0);
            assert!(!loaded.value.is_empty(), "{session}");
            for v in loaded.value.iter().flat_map(|e| &e.versions) {
                assert!(
                    load_content(&home.join("file-history"), session, &v.backup_file_name)
                        .unwrap()
                        .is_some(),
                    "{}",
                    v.backup_file_name
                );
                checked += 1;
            }
        }
        assert!(checked >= 6);
    }
}
