//! `settings.json` writes (SettingsWriter.swift's own plugin/model/hook functions -- MCP *server*
//! add/update/remove targets a different file, `~/.claude.json`, and is `config/write/mcp.rs`'s
//! own job, M3-41). Every write goes through [`JsonDoc`] (ADR 0009): fails closed on a corrupt
//! file, keeps unknown keys and the file's own formatting, and never falls back to writing an
//! empty document.
//!
//! No Poirot unit tests cover these functions directly: `SettingsWriterMCPTests.swift` (this
//! task's own named Input) tests `MCPServerCatalog`/`MCPEnvKey` instead, already carried over to
//! `models/catalog/mcp_catalog.rs` by M1-10 (`docs/spec/test-carryover.md`'s own line 12 says so
//! explicitly). The tests below are written directly from this task's own acceptance criteria.

use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use super::json_doc::JsonDoc;
use crate::error::{AppError, Result, WriteOutcome};
use crate::models::config::hook::{HookEntry, HookGroupInput, HookHandler};
use crate::models::config::scope::ConfigScope;

fn global_settings_path(claude_home: &Path) -> PathBuf {
    claude_home.join("settings.json")
}

/// Swift's own `writeSettingsDictFor`/`setProjectModel` both call
/// `FileManager.default.createDirectory(at: dir, withIntermediateDirectories: true)` before
/// writing a *project's* `.claude/settings.json`, since a project's `.claude/` folder may not
/// exist yet (a session's own global `~/.claude/` always does by the time anything runs). Missed
/// on a first pass here: `atomic::replace`'s own temp-file creation needs the parent directory to
/// already exist, and failed with a real, reproducible `NotFound` on a fresh project without one.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| AppError::from_io(&e, dir))?;
    }
    Ok(())
}

/// Resolves which `settings.json` a scoped write targets. Kept as two independent parameters
/// (`scope` and `project_root`), not one collapsed `Option`, specifically so the inconsistent
/// combination Poirot's own bug sat on -- `scope == .project` with no real project path --
/// is representable here at all, and therefore reachable to reject. Poirot's
/// `SettingsWriter.settingsURL(scope:projectPath:)` (`SettingsWriter.swift:316-329`) silently
/// returned the *global* settings file in that case; every one of its own callers
/// (`loadSettingsDictFor`/`writeSettingsDictFor`, used by `saveHook`/`deleteHook`/
/// `importHooksFromJSON`) inherited that silent fallback. This task's own AC3 refuses instead.
fn resolve_scoped_path(claude_home: &Path, scope: ConfigScope, project_root: Option<&Path>) -> Result<PathBuf> {
    match (scope, project_root) {
        (ConfigScope::Global, _) => Ok(global_settings_path(claude_home)),
        (ConfigScope::Project, Some(root)) => Ok(root.join(".claude").join("settings.json")),
        (ConfigScope::Project, None) => Err(AppError::invalid_input(
            "a project-scoped write was requested but no project could be resolved; refusing to write the global settings file instead",
        )),
    }
}

/// `true` ops (set/toggle/save) create the file fresh when it is missing, matching Swift's own
/// `writeSettingsDict`, which always writes even starting from `[:]`. `false` ops (remove/delete)
/// have nothing to remove from a file that was never there, so `AppError::NotFound` maps to
/// `WriteOutcome::Unchanged` instead of conjuring a new file just to leave it empty -- a real,
/// deliberate improvement over Swift's own blanket "always write" behaviour, still within this
/// task's own AC2 ("never fall back to an empty document").
fn open_settings(path: &Path, create_if_missing: bool) -> Result<Option<JsonDoc>> {
    match JsonDoc::open(path, create_if_missing) {
        Ok(doc) => Ok(Some(doc)),
        Err(AppError::NotFound { .. }) if !create_if_missing => Ok(None),
        Err(e) => Err(e),
    }
}

fn object_entry<'a>(root: &'a mut Map<String, Value>, key: &str) -> &'a mut Map<String, Value> {
    root.entry(key)
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("just inserted as an object")
}

