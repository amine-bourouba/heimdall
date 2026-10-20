//! Configuration readers.
//!
//! Items are named by [`ConfigRef`] (kind, scope, slug), never by path, and [`item_path`] is the
//! one place a ref becomes a path again, under the scope's root only (ADR 0010).

use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::models::config::scope::{ConfigKind, ConfigRef, ConfigScope, ProjectContext};
use crate::paths::plain_file_name;

pub mod agents;
pub mod commands;
pub mod frontmatter;
pub mod hooks;
pub mod mcp;
pub mod memory;
pub mod models;
pub mod output_styles;
pub mod plans;
pub mod plugins;
pub mod settings;
pub mod skills;

/// A project whose `<root>/.claude/` items are listed next to the global ones.
#[derive(Debug, Clone, Copy)]
pub struct ProjectScope<'a> {
    /// The project folder itself (not its `.claude/`).
    pub root: &'a Path,
    pub context: &'a ProjectContext,
}

/// The folder holding items of `kind` under a Claude folder (`~/.claude` or `<project>/.claude`).
pub fn kind_dir(claude_dir: &Path, kind: ConfigKind) -> PathBuf {
    claude_dir.join(match kind {
        ConfigKind::Command => "commands",
        ConfigKind::Skill => "skills",
        ConfigKind::Plan => "plans",
        ConfigKind::OutputStyle => "output-styles",
        ConfigKind::Agent => "agents",
    })
}

/// The file behind an item: `<dir>/<slug>.md`, or `<dir>/<slug>/SKILL.md` for skills. The slug must
/// be a plain file name, so the path cannot leave the kind's folder.
pub fn item_path(claude_dir: &Path, kind: ConfigKind, slug: &str) -> Result<PathBuf> {
    let dir = kind_dir(claude_dir, kind);
    let slug = plain_file_name(slug)?;
    Ok(match kind {
        ConfigKind::Skill => dir.join(slug).join("SKILL.md"),
        _ => dir.join(format!("{slug}.md")),
    })
}

/// `(stem, path)` of each `.md` file in `dir`, by name; a missing folder has none.
pub(crate) fn md_files(dir: &Path) -> Vec<(String, PathBuf)> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut files: Vec<(String, PathBuf)> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x == "md"))
        .filter_map(|p| Some((p.file_stem()?.to_str()?.to_string(), p)))
        .collect();
    files.sort();
    files
}

/// Where a loader looks: the global folder, then the project's when there is one.
pub(crate) fn scopes<'a>(
    claude_home: &'a Path,
    project: Option<ProjectScope<'a>>,
) -> Vec<(PathBuf, ConfigScope, Option<ProjectContext>)> {
    let mut v = vec![(claude_home.to_path_buf(), ConfigScope::Global, None)];
    if let Some(p) = project {
        v.push((p.root.join(".claude"), ConfigScope::Project, Some(p.context.clone())));
    }
    v
}

/// `global` or `project`, the prefix of the Swift reference app's item ids.
pub(crate) fn scope_name(scope: ConfigScope) -> &'static str {
    match scope {
        ConfigScope::Global => "global",
        ConfigScope::Project => "project",
    }
}

pub(crate) fn config_ref(kind: ConfigKind, project: &Option<ProjectContext>, slug: &str) -> ConfigRef {
    ConfigRef {
        kind,
        project: project.clone(),
        slug: slug.to_string(),
    }
}

/// Case-insensitive name order, as the Swift reference app's `localizedCaseInsensitiveCompare`.
pub(crate) fn sort_by_name<T>(items: &mut [T], name: impl Fn(&T) -> &str) {
    items.sort_by_cached_key(|t| name(t).to_lowercase());
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub fn synthetic() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/synthetic")
    }
    pub fn home() -> PathBuf {
        synthetic().join("claude-home")
    }
    pub fn repo() -> PathBuf {
        synthetic().join("project-repo")
    }
    pub fn context() -> ProjectContext {
        ProjectContext::Project {
            project_id: "-Users-fixture-src-app".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refs_resolve_only_inside_their_folder() {
        let home = Path::new("/h/.claude");
        assert_eq!(
            item_path(home, ConfigKind::Skill, "pdf-tools").unwrap(),
            home.join("skills").join("pdf-tools").join("SKILL.md")
        );
        assert_eq!(
            item_path(home, ConfigKind::OutputStyle, "concise").unwrap(),
            home.join("output-styles").join("concise.md")
        );
        for bad in ["../settings", "a/b", "..", ""] {
            assert!(item_path(home, ConfigKind::Command, bad).is_err(), "{bad}");
        }
    }
}
