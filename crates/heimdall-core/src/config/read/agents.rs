//! Sub-agents (ClaudeConfigLoader.loadCustomAgents and SubAgent.builtIn): the four agents Claude
//! Code ships, then the custom `~/.claude/agents/*.md` files. Only the global folder is read, as
//! in the Swift reference app.

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

/// The fields the agent editor models, derived from a parsed front matter. Shared with
/// `write::files::save_agent`, which keeps a field's original text byte for byte whenever the
/// submitted value equals what this returns for the file being edited -- i.e. whenever the user
/// did not change what they were shown. That is what lets a save leave alone a value the editor
/// cannot represent (`color: yellow`, `memory: project`, a `tools:` written as a YAML list, a
/// folded `description: >`) instead of erasing it, and it holds by construction because it is the
/// very derivation `load_custom` uses to build what the user sees.
pub(crate) struct ModelledFields {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub color: Option<AgentColor>,
    pub memory: Option<AgentMemory>,
}

pub(crate) fn modelled_fields(fm: &frontmatter::Frontmatter) -> ModelledFields {
    let meta = |k: &str| fm.metadata.get(k).cloned();
    ModelledFields {
        name: meta("name"),
        description: meta("description"),
        tools: meta("tools")
            .map(|t| {
                t.split(',')
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default(),
        model: meta("model"),
        color: meta("color").and_then(|c| AgentColor::from_raw(&c)),
        memory: meta("memory").and_then(|m| serde_json::from_value::<AgentMemory>(Value::String(m)).ok()),
    }
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
        let fields = modelled_fields(&fm);
        items.push(SubAgent {
            id: format!("custom-{stem}"),
            name: fields.name.unwrap_or_else(|| stem.clone()),
            icon: "person.crop.circle.badge.plus".into(),
            description: fields.description.unwrap_or_default(),
            tools: fields.tools,
            model: fields.model,
            color: fields.color,
            prompt: (!fm.body.is_empty()).then(|| fm.body.clone()),
            file_path: Some(path.display().to_string()),
            memory: fields.memory,
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
        // `memory: project` is not one the Swift reference app knows, so it is left unset.
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
