//! Skills (ClaudeConfigLoader.loadSkills): `skills/<name>/SKILL.md`, global and project. Claude
//! Code's newer `skills/synced/<id>/<name>/` layout is not read, as in the Swift reference app (parity row E12).

use std::path::Path;

use super::{ProjectScope, config_ref, frontmatter, kind_dir, scope_name, scopes, sort_by_name};
use crate::error::Loaded;
use crate::models::config::scope::ConfigKind;
use crate::models::config::skill::ClaudeSkill;

/// Skills named by their frontmatter `name`, else their folder, sorted by name.
pub fn load(claude_home: &Path, project: Option<ProjectScope>) -> Loaded<Vec<ClaudeSkill>> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (dir, scope, ctx) in scopes(claude_home, project) {
        let Ok(entries) = std::fs::read_dir(kind_dir(&dir, ConfigKind::Skill)) else {
            continue;
        };
        let mut folders: Vec<_> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.join("SKILL.md").is_file())
            .collect();
        folders.sort();
        for folder in folders {
            let Some(folder_name) = folder.file_name().and_then(|n| n.to_str()).map(String::from) else {
                continue;
            };
            let file = folder.join("SKILL.md");
            let Ok(text) = std::fs::read_to_string(&file) else {
                skipped += 1;
                continue;
            };
            let fm = frontmatter::parse(&text);
            let meta = |k: &str| fm.metadata.get(k).cloned();
            items.push(ClaudeSkill {
                id: format!("{}-{folder_name}", scope_name(scope)),
                name: meta("name").unwrap_or_else(|| folder_name.clone()),
                description: meta("description").unwrap_or_default(),
                model: meta("model"),
                allowed_tools: meta("allowed-tools"),
                body: fm.body.clone(),
                file_path: file.display().to_string(),
                scope,
                config_ref: config_ref(ConfigKind::Skill, &ctx, &folder_name),
            });
        }
    }
    sort_by_name(&mut items, |s| &s.name);
    Loaded::new(items, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::{context, home, repo};

    #[test]
    fn loads_global_and_project_skills() {
        let (repo, ctx) = (repo(), context());
        let items = load(
            &home(),
            Some(ProjectScope {
                root: &repo,
                context: &ctx,
            }),
        )
        .value;
        let got: Vec<_> = items
            .iter()
            .map(|s| {
                (
                    s.name.as_str(),
                    s.config_ref.slug.as_str(),
                    s.config_ref.project.is_some(),
                )
            })
            .collect();
        assert_eq!(
            got,
            [("db-migrate", "db-migrate", true), ("pdf-tools", "pdf-tools", false)]
        );
        assert_eq!(items[1].allowed_tools.as_deref(), Some("Read, Bash"));
    }

    #[test]
    fn a_folder_without_skill_md_is_not_a_skill() {
        let d = tempfile::tempdir().unwrap();
        let nested = d.path().join("skills").join("synced").join("x").join("tidy");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(nested.join("SKILL.md"), "---\nname: tidy\n---\n").unwrap();
        assert!(load(d.path(), None).value.is_empty());
    }
}
