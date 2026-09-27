//! Slash commands (ClaudeConfigLoader.loadCommands): `commands/*.md`, global and project.

use std::path::Path;

use super::{ProjectScope, config_ref, frontmatter, kind_dir, md_files, scope_name, scopes, sort_by_name};
use crate::error::Loaded;
use crate::models::config::command::ClaudeCommand;
use crate::models::config::scope::ConfigKind;

/// Commands named `/<file stem>`, sorted by name. `skipped` counts files that are not UTF-8.
pub fn load(claude_home: &Path, project: Option<ProjectScope>) -> Loaded<Vec<ClaudeCommand>> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (dir, scope, ctx) in scopes(claude_home, project) {
        for (stem, path) in md_files(&kind_dir(&dir, ConfigKind::Command)) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                skipped += 1;
                continue;
            };
            let fm = frontmatter::parse(&text);
            let meta = |k: &str| fm.metadata.get(k).cloned();
            items.push(ClaudeCommand {
                id: format!("{}-{stem}", scope_name(scope)),
                name: format!("/{stem}"),
                description: meta("description").unwrap_or_default(),
                model: meta("model"),
                argument_hint: meta("argument-hint"),
                allowed_tools: meta("allowed-tools"),
                output_style: meta("output-style"),
                body: fm.body.clone(),
                file_path: path.display().to_string(),
                scope,
                config_ref: config_ref(ConfigKind::Command, &ctx, &stem),
            });
        }
    }
    sort_by_name(&mut items, |c| &c.name);
    Loaded::new(items, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::{context, home, repo};
    use crate::models::config::scope::ConfigScope;

    #[test]
    fn loads_global_and_project_commands_with_refs() {
        let (repo, ctx) = (repo(), context());
        let items = load(
            &home(),
            Some(ProjectScope {
                root: &repo,
                context: &ctx,
            }),
        )
        .value;
        let names: Vec<_> = items.iter().map(|c| (c.name.as_str(), c.scope)).collect();
        assert_eq!(
            names,
            [("/deploy", ConfigScope::Project), ("/review", ConfigScope::Global)]
        );
        let review = &items[1];
        assert_eq!(
            (
                review.id.as_str(),
                review.argument_hint.as_deref(),
                review.output_style.as_deref()
            ),
            ("global-review", Some("<file>"), Some("concise"))
        );
        assert_eq!(
            (review.config_ref.slug.as_str(), review.config_ref.project.clone()),
            ("review", None)
        );
        assert_eq!(items[0].config_ref.project, Some(ctx));
        assert!(review.body.starts_with("Review the diff"));
    }

    #[test]
    fn global_only_without_a_project() {
        assert_eq!(load(&home(), None).value.len(), 1);
    }
}
