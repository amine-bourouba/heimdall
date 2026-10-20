//! Agent, command, skill and output-style file writes (`ClaudeConfigLoader.swift:650-862`): agent
//! save (a rename deletes the old file), template scaffolding with unique names, generic
//! config-item delete by [`ConfigRef`], and agent import/export as JSON. Global scope only, the
//! same scope the Swift reference app's own functions use (`agentsDir`, `claudeDir.appendingPathComponent(...)`,
//! no project parameter anywhere in the Swift source) -- a caller resolving a project-scoped
//! `claude_dir` instead is free to, these functions never assume `~/.claude` specifically.
//!
//! This task's own AC1 (paths canonicalised, must stay inside the resolved root, symlink escapes
//! refused): `item_path` (`config::read`) already rejects a slug that could itself traverse out of
//! its folder (`paths::plain_file_name`), but says nothing about a *symlink* already sitting at an
//! otherwise-legitimate-looking path inside that folder and pointing elsewhere -- `ensure_within_root`
//! below canonicalises (resolving every symlink) and checks containment before any delete or
//! rename-induced old-file removal touches a path that already exists on disk. New template files
//! never need this check at all: their own uniqueness loop already skips straight past any name
//! that already exists (symlink or not), so a planted symlink is simply never written through.

use std::path::Path;

use crate::error::{AppError, Result, WriteOutcome};
use crate::models::config::scope::{ConfigKind, ConfigRef};
use crate::models::config::sub_agent::AgentInput;
use crate::paths::ensure_within_root;

use super::super::read::{config_ref, item_path, kind_dir};

fn ensure_directory(dir: &Path) -> Result<()> {
    std::fs::create_dir_all(dir).map_err(|e| AppError::from_io(&e, dir))
}

fn agent_slug(name: &str) -> String {
    name.to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}

/// `<base>.md` (or, for skills, `<base>/SKILL.md`), else `<base>-2`, `<base>-3`, ... -- the first
/// name `item_path` resolves to something that does not already exist.
fn unique_slug(claude_dir: &Path, kind: ConfigKind, base: &str) -> Result<String> {
    let mut candidate = base.to_string();
    let mut counter = 2;
    loop {
        if !item_path(claude_dir, kind, &candidate)?.exists() {
            return Ok(candidate);
        }
        candidate = format!("{base}-{counter}");
        counter += 1;
    }
}

fn agent_frontmatter(input: &AgentInput) -> String {
    let mut fm = String::from("---\n");
    fm += &format!("name: {}\n", input.name);
    fm += &format!("description: {}\n", input.description);
    if !input.tools.is_empty() {
        fm += &format!("tools: {}\n", input.tools.join(", "));
    }
    if let Some(model) = &input.model
        && !model.is_empty()
    {
        fm += &format!("model: {model}\n");
    }
    if let Some(color) = input.color {
        // AgentColor's own Serialize (camelCase of a single-word variant) is exactly the
        // lowercase raw value the frontmatter and `AgentColor::from_raw` already agree on --
        // reused rather than a second hand-written lowercasing.
        fm += &format!("color: {}\n", serde_json::to_value(color).unwrap().as_str().unwrap());
    }
    if let Some(memory) = input.memory {
        fm += &format!("memory: {}\n", serde_json::to_value(memory).unwrap().as_str().unwrap());
    }
    fm += "---\n";
    format!("{fm}\n{}", input.prompt.as_deref().unwrap_or(""))
}

