//! Plans (ClaudeConfigLoader.loadPlans): `~/.claude/plans/*.md`, global only.

use std::path::Path;

use super::{config_ref, kind_dir, md_files, sort_by_name};
use crate::error::Loaded;
use crate::models::config::plan::Plan;
use crate::models::config::scope::ConfigKind;

/// Plans named from their file name (`refactor-auth-module` → `Refactor Auth Module`), sorted.
pub fn load(claude_home: &Path) -> Loaded<Vec<Plan>> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (slug, path) in md_files(&kind_dir(claude_home, ConfigKind::Plan)) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            skipped += 1;
            continue;
        };
        let config_ref = config_ref(ConfigKind::Plan, &None, &slug);
        items.push(Plan { name: Plan::humanize(&slug), content, file: path, config_ref, id: slug });
    }
    sort_by_name(&mut items, |p| &p.name);
    Loaded::new(items, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::home;

    #[test]
    fn plans_are_humanised_from_file_names() {
        let items = load(&home()).value;
        assert_eq!(items.iter().map(|p| (p.id.as_str(), p.name.as_str())).collect::<Vec<_>>(), [("refactor-auth-module", "Refactor Auth Module")]);
        assert!(items[0].config_ref.project.is_none() && !items[0].content.is_empty());
    }
}
