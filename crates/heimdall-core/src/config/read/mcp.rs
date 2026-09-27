//! MCP servers from their five sources (ClaudeConfigLoader.loadMCPServers), in Poirot's
//! precedence:
//! 1. user: top-level `mcpServers` in `~/.claude.json`;
//! 2. local: `projects.<project path>.mcpServers` in `~/.claude.json`, overriding user servers;
//! 3. project: `<project>/.mcp.json`, only for names not already present;
//! 4. plugin servers, found through `mcp__plugin_<name>_<server>__<tool>` permissions;
//! 5. cloud integrations: `claude.ai <name>` keys of the auth cache that are not configured.
//!
//! Env and header values stay [`Secret`](crate::models::secret::Secret)s; only their names serialise.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::Value;

use super::{plugins, settings, sort_by_name};
use crate::error::Loaded;
use crate::models::config::mcp::{
    ClaudeConfig, McpProjectConfig, McpServer, McpServerDefinition, McpServerSource, McpServerStatus,
};
use crate::models::config::scope::ConfigScope;
use crate::paths::{ClaudeHome, TargetOs, same_path};

pub const CLOUD_PREFIX: &str = "claude.ai ";

/// The servers for the global scope, plus a project's when `project_path` is given, sorted by
/// display name. `skipped` counts config files that exist but do not parse.
pub fn load(home: &ClaudeHome, project_path: Option<&str>, os: TargetOs) -> Loaded<Vec<McpServer>> {
    let mut skipped = 0;
    let config: ClaudeConfig = read_json(home.config_file(), &mut skipped).unwrap_or_default();
    let allow = match settings::load(&home.settings_file()) {
        Ok(s) => s.and_then(|s| s.permissions).and_then(|p| p.allow).unwrap_or_default(),
        Err(_) => {
            skipped += 1;
            Vec::new()
        }
    };
    let tools = tool_permissions(&allow);
    let mut by_name: BTreeMap<String, McpServer> = BTreeMap::new();
    let make = |name: &str, def: &McpServerDefinition, scope, source| {
        let known = tools.get(name);
        McpServer::new(
            format!("{}-{name}", source_id(source)),
            format_server_name(name),
            name,
            scope,
            source,
            def,
            known.cloned().unwrap_or_default(),
            known.is_none(),
        )
    };

    for (name, def) in config.mcp_servers.iter().flatten() {
        by_name.insert(
            name.clone(),
            make(name, def, ConfigScope::Global, McpServerSource::User),
        );
    }
    if let Some(project) = project_path {
        // Claude Code keys projects by path; on Windows the key uses `/` while paths elsewhere use `\`.
        let local = config
            .projects
            .iter()
            .flatten()
            .find(|(key, _)| same_path(key, project, os))
            .and_then(|(_, p)| p.mcp_servers.as_ref());
        for (name, def) in local.into_iter().flatten() {
            by_name.insert(
                name.clone(),
                make(name, def, ConfigScope::Project, McpServerSource::Local),
            );
        }
        let project_config: Option<McpProjectConfig> = read_json(&Path::new(project).join(".mcp.json"), &mut skipped);
        for (name, def) in project_config.and_then(|c| c.mcp_servers).into_iter().flatten() {
            by_name
                .entry(name.clone())
                .or_insert_with(|| make(&name, &def, ConfigScope::Project, McpServerSource::Project));
        }
    }
    for server in plugin_servers(&tools, &plugins::enabled_plugins(home)) {
        by_name.insert(server.raw_name.clone(), server);
    }
    for name in auth_cache_names(home) {
        if name.starts_with(CLOUD_PREFIX) && !by_name.contains_key(&name) {
            let def = McpServerDefinition {
                transport: Some("http".into()),
                ..Default::default()
            };
            let mut server = McpServer::new(
                format!("cloud-{name}"),
                &name[CLOUD_PREFIX.len()..],
                name.as_str(),
                ConfigScope::Global,
                McpServerSource::CloudIntegration,
                &def,
                Vec::new(),
                true,
            );
            server.status = McpServerStatus::NeedsAuth;
            by_name.insert(name, server);
        }
    }

    let mut servers: Vec<McpServer> = by_name.into_values().collect();
    sort_by_name(&mut servers, |s| &s.name);
    Loaded::new(servers, skipped)
}