/// Renaming (a new slug derived from `input.name` differs from `original_slug`) writes the new
/// file first, then deletes the old one -- matching Swift's own order (`saveAgent`'s own comment:
/// "Delete old file if renamed") so a failed write never loses the old content.
pub fn save_agent(claude_dir: &Path, input: &AgentInput, original_slug: Option<&str>) -> Result<WriteOutcome> {
    let dir = kind_dir(claude_dir, ConfigKind::Agent);
    ensure_directory(&dir)?;

    let desired_slug = agent_slug(&input.name);
    let target_slug = match original_slug {
        Some(slug) if slug == desired_slug => slug.to_string(),
        _ => unique_slug(claude_dir, ConfigKind::Agent, &desired_slug)?,
    };
    let path = item_path(claude_dir, ConfigKind::Agent, &target_slug)?;
    std::fs::write(&path, agent_frontmatter(input)).map_err(|e| AppError::from_io(&e, &path))?;

    if let Some(original_slug) = original_slug
        && original_slug != target_slug
    {
        let old_path = item_path(claude_dir, ConfigKind::Agent, original_slug)?;
        if old_path.exists() {
            let safe = ensure_within_root(&old_path, &dir)?;
            std::fs::remove_file(&safe).map_err(|e| AppError::from_io(&e, &safe))?;
        }
    }
    Ok(WriteOutcome::Written)
}

const AGENT_TEMPLATE: &str = "---\nname: New Agent\ndescription: Describe what this agent does\ntools: Glob, Grep, Read, Bash\nmodel: sonnet\ncolor: orange\n---\n\nYour agent instructions go here.";
const COMMAND_TEMPLATE: &str = "---\ndescription: Describe what this command does\n# model: claude-sonnet-4-6\n# allowed-tools: Bash, Read, Write\n# argument-hint: <arg>\n---\n\nYour command instructions go here.";
const SKILL_TEMPLATE: &str = "---\nname: New Skill\ndescription: Describe what this skill does\n# model: claude-sonnet-4-6\n# allowed-tools: Bash, Read, Write, Edit\n---\n\nYour skill instructions go here.";
// Swift's own base name is "new-style", not "new-output-style" -- kept exactly, not "fixed" into
// a more consistent-looking name that would silently disagree with the reference behaviour.
const OUTPUT_STYLE_TEMPLATE: &str = "---\nname: New Style\ndescription: Describe this output style\n---\n\nDefine your output formatting instructions here.";

/// `docs/ipc-contract.md`'s own `create_config_template(kind, project)`. Plans have no template
/// in the Swift reference app at all (no `createPlanTemplate` exists in the Swift source) and none is invented
/// here -- this task's own Goal names exactly "agent, command, skill and output-style".
pub fn create_template(claude_dir: &Path, kind: ConfigKind) -> Result<ConfigRef> {
    let (base, template) = match kind {
        ConfigKind::Agent => ("new-agent", AGENT_TEMPLATE),
        ConfigKind::Command => ("new-command", COMMAND_TEMPLATE),
        ConfigKind::Skill => ("new-skill", SKILL_TEMPLATE),
        ConfigKind::OutputStyle => ("new-style", OUTPUT_STYLE_TEMPLATE),
        ConfigKind::Plan => return Err(AppError::invalid_input("plans have no template to scaffold")),
    };
    let slug = unique_slug(claude_dir, kind, base)?;
    let path = item_path(claude_dir, kind, &slug)?;
    ensure_directory(path.parent().expect("item_path always has a parent"))?;
    std::fs::write(&path, template).map_err(|e| AppError::from_io(&e, &path))?;
    Ok(config_ref(kind, &None, &slug))
}

/// Deleting a skill removes its whole folder (the one its `SKILL.md` lives in), matching Swift's
/// own `deleteConfigFile` special case; every other kind removes just its one file.
pub fn delete_item(claude_dir: &Path, config_ref: &ConfigRef) -> Result<WriteOutcome> {
    let path = item_path(claude_dir, config_ref.kind, &config_ref.slug)?;
    if !path.exists() {
        return Ok(WriteOutcome::Unchanged);
    }
    let root = kind_dir(claude_dir, config_ref.kind);
    let safe = ensure_within_root(&path, &root)?;
    let target = if config_ref.kind == ConfigKind::Skill {
        safe.parent()
            .expect("a skill's item_path is always <dir>/<slug>/SKILL.md")
            .to_path_buf()
    } else {
        safe
    };
    if target.is_dir() {
        std::fs::remove_dir_all(&target).map_err(|e| AppError::from_io(&e, &target))?;
    } else {
        std::fs::remove_file(&target).map_err(|e| AppError::from_io(&e, &target))?;
    }
    Ok(WriteOutcome::Written)
}

