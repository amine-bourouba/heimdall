//! `~/.claude.json` top-level MCP server writes (SettingsWriter.swift:84-197's own `addMCPServer`/
//! `updateMCPServer`/`removeMCPServer`/`lineNumber(forMCPServer:)`). Only the global, top-level
//! `mcpServers` object -- `docs/ipc-contract.md`'s own `add_mcp_server`/`update_mcp_server`/
//! `remove_mcp_server` take no project/scope parameter at all, matching Swift's own
//! `claudeConfigFileURL()`-only scope exactly. Project-scoped `projects.<path>.mcpServers` and
//! `<project>/.mcp.json` are read-only sources (`config/read/mcp.rs`'s own precedence list), never
//! written here.
//!
//! A real, confirmed bug this task's own AC1 names and fixes: Swift's own `updateMCPServer`
//! (`SettingsWriter.swift:150-177`) always writes a fresh `{"command", "args", "env"}` object from
//! its own three plain parameters, regardless of what the server being edited actually is -- an
//! HTTP or SSE server (`type`/`url`/`headers`, no `command`/`args` at all) silently loses every one
//! of those fields the moment its edit form is submitted, since that function's own signature has
//! nowhere to put them. This never comes up for Heimdall's own `McpServerInput`: its `transport`
//! field is required and its own command/url/env/headers always fully describe the chosen
//! transport, so every write below constructs a complete, correct object from the input directly,
//! never a partial patch of the old one -- there is no way to reach the Swift bug's own precondition
//! (the fields it drops were never modeled on this input's own shape to begin with).

use std::path::Path;

use serde_json::{Map, Value};

use super::json_doc::JsonDoc;
use crate::error::{AppError, Result, WriteOutcome};
use crate::models::config::mcp::{McpServerDefinition, McpServerInput, McpServerInputTransport, SecretInput};
use crate::models::secret::Secret;

fn open_or_unchanged(path: &Path, create_if_missing: bool) -> Result<Option<JsonDoc>> {
    match JsonDoc::open(path, create_if_missing) {
        Ok(doc) => Ok(Some(doc)),
        Err(AppError::NotFound { .. }) if !create_if_missing => Ok(None),
        Err(e) => Err(e),
    }
}

fn servers_object(root: &mut Map<String, Value>) -> &mut Map<String, Value> {
    root.entry("mcpServers")
        .or_insert_with(|| Value::Object(Map::new()))
        .as_object_mut()
        .expect("just inserted as an object")
}

/// `SecretInput::Keep` reads the old value back from `old` (`None` if there was none to keep, a
/// sensible default rather than an error); `Set` replaces it; `Remove` drops the key entirely.
/// This task's own AC2.
fn resolve_secret(input: &SecretInput, old: Option<&Secret>) -> Option<Secret> {
    match input {
        SecretInput::Keep => old.cloned(),
        SecretInput::Set { value } => Some(Secret::new(value.clone())),
        SecretInput::Remove => None,
    }
}

fn resolve_secrets(
    input: &std::collections::BTreeMap<String, SecretInput>,
    old: Option<&std::collections::BTreeMap<String, Secret>>,
) -> Map<String, Value> {
    input
        .iter()
        .filter_map(|(k, si)| {
            let old_value = old.and_then(|m| m.get(k));
            resolve_secret(si, old_value).map(|s| (k.clone(), Value::String(s.expose().to_string())))
        })
        .collect()
}

/// The complete, correct server object for `input`, given `old` (the previously-stored definition,
/// for secret `Keep` resolution only -- `None` for a brand new server). Every field this task's own
/// Goal names (`type`, `url`, `headers`, `command`, `args` as an array, `env`) is written based
/// entirely on `input`'s own transport; nothing is conditionally carried over from `old` except a
/// kept secret's own value.
fn server_json(input: &McpServerInput, old: Option<&McpServerDefinition>) -> Value {
    let mut obj = Map::new();
    match input.transport {
        McpServerInputTransport::Stdio => {
            obj.insert("type".to_string(), Value::String("stdio".to_string()));
            if let Some(command) = &input.command {
                obj.insert("command".to_string(), Value::String(command.clone()));
            }
            obj.insert(
                "args".to_string(),
                Value::Array(input.args.iter().cloned().map(Value::String).collect()),
            );
            let env = resolve_secrets(&input.env, old.and_then(|o| o.env.as_ref()));
            if !env.is_empty() {
                obj.insert("env".to_string(), Value::Object(env));
            }
        }
        McpServerInputTransport::Http | McpServerInputTransport::Sse => {
            let transport = if matches!(input.transport, McpServerInputTransport::Http) {
                "http"
            } else {
                "sse"
            };
            obj.insert("type".to_string(), Value::String(transport.to_string()));
            if let Some(url) = &input.url {
                obj.insert("url".to_string(), Value::String(url.clone()));
            }
            let headers = resolve_secrets(&input.headers, old.and_then(|o| o.headers.as_ref()));
            if !headers.is_empty() {
                obj.insert("headers".to_string(), Value::Object(headers));
            }
        }
    }
    Value::Object(obj)
}

