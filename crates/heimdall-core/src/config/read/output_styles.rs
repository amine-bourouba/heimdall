//! Output styles (ClaudeConfigLoader.loadOutputStyles): `output-styles/*.md` except `README.md`,
//! global and project.

use std::path::Path;

use super::{ProjectScope, config_ref, frontmatter, kind_dir, md_files, scope_name, scopes, sort_by_name};
use crate::error::Loaded;
use crate::models::config::output_style::OutputStyle;
use crate::models::config::scope::ConfigKind;

/// Styles named by their frontmatter `name`, else their file stem, sorted by name.
pub fn load(claude_home: &Path, project: Option<ProjectScope>) -> Loaded<Vec<OutputStyle>> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (dir, scope, ctx) in scopes(claude_home, project) {
        for (stem, path) in md_files(&kind_dir(&dir, ConfigKind::OutputStyle)) {
            let filename = format!("{stem}.md");
            if filename == "README.md" {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                skipped += 1;
                continue;
            };
            let fm = frontmatter::parse(&text);
            items.push(OutputStyle {
                id: format!("{}-{filename}", scope_name(scope)),
                name: fm.metadata.get("name").cloned().unwrap_or_else(|| stem.clone()),
                description: fm.metadata.get("description").cloned().unwrap_or_default(),
                filename,
                body: fm.body.clone(),
                file_path: path.display().to_string(),
                scope,
                config_ref: config_ref(ConfigKind::OutputStyle, &ctx, &stem),
            });
        }
    }
    sort_by_name(&mut items, |o| &o.name);
    Loaded::new(items, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::{context, home, repo};

    #[test]
    fn loads_global_and_project_styles_without_the_readme() {
        let (repo, ctx) = (repo(), context());
        let items = load(
            &home(),
            Some(ProjectScope {
                root: &repo,
                context: &ctx,
            }),
        )
        .value;
        let files: Vec<_> = items
            .iter()
            .map(|o| (o.filename.as_str(), o.config_ref.project.is_some()))
            .collect();
        assert_eq!(files, [("concise.md", false), ("terse.md", true)]);
        assert_eq!(
            (items[0].name.as_str(), items[0].id.as_str()),
            ("Concise", "global-concise.md")
        );
    }
}