// MARK: - Plugin toggle / removal

pub fn set_plugin_enabled(claude_home: &Path, key: &str, enabled: bool) -> Result<WriteOutcome> {
    let mut doc = JsonDoc::open(&global_settings_path(claude_home), true)?;
    let plugins = object_entry(doc.root_mut()?, "enabledPlugins");
    plugins.insert(key.to_string(), Value::Bool(enabled));
    doc.save()
}

pub fn remove_plugin(claude_home: &Path, key: &str) -> Result<WriteOutcome> {
    let Some(mut doc) = open_settings(&global_settings_path(claude_home), false)? else {
        return Ok(WriteOutcome::Unchanged);
    };
    let plugins = object_entry(doc.root_mut()?, "enabledPlugins");
    plugins.remove(key);
    doc.save()
}

// MARK: - MCP permission stripping (settings.json's own `permissions.allow`, not `~/.claude.json`'s
// `mcpServers` -- that half of removing an MCP server is `config::write::mcp`'s own job, M3-41;
// the command layer that removes a server end to end calls both).

pub fn remove_mcp_permissions(claude_home: &Path, server_name: &str) -> Result<WriteOutcome> {
    let Some(mut doc) = open_settings(&global_settings_path(claude_home), false)? else {
        return Ok(WriteOutcome::Unchanged);
    };
    let root = doc.root_mut()?;
    let Some(permissions) = root.get_mut("permissions").and_then(Value::as_object_mut) else {
        return Ok(WriteOutcome::Unchanged);
    };
    let Some(allow) = permissions.get_mut("allow").and_then(Value::as_array_mut) else {
        return Ok(WriteOutcome::Unchanged);
    };
    let prefix = format!("mcp__{server_name}__");
    allow.retain(|v| !v.as_str().is_some_and(|s| s.starts_with(&prefix)));
    doc.save()
}

// MARK: - Default / project model. Port-plan C-27: a model *id or alias* (`claude-opus-4-8`,
// `opus`), never a display name (`Opus 4.8`) -- enforced by the caller choosing what string to
// pass in (`models::read`'s own catalog is the one place display names and ids are both known);
// this function writes whatever string it is given verbatim, the same as Swift's own `dict["model"]
// = model` does.

pub fn set_default_model(claude_home: &Path, model: &str) -> Result<WriteOutcome> {
    let mut doc = JsonDoc::open(&global_settings_path(claude_home), true)?;
    doc.root_mut()?
        .insert("model".to_string(), Value::String(model.to_string()));
    doc.save()
}

/// `project_root` is required, not `Option` -- Swift's own `setProjectModel` takes a non-optional
/// `projectPath: String` too, so (unlike the hook functions below) there is no "scope says
/// project but no path given" state to guard against here at all; the caller simply cannot reach
/// this function without a real path.
pub fn set_project_model(project_root: &Path, model: Option<&str>) -> Result<WriteOutcome> {
    let path = project_root.join(".claude").join("settings.json");
    ensure_parent_dir(&path)?;
    let mut doc = JsonDoc::open(&path, true)?;
    let root = doc.root_mut()?;
    match model {
        Some(m) => root.insert("model".to_string(), Value::String(m.to_string())),
        None => root.remove("model"),
    };
    doc.save()
}

// MARK: - Hooks (save / delete / import)

fn hook_groups<'a>(root: &'a mut Map<String, Value>, event: &str) -> &'a mut Vec<Value> {
    object_entry(root, "hooks")
        .entry(event)
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .expect("just inserted as an array")
}

