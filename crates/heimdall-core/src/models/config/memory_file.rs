//! Memory files (Models/Config/MemoryFile.swift).

use std::path::PathBuf;

use serde::Serialize;
use specta::Type;

/// Prefix marking a `project_id` as a user-added folder (`custom:<path>`).
pub const CUSTOM_SOURCE_PREFIX: &str = "custom:";

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFile {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub content: String,
    #[serde(skip)]
    pub file: PathBuf,
    pub project_id: String,
    /// Label of a user-added folder; `None` for per-project memory.
    pub source_label: Option<String>,
}

/// One row in a memory source's file list (contract `MemoryFileSummary`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileSummary {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub filename: String,
    pub is_main: bool,
}

/// A summary plus its content, opened on demand (contract `MemoryFileDetail`); fields stay flat,
/// matching the contract's intersection type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileDetail {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub filename: String,
    pub is_main: bool,
    pub content: String,
    /// For showing where the file lives; never sent back (ADR 0010).
    pub display_path: String,
}

impl MemoryFile {
    pub fn is_custom_source(&self) -> bool {
        self.project_id.starts_with(CUSTOM_SOURCE_PREFIX)
    }

    /// `MEMORY.md` in any letter case is the main entry point.
    pub fn is_main(&self) -> bool {
        self.filename.eq_ignore_ascii_case("MEMORY.md")
    }

    /// `debugging.md` → `Debugging`, `my-code_style.md` → `My Code Style`, `MEMORY.md` → `MEMORY`.
    pub fn display_name(filename: &str) -> String {
        let stem = filename.rsplit_once('.').map_or(filename, |(s, _)| s);
        if stem.eq_ignore_ascii_case("MEMORY") {
            return "MEMORY".into();
        }
        stem.split(['-', '_', ' '])
            .filter(|w| !w.is_empty())
            .map(|w| {
                let mut c = w.chars();
                c.next().map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn file(id: &str, filename: &str, content: &str) -> MemoryFile {
        MemoryFile {
            id: id.into(), name: MemoryFile::display_name(filename), filename: filename.into(), content: content.into(),
            file: format!("/tmp/{filename}").into(), project_id: "proj".into(), source_label: None,
        }
    }

    // Ported from PoirotTests/Models/MemoryFileTests.swift (fuzzyMatch cases move to search/matcher.rs, M1-29).
    #[test]
    fn display_name_memory_md_returns_memory() {
        assert_eq!(MemoryFile::display_name("MEMORY.md"), "MEMORY");
    }
    #[test]
    fn display_name_lowercase_memory_md_returns_memory() {
        assert_eq!(MemoryFile::display_name("memory.md"), "MEMORY");
    }
    #[test]
    fn display_name_simple_filename() {
        assert_eq!(MemoryFile::display_name("debugging.md"), "Debugging");
    }
    #[test]
    fn display_name_hyphenated_filename() {
        assert_eq!(MemoryFile::display_name("code-patterns.md"), "Code Patterns");
    }
    #[test]
    fn display_name_underscored_filename() {
        assert_eq!(MemoryFile::display_name("test_patterns.md"), "Test Patterns");
    }
    #[test]
    fn display_name_mixed_separators() {
        assert_eq!(MemoryFile::display_name("my-code_style.md"), "My Code Style");
    }
    #[test]
    fn is_main_true_for_memory_md() {
        assert!(file("proj-MEMORY.md", "MEMORY.md", "# Memory").is_main());
    }
    #[test]
    fn is_main_true_for_lowercase_memory_md() {
        assert!(file("proj-memory.md", "memory.md", "").is_main());
    }
    #[test]
    fn is_main_false_for_other_files() {
        assert!(!file("proj-debugging.md", "debugging.md", "").is_main());
    }
    #[test]
    fn identifiable_uses_id_property() {
        assert_eq!(file("test-id", "test.md", "").id, "test-id");
    }
    #[test]
    fn hashable_equal_files_have_same_hash() {
        let (a, b) = (file("a", "a.md", "x"), file("a", "a.md", "x"));
        let h = |f: &MemoryFile| {
            let mut s = DefaultHasher::new();
            f.hash(&mut s);
            s.finish()
        };
        assert_eq!(a, b);
        assert_eq!(h(&a), h(&b));
    }
    #[test]
    fn hashable_different_files_are_different() {
        assert_ne!(file("a", "a.md", "x"), file("b", "b.md", "y"));
    }

    #[test]
    fn custom_sources_are_recognised_by_prefix() {
        let mut f = file("x", "notes.md", "");
        assert!(!f.is_custom_source());
        f.project_id = "custom:/Users/me/notes".into();
        assert!(f.is_custom_source());
    }
}
