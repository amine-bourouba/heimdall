//! Hooks reader (ClaudeConfigLoader.loadHooks): the `hooks` object of the global and project
//! `settings.json`, one entry per matcher group. Handlers keep keys Heimdall does not model
//! (`HookHandler::extra`) so a later save does not drop them.

use std::path::Path;

use serde_json::Value;

use crate::error::Loaded;
use crate::models::config::hook::{HookEntry, HookHandler};
use crate::models::config::scope::ConfigScope;

/// Global entries, then the project's, sorted by event name (stable, so file order holds within
/// an event). Events are kept as text, so events newer than Heimdall still show; handlers of an
/// unknown type are dropped, and a group left with none is skipped, as in Poirot. `skipped` counts
/// settings files that exist but do not parse.
pub fn load(claude_home: &Path, project_root: Option<&Path>) -> Loaded<Vec<HookEntry>> {
    let mut files = vec![(claude_home.join("settings.json"), ConfigScope::Global)];
    if let Some(root) = project_root {
        files.push((root.join(".claude").join("settings.json"), ConfigScope::Project));
    }
    let mut entries = Vec::new();
    let mut skipped = 0;
    for (file, scope) in files {
        let Ok(bytes) = std::fs::read(&file) else { continue };
        let Ok(doc) = serde_json::from_slice::<Value>(&bytes) else {
            skipped += 1;
            continue;
        };
        let Some(hooks) = doc.get("hooks").and_then(Value::as_object) else { continue };
        for (event, groups) in hooks {
            let Some(groups) = groups.as_array() else { continue };
            for (index, group) in groups.iter().enumerate() {
                let handlers: Vec<HookHandler> = group
                    .get("hooks")
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|h| h.as_object().and_then(HookHandler::from_json))
                    .collect();
                if handlers.is_empty() {
                    continue;
                }
                entries.push(HookEntry {
                    event: event.clone(),
                    matcher_group_index: index as u32,
                    matcher: group.get("matcher").and_then(Value::as_str).map(String::from),
                    handlers,
                    scope,
                });
            }
        }
    }
    entries.sort_by(|a, b| a.event.cmp(&b.event));
    Loaded::new(entries, skipped)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::{home, repo};
    use crate::models::config::hook::HookHandlerType;

    #[test]
    fn global_and_project_hooks_with_their_groups_and_handlers() {
        let repo = repo();
        let entries = load(&home(), Some(&repo)).value;
        let summary: Vec<_> = entries.iter().map(|e| (e.event.as_str(), e.scope, e.matcher.as_deref(), e.handlers.len())).collect();
        assert_eq!(summary, [
            ("PostToolUse", ConfigScope::Project, Some("Bash"), 1),
            ("PreToolUse", ConfigScope::Global, Some("Edit|Write"), 2),
            ("SessionStart", ConfigScope::Global, None, 1),
        ]);
        let pre = &entries[1];
        assert_eq!(pre.handlers.iter().map(|h| h.handler_type).collect::<Vec<_>>(), [HookHandlerType::Command, HookHandlerType::Http]);
        assert_eq!((pre.handlers[1].url.as_deref(), pre.handlers[1].status_message.as_deref(), pre.handlers[1].timeout), (Some("https://hooks.example.com/pre-tool-use"), Some("Notifying the audit service"), Some(10)));
        assert_eq!(load(&home(), None).value.len(), 2);
    }

    #[test]
    fn unknown_handler_keys_and_new_events_are_kept() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("settings.json"), r#"{"hooks": {"SomeFutureEvent": [{"hooks": [
            {"type": "command", "command": "x.sh", "async": true, "env": {"A": "1"}},
            {"type": "prompt", "prompt": "unknown handler type"}]}]}}"#).unwrap();
        let entries = load(d.path(), None).value;
        assert_eq!((entries.len(), entries[0].event.as_str(), entries[0].handlers.len()), (1, "SomeFutureEvent", 1));
        let h = &entries[0].handlers[0];
        assert_eq!(h.extra_keys, ["async", "env"]);
        assert_eq!(h.extra["env"]["A"], "1");
    }

    #[test]
    fn a_corrupt_settings_file_is_counted_not_fatal() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("settings.json"), "{ not json").unwrap();
        let loaded = load(d.path(), None);
        assert_eq!((loaded.value.len(), loaded.skipped), (0, 1));
    }
}
