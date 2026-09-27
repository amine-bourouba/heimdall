//! Memory files (ClaudeConfigLoader.loadMemoryFiles): `projects/<project>/memory/*.md` and the
//! user's own folders. The caller passes the folders (`Prefs.customMemoryFolders`); core never
//! reads preferences itself. `MEMORY.md` sorts first, then names case-insensitively.

use std::path::{Path, PathBuf};

use serde::Serialize;
use specta::Type;

use crate::error::{Loaded, Result};
use crate::models::config::memory_file::{CUSTOM_SOURCE_PREFIX, MemoryFile};
use crate::paths::{last_component, plain_file_name};

/// A place memory files come from (contract `MemorySource`). Ids are `project:<ProjectId>` or
/// `folder:<index into customMemoryFolders>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemorySource {
    pub id: String,
    pub label: String,
    pub kind: MemorySourceKind,
    pub file_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum MemorySourceKind {
    Project,
    Folder,
}

/// A project's memory files.
pub fn load_project(projects_dir: &Path, project_id: &str) -> Result<Loaded<Vec<MemoryFile>>> {
    let dir = projects_dir.join(plain_file_name(project_id)?).join("memory");
    Ok(load_dir(
        &dir,
        project_id,
        |filename| format!("{project_id}-{filename}"),
        None,
    ))
}

/// A user-added folder's memory files (not recursive), labelled with the folder's name.
pub fn load_folder(folder: &Path) -> Loaded<Vec<MemoryFile>> {
    let project_id = format!("{CUSTOM_SOURCE_PREFIX}{}", folder.display());
    let label = last_component(&folder.display().to_string()).to_string();
    load_dir(
        folder,
        &project_id,
        |filename| format!("{project_id}/{filename}"),
        Some(label),
    )
}

/// `.md` files in a folder, without reading them.
pub fn folder_file_count(folder: &Path) -> u32 {
    md_paths(folder).len() as u32
}

