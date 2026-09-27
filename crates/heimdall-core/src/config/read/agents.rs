//! Sub-agents (ClaudeConfigLoader.loadCustomAgents and SubAgent.builtIn): the four agents Claude
//! Code ships, then the custom `~/.claude/agents/*.md` files. Only the global folder is read, as
//! in Poirot.

use std::path::Path;

use serde_json::Value;

use super::{config_ref, frontmatter, kind_dir, md_files, sort_by_name};
use crate::error::Loaded;
use crate::models::config::scope::ConfigKind;
use crate::models::config::sub_agent::{AgentColor, AgentMemory, AgentScope, SubAgent};

/// Built-in agents first, then custom ones sorted by name.
pub fn load(claude_home: &Path) -> Loaded<Vec<SubAgent>> {
    let custom = load_custom(claude_home);
    Loaded::new(
        SubAgent::built_in().into_iter().chain(custom.value).collect(),
        custom.skipped,
    )
}

/// Custom agents only. Tools are the comma-separated `tools` list; unknown colours and memory
/// settings are left unset.
pub fn load_custom(claude_home: &Path) -> Loaded<Vec<SubAgent>> {
    let mut items = Vec::new();
    let mut skipped = 0;
    for (stem, path) in md_files(&kind_dir(claude_home, ConfigKind::Agent)) {
        let Ok(text) = std::fs::read_to_string(&path) else {
            skipped += 1;
            continue;
        };
        let fm = frontmatter::parse(&text);
        let meta = |k: &str| fm.metadata.get(k).cloned();
        let tools = meta("tools")
            .map(|t| {
                t.split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        items.push(SubAgent {
            id: format!("custom-{stem}"),
            name: meta("name").unwrap_or_else(|| stem.clone()),
            icon: "person.crop.circle.badge.plus".into(),
            description: meta("description").unwrap_or_default(),
            tools,
            model: meta("model"),
            color: meta("color").and_then(|c| AgentColor::from_raw(&c)),
            prompt: (!fm.body.is_empty()).then(|| fm.body.clone()),
            file_path: Some(path.display().to_string()),
            memory: meta("memory").and_then(|m| serde_json::from_value::<AgentMemory>(Value::String(m)).ok()),
            scope: AgentScope::Global,
            config_ref: Some(config_ref(ConfigKind::Agent, &None, &stem)),
        });
    }
    sort_by_name(&mut items, |a| &a.name);
    Loaded::new(items, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::home;

    #[test]
    fn built_in_agents_are_merged_with_custom_ones() {
        let all = load(&home()).value;
        let ids: Vec<_> = all.iter().map(|a| a.id.as_str()).collect();
        assert_eq!(ids, ["explore", "plan", "bash", "general", "custom-reviewer"]);
        let r = &all[4];
        assert_eq!(
            (r.tools.clone(), r.color, r.model.as_deref()),
            (
                vec!["Read".to_string(), "Grep".into(), "Bash".into()],
                Some(AgentColor::Blue),
                Some("claude-opus-4-6")
            )
        );
        // `memory: project` is not one Poirot knows, so it is left unset.
        assert_eq!(
            (r.memory, r.scope, r.config_ref.as_ref().map(|c| c.slug.as_str())),
            (None, AgentScope::Global, Some("reviewer"))
        );
        assert!(
            r.prompt
                .as_deref()
                .is_some_and(|p| p.starts_with("You are a meticulous"))
        );
        assert!(all[..4].iter().all(|a| a.config_ref.is_none()));
    }
}