pub fn add_server(config_file: &Path, input: &McpServerInput) -> Result<WriteOutcome> {
    let mut doc = JsonDoc::open(config_file, true)?;
    let servers = servers_object(doc.root_mut()?);
    servers.insert(input.name.clone(), server_json(input, None));
    doc.save()
}

/// `original_name` may differ from `input.name` (a rename); the old entry is dropped either way
/// and the new one inserted under whatever name the input gives, matching Swift's own
/// `updateMCPServer(originalName:newName:...)`.
pub fn update_server(config_file: &Path, original_name: &str, input: &McpServerInput) -> Result<WriteOutcome> {
    let mut doc = JsonDoc::open(config_file, true)?;
    let servers = servers_object(doc.root_mut()?);
    let old: Option<McpServerDefinition> = servers
        .get(original_name)
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    servers.remove(original_name);
    servers.insert(input.name.clone(), server_json(input, old.as_ref()));
    doc.save()
}

pub fn remove_server(config_file: &Path, name: &str) -> Result<WriteOutcome> {
    let Some(mut doc) = open_or_unchanged(config_file, false)? else {
        return Ok(WriteOutcome::Unchanged);
    };
    let root = doc.root_mut()?;
    let Some(servers) = root.get_mut("mcpServers").and_then(Value::as_object_mut) else {
        return Ok(WriteOutcome::Unchanged);
    };
    if servers.remove(name).is_none() {
        return Ok(WriteOutcome::Unchanged);
    }
    doc.save()
}