/// Every source: projects with at least one memory file (by folder name), then the custom folders
/// in their stored order (missing folders count zero).
pub fn sources(projects_dir: &Path, custom_folders: &[String]) -> Vec<MemorySource> {
    let mut projects: Vec<PathBuf> = std::fs::read_dir(projects_dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    projects.sort();
    let project_sources = projects.into_iter().filter_map(|p| {
        let count = folder_file_count(&p.join("memory"));
        let id = p.file_name()?.to_str()?.to_string();
        (count > 0).then(|| MemorySource {
            id: format!("project:{id}"),
            label: id,
            kind: MemorySourceKind::Project,
            file_count: count,
        })
    });
    let folder_sources = custom_folders.iter().enumerate().map(|(i, f)| MemorySource {
        id: format!("folder:{i}"),
        label: last_component(f).to_string(),
        kind: MemorySourceKind::Folder,
        file_count: folder_file_count(Path::new(f)),
    });
    project_sources.chain(folder_sources).collect()
}

fn md_paths(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .collect();
    v.sort();
    v
}

fn load_dir(
    dir: &Path,
    project_id: &str,
    id_for: impl Fn(&str) -> String,
    source_label: Option<String>,
) -> Loaded<Vec<MemoryFile>> {
    let mut files = Vec::new();
    let mut skipped = 0;
    for path in md_paths(dir) {
        let Some(filename) = path.file_name().and_then(|n| n.to_str()).map(String::from) else {
            continue;
        };
        let Ok(content) = std::fs::read_to_string(&path) else {
            skipped += 1;
            continue;
        };
        files.push(MemoryFile {
            id: id_for(&filename),
            name: MemoryFile::display_name(&filename),
            filename,
            content,
            file: path,
            project_id: project_id.to_string(),
            source_label: source_label.clone(),
        });
    }
    files.sort_by_cached_key(|f| (!f.is_main(), f.name.to_lowercase()));
    Loaded::new(files, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn projects_with_memory(files: &[(&str, &str)]) -> (tempfile::TempDir, String) {
        let d = tempfile::tempdir().unwrap();
        let id = "test-memory-project".to_string();
        let memory = d.path().join(&id).join("memory");
        std::fs::create_dir_all(&memory).unwrap();
        for (name, content) in files {
            std::fs::write(memory.join(name), content).unwrap();
        }
        (d, id)
    }
    fn folder_with(files: &[(&str, &str)]) -> tempfile::TempDir {
        let d = tempfile::tempdir().unwrap();
        for (name, content) in files {
            std::fs::write(d.path().join(name), content).unwrap();
        }
        d
    }

    // Ported from PoirotTests/Services/MemoryLoaderTests.swift.
    #[test]
    fn load_memory_files_returns_empty_for_missing_directory() {
        let d = tempfile::tempdir().unwrap();
        assert!(load_project(d.path(), "nonexistent-project").unwrap().value.is_empty());
    }
    #[test]
    fn load_memory_files_loads_markdown_files() {
        let (d, id) = projects_with_memory(&[
            ("MEMORY.md", "# Main Memory"),
            ("debugging.md", "# Debugging tips"),
            ("notes.txt", "not markdown"),
        ]);
        let f = load_project(d.path(), &id).unwrap().value;
        assert_eq!(
            (f.len(), f[0].is_main(), f[0].name.as_str(), f[0].content.as_str()),
            (2, true, "MEMORY", "# Main Memory")
        );
    }
    #[test]
    fn load_memory_files_memory_md_sorted_first() {
        let (d, id) = projects_with_memory(&[
            ("patterns.md", "# Patterns"),
            ("MEMORY.md", "# Main"),
            ("architecture.md", "# Architecture"),
        ]);
        let f = load_project(d.path(), &id).unwrap().value;
        assert_eq!(
            f.iter().map(|x| x.name.as_str()).collect::<Vec<_>>(),
            ["MEMORY", "Architecture", "Patterns"]
        );
        assert_eq!(f[0].filename, "MEMORY.md");
    }
    #[test]
    fn load_memory_files_sets_project_id() {
        let (d, id) = projects_with_memory(&[("test.md", "# Test")]);
        let f = load_project(d.path(), &id).unwrap().value;
        assert_eq!(
            (f[0].project_id.as_str(), f[0].id.as_str()),
            (id.as_str(), "test-memory-project-test.md")
        );
    }
    #[test]
    fn load_memory_files_custom_folder_loads_markdown_with_source_label() {
        let d = folder_with(&[("notes.md", "# Notes"), ("MEMORY.md", "# Main"), ("todo.txt", "ignore")]);
        let f = load_folder(d.path()).value;
        let label = d.path().file_name().unwrap().to_str().unwrap();
        assert_eq!(f.len(), 2);
        assert!(f[0].is_main() && f.iter().all(MemoryFile::is_custom_source));
        assert_eq!(f[0].source_label.as_deref(), Some(label));
        assert_eq!(f[0].project_id, format!("custom:{}", d.path().display()));
    }
    #[test]
    fn custom_memory_file_count_counts_only_markdown() {
        let d = folder_with(&[("a.md", "a"), ("b.md", "b"), ("c.txt", "c")]);
        assert_eq!(folder_file_count(d.path()), 2);
    }
    #[test]
    fn load_memory_files_custom_folder_missing_returns_empty() {
        assert!(
            load_folder(Path::new("/nonexistent/poirot-missing-5e0a"))
                .value
                .is_empty()
        );
    }

    #[test]
    fn sources_list_projects_with_memory_then_custom_folders() {
        let (d, id) = projects_with_memory(&[("MEMORY.md", "m")]);
        std::fs::create_dir_all(d.path().join("empty-project")).unwrap();
        let folder = folder_with(&[("a.md", "a")]);
        let custom = [folder.path().display().to_string(), "/nonexistent/gone".to_string()];
        let s = sources(d.path(), &custom);
        let got: Vec<_> = s.iter().map(|x| (x.id.as_str(), x.kind, x.file_count)).collect();
        assert_eq!(
            got,
            [
                (format!("project:{id}").as_str(), MemorySourceKind::Project, 1),
                ("folder:0", MemorySourceKind::Folder, 1),
                ("folder:1", MemorySourceKind::Folder, 0)
            ]
        );
        assert!(load_project(d.path(), "../x").is_err());
    }

    #[test]
    fn windows_fixture_project_memory_loads() {
        let projects = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/windows/claude-home/projects");
        let s = sources(&projects, &[]);
        assert!(s.len() >= 3 && s.iter().all(|x| x.file_count > 0));
        let first = &s[0].id["project:".len()..];
        assert!(!load_project(&projects, first).unwrap().value.is_empty());
    }
}
