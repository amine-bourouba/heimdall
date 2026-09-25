//! Saved plans under `~/.claude/plans/` (Models/Config/Plan.swift).

use std::path::PathBuf;

use serde::Serialize;
use specta::Type;

use super::scope::ConfigRef;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    /// The file stem (slug).
    pub id: String,
    pub name: String,
    pub content: String,
    #[serde(skip)]
    pub file: PathBuf,
    pub config_ref: ConfigRef,
}

impl Plan {
    /// `abstract-wobbling-mccarthy` → `Abstract Wobbling Mccarthy`. Only the first letter of each word
    /// changes case, as in Plan.swift.
    pub fn humanize(slug: &str) -> String {
        slug.split(['-', '_', ' '])
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

    fn plan(id: &str, content: &str, file: &str) -> Plan {
        Plan { id: id.into(), name: Plan::humanize(id), content: content.into(), file: file.into(),
               config_ref: ConfigRef { kind: crate::models::config::scope::ConfigKind::Plan, project: None, slug: id.into() } }
    }
    fn hash(p: &Plan) -> u64 {
        let mut h = DefaultHasher::new();
        p.hash(&mut h);
        h.finish()
    }

    // Ported from PoirotTests/Models/PlanTests.swift. The fuzzyMatch cases move to search/matcher.rs
    // (M1-29) and the delete case to config/write/files.rs (M3-42).
    #[test]
    fn humanize_converts_dash_slugs() {
        assert_eq!(Plan::humanize("abstract-wobbling-mccarthy"), "Abstract Wobbling Mccarthy");
    }
    #[test]
    fn humanize_converts_underscore_slugs() {
        assert_eq!(Plan::humanize("some_plan_name"), "Some Plan Name");
    }
    #[test]
    fn humanize_handles_single_word() {
        assert_eq!(Plan::humanize("plan"), "Plan");
    }
    #[test]
    fn humanize_handles_mixed_separators() {
        assert_eq!(Plan::humanize("my-plan_name"), "My Plan Name");
    }
    #[test]
    fn humanize_handles_empty_string() {
        assert_eq!(Plan::humanize(""), "");
    }
    #[test]
    fn identifiable_uses_id_property() {
        assert_eq!(plan("test-slug", "# Hello", "/tmp/test-slug.md").id, "test-slug");
    }
    #[test]
    fn hashable_equal_plans_have_same_hash() {
        let (a, b) = (plan("a", "x", "/tmp/test.md"), plan("a", "x", "/tmp/test.md"));
        assert_eq!(a, b);
        assert_eq!(hash(&a), hash(&b));
    }
    #[test]
    fn hashable_different_plans_are_different() {
        assert_ne!(plan("a", "x", "/tmp/a.md"), plan("b", "y", "/tmp/b.md"));
    }
}