/// `{"name", "description", "tools", "model", "color", "prompt", "memory"}`, every optional field
/// as an empty string when absent -- matching `SettingsWriter.exportAgentAsJSON`'s own exact shape
/// byte for byte, since this is an interop format (export, then import, possibly by a different
/// the Swift reference app or Heimdall user) rather than an internal IPC contract type free to use `null`.
pub fn export_agent_json(claude_dir: &Path, slug: &str) -> Result<Vec<u8>> {
    let path = item_path(claude_dir, ConfigKind::Agent, slug)?;
    let text = std::fs::read_to_string(&path).map_err(|e| AppError::from_io(&e, &path))?;
    let fm = super::super::read::frontmatter::parse(&text);
    let meta = |k: &str| fm.metadata.get(k).cloned().unwrap_or_default();
    let tools: Vec<String> = meta("tools")
        .split(',')
        .map(str::trim)
        .filter(|t| !t.is_empty())
        .map(String::from)
        .collect();
    let obj = serde_json::json!({
        "name": meta("name"),
        "description": meta("description"),
        "tools": tools,
        "model": meta("model"),
        "color": meta("color"),
        "prompt": fm.body,
        "memory": meta("memory"),
    });
    serde_json::to_vec_pretty(&obj).map_err(|e| AppError::parse(e.to_string()))
}

