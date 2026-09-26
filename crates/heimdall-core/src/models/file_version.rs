//! File-history versions (Models/FileVersion.swift).

use chrono::{DateTime, Utc};
use serde::Serialize;
use specta::Type;

/// One backup of a file. `id` is `<sessionId>-<backupFileName>`; the contract's `VersionId` is `backup_file_name`.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileVersion {
    pub id: String,
    pub file_name: String,
    pub session_id: String,
    pub version: u32,
    pub backup_time: DateTime<Utc>,
    pub content_hash: String,
    pub backup_file_name: String,
}

impl PartialEq for FileVersion {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// One file's versions, grouped for the file-history screen (contract `FileHistory`).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileHistory {
    pub files: Vec<FileHistoryFile>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryFile {
    pub file_name: String,
    pub versions: Vec<FileHistoryVersion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryVersion {
    /// The contract `VersionId` (`FileVersion::backup_file_name`).
    pub version_id: String,
    pub version: u32,
    pub backup_time: DateTime<Utc>,
}

/// One version's content, fetched on demand (contract `FileVersionContent`).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileVersionContent {
    pub version_id: String,
    pub content: String,
}

impl FileVersion {
    pub fn new(file_name: impl Into<String>, session_id: impl Into<String>, version: u32, backup_time: DateTime<Utc>,
               content_hash: impl Into<String>, backup_file_name: impl Into<String>) -> Self {
        let (session_id, backup_file_name) = (session_id.into(), backup_file_name.into());
        Self {
            id: format!("{session_id}-{backup_file_name}"),
            file_name: file_name.into(),
            session_id,
            version,
            backup_time,
            content_hash: content_hash.into(),
            backup_file_name,
        }
    }
}

/// All versions of one file in a session, oldest first (by version, then backup time).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct FileHistoryEntry {
    pub file_name: String,
    pub versions: Vec<FileVersion>,
}

impl FileHistoryEntry {
    pub fn new(file_name: impl Into<String>, mut versions: Vec<FileVersion>) -> Self {
        versions.sort_by(|a, b| a.version.cmp(&b.version).then(a.backup_time.cmp(&b.backup_time)));
        Self { file_name: file_name.into(), versions }
    }

    pub fn edit_count(&self) -> usize {
        self.versions.len()
    }

    pub fn latest_version(&self) -> Option<&FileVersion> {
        self.versions.last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(version: u32, secs: i64, backup: &str) -> FileVersion {
        FileVersion::new("a.rs", "s1", version, DateTime::from_timestamp(secs, 0).unwrap(), "h", backup)
    }

    #[test]
    fn versions_sort_by_number_then_time_and_ids_combine_session_and_backup() {
        let e = FileHistoryEntry::new("a.rs", vec![v(2, 5, "h@v2"), v(1, 9, "h@v1b"), v(1, 3, "h@v1a")]);
        assert_eq!(e.versions.iter().map(|x| x.backup_file_name.as_str()).collect::<Vec<_>>(), ["h@v1a", "h@v1b", "h@v2"]);
        assert_eq!((e.edit_count(), e.latest_version().unwrap().id.as_str()), (3, "s1-h@v2"));
    }
}