/// Ported from `SettingsWriter.lineNumber(forMCPServer:)`: a plain text scan (not JSON-aware) for
/// "open at line" editor integration, scoped to the *top-level* `mcpServers` block only (tracked
/// via brace depth and a `<= 2`-space indent check on the block's own opening key) so a same-named
/// server nested under `projects.<path>.mcpServers` is never matched instead. 1-indexed, `None` if
/// the file is unreadable or the key is not found in that block.
pub fn line_number_for_server(config_file: &Path, server_key: &str) -> Option<usize> {
    let content = std::fs::read_to_string(config_file).ok()?;
    let needle = format!("\"{server_key}\"");
    let mut in_top_level = false;
    let mut brace_depth: i32 = 0;
    for (index, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        let indent = line.chars().take_while(|c| *c == ' ' || *c == '\t').count();
        if trimmed.starts_with("\"mcpServers\"") && indent <= 2 {
            in_top_level = true;
            brace_depth = 0;
        }
        if in_top_level {
            brace_depth += trimmed.matches('{').count() as i32;
            brace_depth -= trimmed.matches('}').count() as i32;
            if trimmed.starts_with(&needle) {
                return Some(index + 1);
            }
            if brace_depth <= 0 && trimmed.contains('}') {
                in_top_level = false;
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn config_with(dir: &Path, body: &str) -> std::path::PathBuf {
        let path = dir.join(".claude.json");
        std::fs::write(&path, body).unwrap();
        path
    }

    fn read(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    fn stdio_input(name: &str, command: &str, env: BTreeMap<String, SecretInput>) -> McpServerInput {
        McpServerInput {
            name: name.to_string(),
            transport: McpServerInputTransport::Stdio,
            command: Some(command.to_string()),
            args: vec!["--flag".to_string()],
            url: None,
            env,
            headers: BTreeMap::new(),
        }
    }

    fn http_input(name: &str, url: &str, headers: BTreeMap<String, SecretInput>) -> McpServerInput {
        McpServerInput {
            name: name.to_string(),
            transport: McpServerInputTransport::Http,
            command: None,
            args: vec![],
            url: Some(url.to_string()),
            env: BTreeMap::new(),
            headers,
        }
    }

    // MARK: - Add

    #[test]
    fn adding_a_stdio_server_writes_type_command_args_and_env() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), "{}");
        let mut env = BTreeMap::new();
        env.insert(
            "API_KEY".to_string(),
            SecretInput::Set {
                value: "sk-123".to_string(),
            },
        );
        add_server(&path, &stdio_input("gh", "node", env)).unwrap();
        let v = read(&path);
        assert_eq!(
            v["mcpServers"]["gh"],
            serde_json::json!({"type": "stdio", "command": "node", "args": ["--flag"], "env": {"API_KEY": "sk-123"}})
        );
    }

    #[test]
    fn adding_an_http_server_writes_type_url_and_headers_never_command_or_args() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), "{}");
        let mut headers = BTreeMap::new();
        headers.insert(
            "Authorization".to_string(),
            SecretInput::Set {
                value: "Bearer t".to_string(),
            },
        );
        add_server(&path, &http_input("remote", "https://x", headers)).unwrap();
        let v = read(&path);
        assert_eq!(
            v["mcpServers"]["remote"],
            serde_json::json!({"type": "http", "url": "https://x", "headers": {"Authorization": "Bearer t"}})
        );
        assert!(v["mcpServers"]["remote"].get("command").is_none());
        assert!(v["mcpServers"]["remote"].get("args").is_none());
    }

    #[test]
    fn adding_preserves_other_top_level_keys_and_other_servers() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"existing": {"type": "stdio", "command": "x"}}, "numStartups": 5}"#,
        );
        add_server(&path, &stdio_input("new", "node", BTreeMap::new())).unwrap();
        let v = read(&path);
        assert_eq!(v["numStartups"], 5);
        assert_eq!(v["mcpServers"]["existing"]["command"], "x");
        assert_eq!(v["mcpServers"]["new"]["command"], "node");
    }

    // MARK: - Update: AC1, the real Swift bug this task fixes

    #[test]
    fn updating_an_http_server_keeps_type_url_and_headers_unlike_swifts_own_bug() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"remote": {"type": "http", "url": "https://old", "headers": {"Authorization": "Bearer old"}}}}"#,
        );
        let mut headers = BTreeMap::new();
        headers.insert("Authorization".to_string(), SecretInput::Keep);
        update_server(&path, "remote", &http_input("remote", "https://new", headers)).unwrap();
        let v = read(&path);
        // Swift's own `updateMCPServer` would have overwritten this whole entry with only
        // `{"command", "args", "env"}` -- `type`/`url`/`headers` all present here is this task's
        // own fix, not an accident of the test data.
        assert_eq!(
            v["mcpServers"]["remote"],
            serde_json::json!({"type": "http", "url": "https://new", "headers": {"Authorization": "Bearer old"}})
        );
    }

    #[test]
    fn updating_with_a_new_name_renames_the_server() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"old-name": {"type": "stdio", "command": "node", "args": []}}}"#,
        );
        update_server(&path, "old-name", &stdio_input("new-name", "node", BTreeMap::new())).unwrap();
        let v = read(&path);
        assert!(v["mcpServers"].get("old-name").is_none());
        assert_eq!(v["mcpServers"]["new-name"]["command"], "node");
    }

    #[test]
    fn updating_a_server_that_does_not_exist_yet_just_adds_it() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), "{}");
        update_server(&path, "nonexistent", &stdio_input("brand-new", "node", BTreeMap::new())).unwrap();
        assert_eq!(read(&path)["mcpServers"]["brand-new"]["command"], "node");
    }

    // MARK: - Secret keep/set/remove: AC2

    #[test]
    fn keep_with_an_old_value_leaves_it_untouched() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"gh": {"type": "stdio", "command": "node", "args": [], "env": {"API_KEY": "sk-original"}}}}"#,
        );
        let mut env = BTreeMap::new();
        env.insert("API_KEY".to_string(), SecretInput::Keep);
        update_server(&path, "gh", &stdio_input("gh", "node", env)).unwrap();
        assert_eq!(read(&path)["mcpServers"]["gh"]["env"]["API_KEY"], "sk-original");
    }

    #[test]
    fn set_replaces_the_value() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"gh": {"type": "stdio", "command": "node", "args": [], "env": {"API_KEY": "sk-original"}}}}"#,
        );
        let mut env = BTreeMap::new();
        env.insert(
            "API_KEY".to_string(),
            SecretInput::Set {
                value: "sk-new".to_string(),
            },
        );
        update_server(&path, "gh", &stdio_input("gh", "node", env)).unwrap();
        assert_eq!(read(&path)["mcpServers"]["gh"]["env"]["API_KEY"], "sk-new");
    }

    #[test]
    fn remove_drops_the_key_and_an_empty_env_is_omitted_entirely() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"gh": {"type": "stdio", "command": "node", "args": [], "env": {"API_KEY": "sk-original"}}}}"#,
        );
        let mut env = BTreeMap::new();
        env.insert("API_KEY".to_string(), SecretInput::Remove);
        update_server(&path, "gh", &stdio_input("gh", "node", env)).unwrap();
        assert!(read(&path)["mcpServers"]["gh"].get("env").is_none());
    }

    #[test]
    fn keep_with_no_old_value_at_all_is_simply_absent_not_an_error() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), "{}");
        let mut env = BTreeMap::new();
        env.insert("API_KEY".to_string(), SecretInput::Keep);
        add_server(&path, &stdio_input("gh", "node", env)).unwrap();
        assert!(read(&path)["mcpServers"]["gh"].get("env").is_none());
    }

    // MARK: - Remove

    #[test]
    fn removing_a_server_drops_only_that_key() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            r#"{"mcpServers": {"a": {"type": "stdio", "command": "x"}, "b": {"type": "stdio", "command": "y"}}}"#,
        );
        remove_server(&path, "a").unwrap();
        let v = read(&path);
        assert!(v["mcpServers"].get("a").is_none());
        assert_eq!(v["mcpServers"]["b"]["command"], "y");
    }

    #[test]
    fn removing_from_a_missing_file_is_a_no_op() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(
            remove_server(&d.path().join(".claude.json"), "a").unwrap(),
            WriteOutcome::Unchanged
        );
        assert!(!d.path().join(".claude.json").exists());
    }

    #[test]
    fn removing_a_name_that_does_not_exist_is_a_no_op() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), r#"{"mcpServers": {"a": {"type": "stdio", "command": "x"}}}"#);
        assert_eq!(remove_server(&path, "nonexistent").unwrap(), WriteOutcome::Unchanged);
        assert_eq!(read(&path)["mcpServers"]["a"]["command"], "x");
    }

    // MARK: - Corrupt file: the same AC2 discipline as M3-40's settings.rs

    #[test]
    fn every_write_on_a_corrupt_file_is_a_parse_error_and_leaves_it_untouched() {
        let d = tempfile::tempdir().unwrap();
        let corrupt = b"{ not json".to_vec();
        let path = config_with(d.path(), "{ not json");
        assert!(matches!(
            add_server(&path, &stdio_input("a", "node", BTreeMap::new())).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(
            update_server(&path, "a", &stdio_input("a", "node", BTreeMap::new())).unwrap_err(),
            AppError::Parse { .. }
        ));
        assert!(matches!(remove_server(&path, "a").unwrap_err(), AppError::Parse { .. }));
        assert_eq!(std::fs::read(&path).unwrap(), corrupt);
    }

    // MARK: - Line number lookup: AC3

    #[test]
    fn finds_the_line_of_a_top_level_server_key() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            "{\n  \"mcpServers\": {\n    \"first\": {\n      \"command\": \"a\"\n    },\n    \"second\": {\n      \"command\": \"b\"\n    }\n  }\n}\n",
        );
        assert_eq!(line_number_for_server(&path, "first"), Some(3));
        assert_eq!(line_number_for_server(&path, "second"), Some(6));
    }

    #[test]
    fn does_not_match_a_same_named_key_nested_under_a_projects_mcp_servers() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(
            d.path(),
            "{\n  \"mcpServers\": {\n    \"top-level\": {\n      \"command\": \"a\"\n    }\n  },\n  \"projects\": {\n    \"/x\": {\n      \"mcpServers\": {\n        \"top-level\": {\n          \"command\": \"b\"\n        }\n      }\n    }\n  }\n}\n",
        );
        // Only the real top-level occurrence (line 3) is found -- the identically-named key
        // nested under `projects./x.mcpServers` (line 9) is never matched.
        assert_eq!(line_number_for_server(&path, "top-level"), Some(3));
    }

    #[test]
    fn returns_none_for_an_unknown_key_or_unreadable_file() {
        let d = tempfile::tempdir().unwrap();
        let path = config_with(d.path(), r#"{"mcpServers": {"a": {"command": "x"}}}"#);
        assert_eq!(line_number_for_server(&path, "nonexistent"), None);
        assert_eq!(line_number_for_server(&d.path().join("missing.json"), "a"), None);
    }
}