/// The counterpart to [`export_agent_json`]: always creates a brand-new agent (`original_slug:
/// None`), matching `importAgentFromJSON`'s own behaviour of building a fresh `SubAgent` with a
/// new id rather than ever updating an existing one.
pub fn import_agent_json(claude_dir: &Path, json_bytes: &[u8]) -> Result<WriteOutcome> {
    let value: serde_json::Value =
        serde_json::from_slice(json_bytes).map_err(|e| AppError::parse(format!("not a valid agent export: {e}")))?;
    let name = value
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::parse("missing \"name\""))?;
    let str_field = |k: &str| {
        value
            .get(k)
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let tools: Vec<String> = value
        .get("tools")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str())
        .map(String::from)
        .collect();
    let input = AgentInput {
        name: name.to_string(),
        icon: "person.crop.circle.badge.plus".to_string(),
        description: str_field("description").unwrap_or_default(),
        tools,
        model: str_field("model"),
        color: str_field("color").and_then(|c| crate::models::config::sub_agent::AgentColor::from_raw(&c)),
        prompt: str_field("prompt"),
        memory: str_field("memory").and_then(|m| serde_json::from_value(serde_json::Value::String(m)).ok()),
    };
    save_agent(claude_dir, &input, None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::sub_agent::{AgentColor, AgentMemory};

    fn input(name: &str) -> AgentInput {
        AgentInput {
            name: name.to_string(),
            icon: "person.crop.circle.badge.plus".to_string(),
            description: "A test agent".to_string(),
            tools: vec!["Read".to_string(), "Grep".to_string()],
            model: Some("sonnet".to_string()),
            color: Some(AgentColor::Blue),
            prompt: Some("Be helpful.".to_string()),
            memory: Some(AgentMemory::User),
        }
    }

    // MARK: - save_agent

    #[test]
    fn saving_a_new_agent_writes_frontmatter_and_prompt() {
        let d = tempfile::tempdir().unwrap();
        save_agent(d.path(), &input("Code Reviewer"), None).unwrap();
        let path = d.path().join("agents").join("code-reviewer.md");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("name: Code Reviewer"));
        assert!(text.contains("description: A test agent"));
        assert!(text.contains("tools: Read, Grep"));
        assert!(text.contains("model: sonnet"));
        assert!(text.contains("color: blue"));
        assert!(text.contains("memory: user"));
        assert!(text.ends_with("Be helpful."));
    }

    #[test]
    fn saving_the_same_name_again_overwrites_in_place_not_as_a_new_unique_file() {
        let d = tempfile::tempdir().unwrap();
        save_agent(d.path(), &input("Reviewer"), None).unwrap();
        let mut second = input("Reviewer");
        second.description = "Updated description".to_string();
        save_agent(d.path(), &second, Some("reviewer")).unwrap();
        let siblings: Vec<_> = std::fs::read_dir(d.path().join("agents")).unwrap().collect();
        assert_eq!(
            siblings.len(),
            1,
            "a second save with the same resulting slug must not create reviewer-2.md"
        );
        assert!(
            std::fs::read_to_string(d.path().join("agents").join("reviewer.md"))
                .unwrap()
                .contains("Updated description")
        );
    }

    #[test]
    fn renaming_an_agent_writes_the_new_file_and_deletes_the_old_one() {
        let d = tempfile::tempdir().unwrap();
        save_agent(d.path(), &input("Old Name"), None).unwrap();
        save_agent(d.path(), &input("New Name"), Some("old-name")).unwrap();
        assert!(!d.path().join("agents").join("old-name.md").exists());
        assert!(d.path().join("agents").join("new-name.md").exists());
    }

    #[test]
    fn renaming_to_a_name_that_collides_gets_a_unique_slug_and_still_deletes_the_old_file() {
        let d = tempfile::tempdir().unwrap();
        save_agent(d.path(), &input("Keep This One"), None).unwrap(); // keep-this-one.md
        save_agent(d.path(), &input("Will Be Renamed"), None).unwrap(); // will-be-renamed.md
        save_agent(d.path(), &input("Keep This One"), Some("will-be-renamed")).unwrap();
        assert!(!d.path().join("agents").join("will-be-renamed.md").exists());
        assert!(d.path().join("agents").join("keep-this-one.md").exists());
        assert!(d.path().join("agents").join("keep-this-one-2.md").exists());
    }

    // MARK: - create_template, AC2

    #[test]
    fn template_names_follow_new_agent_new_agent_2() {
        let d = tempfile::tempdir().unwrap();
        let first = create_template(d.path(), ConfigKind::Agent).unwrap();
        let second = create_template(d.path(), ConfigKind::Agent).unwrap();
        let third = create_template(d.path(), ConfigKind::Agent).unwrap();
        assert_eq!(
            [first.slug, second.slug, third.slug],
            ["new-agent", "new-agent-2", "new-agent-3"]
        );
    }

    #[test]
    fn every_named_kind_gets_its_own_real_template_content() {
        let d = tempfile::tempdir().unwrap();
        for (kind, needle) in [
            (ConfigKind::Agent, "Your agent instructions go here."),
            (ConfigKind::Command, "Your command instructions go here."),
            (ConfigKind::Skill, "Your skill instructions go here."),
            (
                ConfigKind::OutputStyle,
                "Define your output formatting instructions here.",
            ),
        ] {
            let config_ref = create_template(d.path(), kind).unwrap();
            let path = item_path(d.path(), kind, &config_ref.slug).unwrap();
            assert!(std::fs::read_to_string(&path).unwrap().contains(needle), "{kind:?}");
        }
    }

    #[test]
    fn a_skill_template_is_a_real_folder_with_a_unique_name() {
        let d = tempfile::tempdir().unwrap();
        create_template(d.path(), ConfigKind::Skill).unwrap();
        create_template(d.path(), ConfigKind::Skill).unwrap();
        assert!(d.path().join("skills").join("new-skill").join("SKILL.md").exists());
        assert!(d.path().join("skills").join("new-skill-2").join("SKILL.md").exists());
    }

    #[test]
    fn plans_have_no_template() {
        let d = tempfile::tempdir().unwrap();
        assert!(create_template(d.path(), ConfigKind::Plan).is_err());
    }

    // MARK: - delete_item

    #[test]
    fn deleting_a_command_removes_its_one_file() {
        let d = tempfile::tempdir().unwrap();
        let config_ref = create_template(d.path(), ConfigKind::Command).unwrap();
        assert_eq!(delete_item(d.path(), &config_ref).unwrap(), WriteOutcome::Written);
        assert!(
            !item_path(d.path(), ConfigKind::Command, &config_ref.slug)
                .unwrap()
                .exists()
        );
    }

    #[test]
    fn deleting_a_skill_removes_its_whole_folder() {
        let d = tempfile::tempdir().unwrap();
        let config_ref = create_template(d.path(), ConfigKind::Skill).unwrap();
        let folder = d.path().join("skills").join(&config_ref.slug);
        assert!(folder.exists());
        assert_eq!(delete_item(d.path(), &config_ref).unwrap(), WriteOutcome::Written);
        assert!(!folder.exists());
    }

    #[test]
    fn deleting_something_that_does_not_exist_is_a_no_op() {
        let d = tempfile::tempdir().unwrap();
        let config_ref = config_ref(ConfigKind::Command, &None, "nonexistent");
        assert_eq!(delete_item(d.path(), &config_ref).unwrap(), WriteOutcome::Unchanged);
    }

    // MARK: - AC1: canonicalised, must stay inside the resolved root; symlink escapes refused

    #[cfg(unix)]
    #[test]
    fn deleting_through_a_symlink_that_escapes_the_kind_folder_is_refused() {
        let d = tempfile::tempdir().unwrap();
        let outside = d.path().join("outside.txt");
        std::fs::write(&outside, "do not delete me").unwrap();
        let commands_dir = d.path().join("commands");
        std::fs::create_dir_all(&commands_dir).unwrap();
        std::os::unix::fs::symlink(&outside, commands_dir.join("evil.md")).unwrap();

        let config_ref = config_ref(ConfigKind::Command, &None, "evil");
        let err = delete_item(d.path(), &config_ref).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }), "{err:?}");
        assert!(outside.exists(), "the real target outside the root must survive");
        assert!(
            commands_dir.join("evil.md").exists(),
            "the symlink itself is left alone, not silently removed"
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_old_agent_file_is_not_followed_when_a_rename_would_delete_it() {
        let d = tempfile::tempdir().unwrap();
        let outside = d.path().join("outside.txt");
        std::fs::write(&outside, "do not delete me").unwrap();
        let agents_dir = d.path().join("agents");
        std::fs::create_dir_all(&agents_dir).unwrap();
        std::os::unix::fs::symlink(&outside, agents_dir.join("old-name.md")).unwrap();

        let err = save_agent(d.path(), &input("New Name"), Some("old-name")).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }), "{err:?}");
        assert!(outside.exists());
    }

    // MARK: - Export / import

    #[test]
    fn export_then_import_round_trips_the_agents_own_fields() {
        let d = tempfile::tempdir().unwrap();
        save_agent(d.path(), &input("Round Trip"), None).unwrap();
        let json = export_agent_json(d.path(), "round-trip").unwrap();
        let v: serde_json::Value = serde_json::from_slice(&json).unwrap();
        assert_eq!(v["name"], "Round Trip");
        assert_eq!(v["tools"], serde_json::json!(["Read", "Grep"]));
        assert_eq!(v["color"], "blue");
        assert_eq!(v["prompt"], "Be helpful.");

        import_agent_json(d.path(), &json).unwrap();
        // Import always creates anew (matching Swift) -- the slug comes from the exported name,
        // same as any other new save, and collides with the still-present original.
        assert!(d.path().join("agents").join("round-trip.md").exists());
        assert!(d.path().join("agents").join("round-trip-2.md").exists());
    }

    #[test]
    fn importing_malformed_json_is_a_parse_error() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(
            import_agent_json(d.path(), b"not json").unwrap_err(),
            AppError::Parse { .. }
        ));
    }

    #[test]
    fn importing_without_a_name_is_a_parse_error() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(
            import_agent_json(d.path(), b"{}").unwrap_err(),
            AppError::Parse { .. }
        ));
    }
}