/// Server names in `mcp-needs-auth-cache.json`. Only the keys are read.
pub fn auth_cache_names(home: &ClaudeHome) -> BTreeSet<String> {
    let Ok(bytes) = std::fs::read(home.mcp_auth_cache_file()) else {
        return BTreeSet::new();
    };
    match serde_json::from_slice::<Value>(&bytes) {
        Ok(Value::Object(map)) => map.keys().cloned().collect(),
        _ => BTreeSet::new(),
    }
}

/// `mcp__<server>__<tool>` permissions grouped by server, tools sorted. A server without an entry
/// is "all tools" (wildcard).
pub fn tool_permissions(allow: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut tools: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in allow {
        let Some((server, tool)) = entry.strip_prefix("mcp__").and_then(|rest| rest.split_once("__")) else {
            continue;
        };
        tools.entry(server.to_string()).or_default().push(tool.to_string());
    }
    for list in tools.values_mut() {
        list.sort();
    }
    tools
}

/// `my-server_name` → `My Server Name`.
pub fn format_server_name(raw: &str) -> String {
    raw.split(['-', '_', ' '])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            c.next()
                .map_or_else(String::new, |f| f.to_uppercase().chain(c).collect())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Plugin servers named by their permission keys (`plugin_context-mode_context-mode` →
/// raw `plugin:context-mode:context-mode`, shown as `Context Mode`); a plugin disabled in
/// `enabledPlugins` hides its server.
fn plugin_servers(tools: &BTreeMap<String, Vec<String>>, enabled: &BTreeMap<String, bool>) -> Vec<McpServer> {
    let def = McpServerDefinition {
        transport: Some("stdio".into()),
        ..Default::default()
    };
    tools
        .iter()
        .filter_map(|(permission, server_tools)| {
            let rest = permission.strip_prefix("plugin_")?;
            let plugin = rest.split('_').next().unwrap_or(rest);
            let key = enabled.keys().find(|k| k.split('@').next() == Some(plugin));
            if key.is_some_and(|k| !enabled[k]) {
                return None;
            }
            let raw = format!("plugin:{}", rest.replace('_', ":"));
            Some(McpServer::new(
                format!("plugin-{raw}"),
                format_server_name(plugin),
                raw,
                ConfigScope::Global,
                McpServerSource::Plugin,
                &def,
                server_tools.clone(),
                false,
            ))
        })
        .collect()
}

fn source_id(source: McpServerSource) -> &'static str {
    match source {
        McpServerSource::User => "user",
        McpServerSource::Local => "local",
        McpServerSource::Project => "project",
        McpServerSource::Plugin => "plugin",
        McpServerSource::CloudIntegration => "cloud",
    }
}

/// `None` when the file is missing; a file that does not parse adds to `skipped`.
fn read_json<T: DeserializeOwned>(path: &Path, skipped: &mut u32) -> Option<T> {
    let bytes = std::fs::read(path).ok()?;
    let parsed = serde_json::from_slice(&bytes).ok();
    if parsed.is_none() {
        *skipped += 1;
    }
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::test_support::synthetic;
    use serde_json::json;

    // Ported from PoirotTests/Services/ClaudeConfigLoaderMCPTests.swift (decoding into the M1-10 models).
    #[test]
    fn decode_claude_config_user_scope_servers() {
        let c: ClaudeConfig = serde_json::from_value(json!({"mcpServers": {
            "github": {"type": "stdio", "command": "npx", "args": ["-y", "@modelcontextprotocol/server-github"]},
            "perplexity": {"type": "http", "url": "https://mcp.perplexity.ai", "headers": {"Authorization": "Bearer token"}}}})).unwrap();
        let servers = c.mcp_servers.unwrap();
        let (g, p) = (&servers["github"], &servers["perplexity"]);
        assert_eq!(
            (
                servers.len(),
                g.transport.as_deref(),
                g.command.as_deref(),
                g.url.as_deref()
            ),
            (2, Some("stdio"), Some("npx"), None)
        );
        assert_eq!(
            g.args.as_deref(),
            Some(&["-y".to_string(), "@modelcontextprotocol/server-github".into()][..])
        );
        assert_eq!(
            (p.transport.as_deref(), p.url.as_deref(), p.command.as_deref()),
            (Some("http"), Some("https://mcp.perplexity.ai"), None)
        );
        assert_eq!(p.headers.as_ref().unwrap()["Authorization"].expose(), "Bearer token");
    }
    #[test]
    fn decode_claude_config_local_scope_servers() {
        let c: ClaudeConfig = serde_json::from_value(json!({"projects": {"/Users/dev/my-project": {"mcpServers": {
            "local-db": {"type": "stdio", "command": "node", "args": ["db-server.js"], "env": {"DB_PATH": "/tmp/dev.db"}}}}}})).unwrap();
        let s = &c.projects.unwrap()["/Users/dev/my-project"]
            .mcp_servers
            .clone()
            .unwrap()["local-db"];
        assert_eq!(
            (
                s.transport.as_deref(),
                s.command.as_deref(),
                s.env.as_ref().unwrap()["DB_PATH"].expose()
            ),
            (Some("stdio"), Some("node"), "/tmp/dev.db")
        );
    }
    #[test]
    fn decode_mcp_project_config() {
        let c: McpProjectConfig = serde_json::from_value(
            json!({"mcpServers": {"team-tool": {"type": "http", "url": "http://localhost:3000/mcp"}}}),
        )
        .unwrap();
        let s = &c.mcp_servers.unwrap()["team-tool"];
        assert_eq!(
            (s.transport.as_deref(), s.url.as_deref()),
            (Some("http"), Some("http://localhost:3000/mcp"))
        );
    }
    #[test]
    fn decode_claude_config_empty_object() {
        let c: ClaudeConfig = serde_json::from_value(json!({})).unwrap();
        assert!(c.mcp_servers.is_none() && c.projects.is_none());
    }
    #[test]
    fn decode_mcp_server_definition_minimal_fields() {
        let d: McpServerDefinition = serde_json::from_value(json!({"type": "stdio"})).unwrap();
        assert_eq!(d.transport.as_deref(), Some("stdio"));
        assert!(d.command.is_none() && d.args.is_none() && d.env.is_none() && d.url.is_none() && d.headers.is_none());
    }
    #[test]
    fn decode_mcp_server_definition_all_fields() {
        let d: McpServerDefinition = serde_json::from_value(
            json!({"type": "stdio", "command": "/usr/bin/python3", "args": ["-m", "server"],
            "env": {"API_KEY": "secret"}, "url": "http://fallback.local", "headers": {"X-Custom": "value"}}),
        )
        .unwrap();
        assert_eq!(
            (d.command.as_deref(), d.url.as_deref()),
            (Some("/usr/bin/python3"), Some("http://fallback.local"))
        );
        assert_eq!(
            (
                d.env.unwrap()["API_KEY"].expose(),
                d.headers.unwrap()["X-Custom"].expose()
            ),
            ("secret", "value")
        );
    }
    #[test]
    fn mcp_server_model_new_fields() {
        let def: McpServerDefinition = serde_json::from_value(json!({"type": "stdio", "command": "node", "args": ["server.js", "--port", "3000"], "env": {"NODE_ENV": "production"}})).unwrap();
        let s = McpServer::new(
            "global-test",
            "Test Server",
            "test",
            ConfigScope::Global,
            McpServerSource::User,
            &def,
            vec!["read".into(), "write".into()],
            false,
        );
        assert_eq!(
            (
                s.transport.as_deref(),
                s.command.as_deref(),
                s.args.len(),
                s.env["NODE_ENV"].expose(),
                s.url.as_deref()
            ),
            (Some("stdio"), Some("node"), 3, "production", None)
        );
    }
    #[test]
    fn mcp_server_model_http_type() {
        let def: McpServerDefinition =
            serde_json::from_value(json!({"type": "http", "url": "https://api.example.com/mcp"})).unwrap();
        let s = McpServer::new(
            "global-api",
            "API Server",
            "api",
            ConfigScope::Project,
            McpServerSource::User,
            &def,
            vec![],
            true,
        );
        assert_eq!(
            (s.transport.as_deref(), s.command.as_deref(), s.url.as_deref(), s.scope),
            (
                Some("http"),
                None,
                Some("https://api.example.com/mcp"),
                ConfigScope::Project
            )
        );
    }

    /// A Claude home with every source populated, and the project folder.
    fn world(project_key: &str) -> (tempfile::TempDir, ClaudeHome, std::path::PathBuf) {
        let d = tempfile::tempdir().unwrap();
        let (root, project) = (d.path().join(".claude"), d.path().join("app"));
        std::fs::create_dir_all(&root).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        let config = json!({
            "mcpServers": {"github": {"type": "stdio", "command": "npx"}, "sentry": {"type": "http", "url": "https://mcp.sentry.io"}},
            "projects": {project_key: {"mcpServers": {"github": {"type": "stdio", "command": "local-github"}}}},
        });
        std::fs::write(d.path().join(".claude.json"), config.to_string()).unwrap();
        std::fs::write(project.join(".mcp.json"), json!({"mcpServers": {"github": {"command": "ignored"}, "team-tool": {"type": "http", "url": "http://localhost:3000/mcp"}}}).to_string()).unwrap();
        std::fs::write(root.join("settings.json"), json!({
            "permissions": {"allow": ["mcp__github__search", "mcp__github__get_file", "mcp__plugin_context-mode_context-mode__index", "mcp__plugin_old_old__x", "Read"]},
            "enabledPlugins": {"context-mode@someone": true, "old@someone": false},
        }).to_string()).unwrap();
        std::fs::write(
            root.join("mcp-needs-auth-cache.json"),
            json!({"github": true, "claude.ai Gmail": true}).to_string(),
        )
        .unwrap();
        let home = ClaudeHome::new(root, d.path().join(".claude.json"));
        (d, home, project)
    }

    #[test]
    fn five_sources_follow_poirots_precedence() {
        let (_d, home, project) = world("PLACEHOLDER");
        // Rewrite the local key to the real project path, as Claude Code stores it.
        let cfg = std::fs::read_to_string(home.config_file())
            .unwrap()
            .replace("PLACEHOLDER", &project.display().to_string().replace('\\', "\\\\"));
        std::fs::write(home.config_file(), cfg).unwrap();
        let servers = load(&home, Some(&project.display().to_string()), TargetOs::current()).value;
        let got: Vec<_> = servers
            .iter()
            .map(|s| (s.raw_name.as_str(), s.source, s.command.as_deref()))
            .collect();
        assert_eq!(
            got,
            [
                ("plugin:context-mode:context-mode", McpServerSource::Plugin, None),
                ("github", McpServerSource::Local, Some("local-github")),
                ("claude.ai Gmail", McpServerSource::CloudIntegration, None),
                ("sentry", McpServerSource::User, None),
                ("team-tool", McpServerSource::Project, None),
            ]
        );
        let github = &servers[1];
        assert_eq!(
            (github.tools.clone(), github.is_wildcard, github.name.as_str()),
            (vec!["get_file".to_string(), "search".into()], false, "Github")
        );
        assert!(servers[4].is_wildcard);
        assert_eq!(
            (servers[0].name.as_str(), servers[2].name.as_str(), servers[2].status),
            ("Context Mode", "Gmail", McpServerStatus::NeedsAuth)
        );
        // Without a project only global sources count.
        assert_eq!(
            load(&home, None, TargetOs::current())
                .value
                .iter()
                .map(|s| s.raw_name.as_str())
                .collect::<Vec<_>>(),
            [
                "plugin:context-mode:context-mode",
                "github",
                "claude.ai Gmail",
                "sentry"
            ]
        );
    }

    #[test]
    fn windows_project_keys_match_whatever_the_separator() {
        let (_d, home, _project) = world("C:/Users/me/app");
        let servers = load(&home, Some(r"c:\Users\me\app"), TargetOs::Windows).value;
        assert!(servers.iter().any(|s| s.source == McpServerSource::Local));
    }

    #[test]
    fn serialised_servers_never_carry_secret_values() {
        let home = ClaudeHome::new(synthetic().join("claude-home"), synthetic().join("claude.json"));
        let servers = load(&home, Some("/Users/fixture/src/app"), TargetOs::Macos).value;
        assert!(
            servers
                .iter()
                .any(|s| s.raw_name == "local-db" && s.env_keys == ["DB_PASSWORD", "DB_PATH"])
        );
        let json = serde_json::to_string(&servers).unwrap();
        assert!(!json.contains("SENTINEL-SECRET"), "{json}");
    }
}