/// The merged handler at `input`'s own index in `old_handlers` (if one exists), reusing
/// [`HookHandler::from_json`]/[`HookHandler::to_json`] (the read side's own round-trip, already
/// tested there) rather than re-deriving the same merge logic: unknown keys the webview's own
/// `HookHandlerInput` never carries (it "only edits the fields it shows", per that type's own doc
/// comment) are read back from whatever was at this position before, so a save can only ever
/// change the fields the editor actually showed -- this task's own AC4.
fn merge_handler(
    input: &crate::models::config::hook::HookHandlerInput,
    old_handlers: Option<&[Value]>,
    index: usize,
) -> Map<String, Value> {
    let extra = old_handlers
        .and_then(|hs| hs.get(index))
        .and_then(Value::as_object)
        .and_then(HookHandler::from_json)
        .map(|h| h.extra)
        .unwrap_or_default();
    HookHandler {
        handler_type: input.handler_type,
        command: input.command.clone(),
        url: input.url.clone(),
        timeout: input.timeout,
        status_message: input.status_message.clone(),
        extra_keys: Vec::new(),
        extra,
    }
    .to_json()
}

fn serialize_group(group: &HookGroupInput, old: Option<&Value>) -> Value {
    let old_handlers = old
        .and_then(|g| g.get("hooks"))
        .and_then(Value::as_array)
        .map(Vec::as_slice);
    let mut obj = Map::new();
    if let Some(matcher) = &group.matcher
        && !matcher.is_empty()
    {
        obj.insert("matcher".to_string(), Value::String(matcher.clone()));
    }
    let handlers: Vec<Value> = group
        .handlers
        .iter()
        .enumerate()
        .map(|(i, h)| Value::Object(merge_handler(h, old_handlers, i)))
        .collect();
    obj.insert("hooks".to_string(), Value::Array(handlers));
    Value::Object(obj)
}

pub fn save_hook(
    claude_home: &Path,
    scope: ConfigScope,
    project_root: Option<&Path>,
    event: &str,
    group: &HookGroupInput,
    existing_index: Option<usize>,
) -> Result<WriteOutcome> {
    let path = resolve_scoped_path(claude_home, scope, project_root)?;
    ensure_parent_dir(&path)?;
    let mut doc = JsonDoc::open(&path, true)?;
    let root = doc.root_mut()?;
    let groups = hook_groups(root, event);
    let old = existing_index.and_then(|i| groups.get(i)).cloned();
    let serialized = serialize_group(group, old.as_ref());
    match existing_index {
        Some(i) if i < groups.len() => groups[i] = serialized,
        _ => groups.push(serialized),
    }
    doc.save()
}

pub fn delete_hook(
    claude_home: &Path,
    scope: ConfigScope,
    project_root: Option<&Path>,
    event: &str,
    index: usize,
) -> Result<WriteOutcome> {
    let path = resolve_scoped_path(claude_home, scope, project_root)?;
    let Some(mut doc) = open_settings(&path, false)? else {
        return Ok(WriteOutcome::Unchanged);
    };
    let root = doc.root_mut()?;
    let Some(hooks) = root.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(WriteOutcome::Unchanged);
    };
    let Some(groups) = hooks.get_mut(event).and_then(Value::as_array_mut) else {
        return Ok(WriteOutcome::Unchanged);
    };
    if index >= groups.len() {
        return Ok(WriteOutcome::Unchanged);
    }
    groups.remove(index);
    if groups.is_empty() {
        hooks.remove(event);
    }
    if hooks.is_empty() {
        root.remove("hooks");
    }
    doc.save()
}

/// `{"hooks": {<event>: [<group>, ...], ...}}`, the same wrapper shape
/// `SettingsWriter.exportHooksAsJSON` writes and `importHooksFromJSON` reads back. Appends every
/// imported group to whatever that event already has; returns the number of groups imported.
/// Rust never opens the file picker itself (heimdall-core has no dependency on Tauri, ADR 0002) --
/// `json_bytes` is already-read file content, the command layer's own job (M3-44) to obtain via a
/// Rust-opened dialog (ADR 0010: "Project-scoped writes accept only... the folder chosen through
/// a Rust-opened dialog").
pub fn import_hooks(
    claude_home: &Path,
    scope: ConfigScope,
    project_root: Option<&Path>,
    json_bytes: &[u8],
) -> Result<usize> {
    let imported: Value =
        serde_json::from_slice(json_bytes).map_err(|e| AppError::parse(format!("not a valid hooks export: {e}")))?;
    let Some(hooks) = imported.get("hooks").and_then(Value::as_object) else {
        return Ok(0);
    };

    let path = resolve_scoped_path(claude_home, scope, project_root)?;
    ensure_parent_dir(&path)?;
    let mut doc = JsonDoc::open(&path, true)?;
    let root = doc.root_mut()?;
    let mut count = 0;
    for (event, groups) in hooks {
        let Some(groups) = groups.as_array() else { continue };
        let existing = hook_groups(root, event);
        existing.extend(groups.iter().cloned());
        count += groups.len();
    }
    doc.save()?;
    Ok(count)
}

/// `{"hooks": {<event>: [<group>, ...]}}`, the same wrapper shape [`import_hooks`] reads back and
/// `SettingsWriter.exportHooksAsJSON` writes. Pure serialization of already-loaded entries
/// (`config::read::hooks::load`, M3-44's own caller) -- no file read, no safe-write primitive,
/// nothing that can fail, so (unlike every write above) this returns the bytes directly rather
/// than a `Result`.
pub fn export_hooks(entries: &[HookEntry]) -> Vec<u8> {
    let mut hooks = Map::new();
    for entry in entries {
        let mut group = Map::new();
        if let Some(matcher) = &entry.matcher {
            group.insert("matcher".to_string(), Value::String(matcher.clone()));
        }
        let handlers: Vec<Value> = entry.handlers.iter().map(|h| Value::Object(h.to_json())).collect();
        group.insert("hooks".to_string(), Value::Array(handlers));
        hooks
            .entry(entry.event.clone())
            .or_insert_with(|| Value::Array(Vec::new()))
            .as_array_mut()
            .expect("just inserted as an array")
            .push(Value::Object(group));
    }
    let mut wrapper = Map::new();
    wrapper.insert("hooks".to_string(), Value::Object(hooks));
    serde_json::to_vec_pretty(&Value::Object(wrapper)).expect("built entirely from safe, already-valid JSON values")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::hook::{HookHandlerInput, HookHandlerType};
    use serde_json::json;

    fn settings_with(dir: &Path, body: &str) -> PathBuf {
        let path = dir.join("settings.json");
        std::fs::write(&path, body).unwrap();
        path
    }

    fn read(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    // MARK: - Plugins

    #[test]
    fn toggling_a_plugin_sets_it_in_an_otherwise_untouched_file() {
        let d = tempfile::tempdir().unwrap();
        settings_with(d.path(), r#"{"model": "opus", "enabledPlugins": {"a@b": true}}"#);
        set_plugin_enabled(d.path(), "c@d", false).unwrap();
        let v = read(&d.path().join("settings.json"));
        assert_eq!(v["model"], "opus");
        assert_eq!(v["enabledPlugins"], json!({"a@b": true, "c@d": false}));
    }

    #[test]
    fn toggling_a_plugin_creates_the_file_when_missing() {
        let d = tempfile::tempdir().unwrap();
        set_plugin_enabled(d.path(), "a@b", true).unwrap();
        assert_eq!(
            read(&d.path().join("settings.json")),
            json!({"enabledPlugins": {"a@b": true}})
        );
    }

    #[test]
    fn removing_a_plugin_drops_only_that_key() {
        let d = tempfile::tempdir().unwrap();
        settings_with(d.path(), r#"{"enabledPlugins": {"a@b": true, "c@d": false}}"#);
        remove_plugin(d.path(), "a@b").unwrap();
        assert_eq!(
            read(&d.path().join("settings.json")),
            json!({"enabledPlugins": {"c@d": false}})
        );
    }

    #[test]
    fn removing_a_plugin_from_a_missing_file_is_a_no_op_that_creates_nothing() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(remove_plugin(d.path(), "a@b").unwrap(), WriteOutcome::Unchanged);
        assert!(!d.path().join("settings.json").exists());
    }

    // MARK: - MCP permission stripping

    #[test]
    fn removing_mcp_permissions_strips_only_that_servers_prefix() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"permissions": {"allow": ["mcp__github__list_prs", "mcp__github__get_pr", "mcp__filesystem__read", "Bash(ls)"], "deny": ["mcp__github__delete_repo"]}}"#,
        );
        remove_mcp_permissions(d.path(), "github").unwrap();
        let v = read(&d.path().join("settings.json"));
        assert_eq!(v["permissions"]["allow"], json!(["mcp__filesystem__read", "Bash(ls)"]));
        // `deny` is untouched -- Swift's own removeMCPPermissions only ever touches `allow`.
        assert_eq!(v["permissions"]["deny"], json!(["mcp__github__delete_repo"]));
    }

    #[test]
    fn removing_mcp_permissions_with_none_set_is_a_no_op() {
        let d = tempfile::tempdir().unwrap();
        settings_with(d.path(), r#"{"model": "opus"}"#);
        assert_eq!(
            remove_mcp_permissions(d.path(), "github").unwrap(),
            WriteOutcome::Unchanged
        );
        assert_eq!(read(&d.path().join("settings.json")), json!({"model": "opus"}));
    }

    // MARK: - Default / project model (port-plan C-27: an id/alias, never a display name)

    #[test]
    fn default_model_is_written_verbatim_as_given() {
        let d = tempfile::tempdir().unwrap();
        set_default_model(d.path(), "claude-opus-4-8").unwrap();
        assert_eq!(read(&d.path().join("settings.json"))["model"], "claude-opus-4-8");
        set_default_model(d.path(), "opus").unwrap();
        assert_eq!(read(&d.path().join("settings.json"))["model"], "opus");
    }

    #[test]
    fn project_model_sets_and_clears_in_the_projects_own_settings_file() {
        let d = tempfile::tempdir().unwrap();
        set_project_model(d.path(), Some("claude-sonnet-4-6")).unwrap();
        let path = d.path().join(".claude").join("settings.json");
        assert_eq!(read(&path)["model"], "claude-sonnet-4-6");
        set_project_model(d.path(), None).unwrap();
        assert!(read(&path).get("model").is_none());
    }

    // MARK: - AC3: a project-scoped write with no resolvable project refuses, not falls back

    #[test]
    fn a_project_scoped_hook_write_with_no_project_returns_invalid_input_and_touches_nothing() {
        let d = tempfile::tempdir().unwrap();
        settings_with(d.path(), r#"{"untouched": true}"#);
        let group = HookGroupInput {
            matcher: None,
            handlers: vec![],
        };
        let err = save_hook(d.path(), ConfigScope::Project, None, "PreToolUse", &group, None).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }), "{err:?}");
        // The *global* file (which a Poirot-style fallback would have silently written to instead)
        // is untouched.
        assert_eq!(read(&d.path().join("settings.json")), json!({"untouched": true}));
    }

    #[test]
    fn a_project_scoped_hook_delete_with_no_project_also_refuses() {
        let d = tempfile::tempdir().unwrap();
        let err = delete_hook(d.path(), ConfigScope::Project, None, "PreToolUse", 0).unwrap_err();
        assert!(matches!(err, AppError::InvalidInput { .. }), "{err:?}");
    }

    // MARK: - AC2: a corrupt target yields Parse and no bytes change

    #[test]
    fn every_operation_on_a_corrupt_file_is_a_parse_error_and_leaves_it_untouched() {
        let d = tempfile::tempdir().unwrap();
        let corrupt = b"{ not json".to_vec();
        let path = settings_with(d.path(), "{ not json");
        let group = HookGroupInput {
            matcher: None,
            handlers: vec![],
        };

        assert!(matches!(
            set_plugin_enabled(d.path(), "a@b", true).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            remove_plugin(d.path(), "a@b").unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            remove_mcp_permissions(d.path(), "gh").unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            set_default_model(d.path(), "opus").unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            save_hook(d.path(), ConfigScope::Global, None, "PreToolUse", &group, None).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            delete_hook(d.path(), ConfigScope::Global, None, "PreToolUse", 0).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            import_hooks(d.path(), ConfigScope::Global, None, br#"{"hooks":{}}"#).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert_eq!(std::fs::read(&path).unwrap(), corrupt);
    }

    // MARK: - AC4: hook handlers keep unknown fields across save

    #[test]
    fn saving_an_existing_group_keeps_unknown_handler_keys() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"hooks": {"PreToolUse": [{"matcher": "Edit", "hooks": [{"type": "command", "command": "old.sh", "async": true, "env": {"A": "1"}}]}]}}"#,
        );
        let group = HookGroupInput {
            matcher: Some("Edit|Write".to_string()),
            handlers: vec![HookHandlerInput {
                handler_type: HookHandlerType::Command,
                command: Some("new.sh".to_string()),
                url: None,
                timeout: Some(30),
                status_message: None,
            }],
        };
        save_hook(d.path(), ConfigScope::Global, None, "PreToolUse", &group, Some(0)).unwrap();
        let v = read(&d.path().join("settings.json"));
        let handler = &v["hooks"]["PreToolUse"][0]["hooks"][0];
        assert_eq!(handler["command"], "new.sh");
        assert_eq!(handler["timeout"], 30);
        // Neither the input nor HookHandlerInput's own type carries these -- they survive only
        // because they were read back from the old handler at the same position.
        assert_eq!(handler["async"], true);
        assert_eq!(handler["env"], json!({"A": "1"}));
        assert_eq!(v["hooks"]["PreToolUse"][0]["matcher"], "Edit|Write");
    }

    #[test]
    fn saving_with_no_existing_index_appends_a_new_group() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "a.sh"}]}]}}"#,
        );
        let group = HookGroupInput {
            matcher: None,
            handlers: vec![HookHandlerInput {
                handler_type: HookHandlerType::Command,
                command: Some("b.sh".to_string()),
                url: None,
                timeout: None,
                status_message: None,
            }],
        };
        save_hook(d.path(), ConfigScope::Global, None, "PreToolUse", &group, None).unwrap();
        let v = read(&d.path().join("settings.json"));
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["hooks"]["PreToolUse"][1]["hooks"][0]["command"], "b.sh");
    }

    // MARK: - Hook delete

    #[test]
    fn deleting_the_last_group_of_an_event_removes_the_event_and_an_empty_hooks_object() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "a.sh"}]}]}, "model": "opus"}"#,
        );
        delete_hook(d.path(), ConfigScope::Global, None, "PreToolUse", 0).unwrap();
        assert_eq!(read(&d.path().join("settings.json")), json!({"model": "opus"}));
    }

    #[test]
    fn deleting_one_of_several_groups_keeps_the_rest_and_the_event() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "a.sh"}]}, {"hooks": [{"type": "command", "command": "b.sh"}]}]}}"#,
        );
        delete_hook(d.path(), ConfigScope::Global, None, "PreToolUse", 0).unwrap();
        let v = read(&d.path().join("settings.json"));
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
        assert_eq!(v["hooks"]["PreToolUse"][0]["hooks"][0]["command"], "b.sh");
    }

    #[test]
    fn deleting_from_a_missing_file_is_a_no_op() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            delete_hook(d.path(), ConfigScope::Global, None, "PreToolUse", 0).unwrap(),
            WriteOutcome::Unchanged
        );
        assert!(!d.path().join("settings.json").exists());
    }

    // MARK: - Hook import

    #[test]
    fn importing_appends_groups_and_returns_the_imported_count() {
        let d = tempfile::tempdir().unwrap();
        settings_with(
            d.path(),
            r#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "existing.sh"}]}]}}"#,
        );
        let export = br#"{"hooks": {"PreToolUse": [{"hooks": [{"type": "command", "command": "new1.sh"}]}], "Stop": [{"hooks": [{"type": "command", "command": "new2.sh"}]}]}}"#;
        let count = import_hooks(d.path(), ConfigScope::Global, None, export).unwrap();
        assert_eq!(count, 2);
        let v = read(&d.path().join("settings.json"));
        assert_eq!(v["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
        assert_eq!(v["hooks"]["Stop"].as_array().unwrap().len(), 1);
    }

    // MARK: - Hook export

    #[test]
    fn export_then_import_round_trips_every_entry() {
        let entries = vec![HookEntry {
            event: "PreToolUse".to_string(),
            matcher_group_index: 0,
            matcher: Some("Edit|Write".to_string()),
            handlers: vec![
                HookHandler::from_json(
                    json!({"type": "command", "command": "a.sh", "timeout": 30})
                        .as_object()
                        .unwrap(),
                )
                .unwrap(),
                HookHandler::from_json(json!({"type": "http", "url": "https://h"}).as_object().unwrap()).unwrap(),
            ],
            scope: ConfigScope::Global,
        }];
        let bytes = export_hooks(&entries);
        let d = tempfile::tempdir().unwrap();
        let count = import_hooks(d.path(), ConfigScope::Global, None, &bytes).unwrap();
        assert_eq!(count, 1);
        let v = read(&d.path().join("settings.json"));
        let group = &v["hooks"]["PreToolUse"][0];
        assert_eq!(group["matcher"], "Edit|Write");
        assert_eq!(group["hooks"][0]["command"], "a.sh");
        assert_eq!(group["hooks"][0]["timeout"], 30);
        assert_eq!(group["hooks"][1]["url"], "https://h");
    }

    #[test]
    fn export_groups_multiple_entries_under_the_same_event() {
        let handlers = |cmd: &str| {
            vec![HookHandler::from_json(json!({"type": "command", "command": cmd}).as_object().unwrap()).unwrap()]
        };
        let entries = vec![
            HookEntry {
                event: "PreToolUse".to_string(),
                matcher_group_index: 0,
                matcher: None,
                handlers: handlers("a.sh"),
                scope: ConfigScope::Global,
            },
            HookEntry {
                event: "PreToolUse".to_string(),
                matcher_group_index: 1,
                matcher: None,
                handlers: handlers("b.sh"),
                scope: ConfigScope::Project,
            },
        ];
        let wrapper: Value = serde_json::from_slice(&export_hooks(&entries)).unwrap();
        assert_eq!(wrapper["hooks"]["PreToolUse"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn exporting_no_entries_is_an_empty_hooks_object() {
        let wrapper: Value = serde_json::from_slice(&export_hooks(&[])).unwrap();
        assert_eq!(wrapper, json!({"hooks": {}}));
    }

    #[test]
    fn importing_malformed_json_is_a_parse_error() {
        let d = tempfile::tempdir().unwrap();
        assert!(matches!(
            import_hooks(d.path(), ConfigScope::Global, None, b"not json").unwrap_err(),
            AppError::Parse { .. }
        ));
    }

    #[test]
    fn importing_a_wrapper_with_no_hooks_key_imports_nothing() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(import_hooks(d.path(), ConfigScope::Global, None, b"{}").unwrap(), 0);
        assert!(!d.path().join("settings.json").exists());
    }

    // MARK: - Project-scoped hook writes land in the project's own file, not the global one

    #[test]
    fn a_project_scoped_hook_save_writes_the_projects_own_settings_file() {
        let claude_home = tempfile::tempdir().unwrap();
        let project = tempfile::tempdir().unwrap();
        settings_with(claude_home.path(), r#"{"should_not_change": true}"#);
        let group = HookGroupInput {
            matcher: None,
            handlers: vec![HookHandlerInput {
                handler_type: HookHandlerType::Command,
                command: Some("p.sh".to_string()),
                url: None,
                timeout: None,
                status_message: None,
            }],
        };
        save_hook(
            claude_home.path(),
            ConfigScope::Project,
            Some(project.path()),
            "PreToolUse",
            &group,
            None,
        )
        .unwrap();
        assert_eq!(
            read(&claude_home.path().join("settings.json")),
            json!({"should_not_change": true})
        );
        let project_settings = read(&project.path().join(".claude").join("settings.json"));
        assert_eq!(
            project_settings["hooks"]["PreToolUse"][0]["hooks"][0]["command"],
            "p.sh"
        );
    }
}
