//! MCP server definitions as stored on disk (Models/Config/ClaudeConfig.swift) and the server model
//! shown in the app (Models/Config/MCPServer.swift). Env and header values stay [`Secret`]s.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use specta::Type;

use super::scope::ConfigScope;
use crate::models::secret::Secret;

/// The parts of `~/.claude.json` Heimdall reads. Every other key is ignored.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ClaudeConfig {
    pub mcp_servers: Option<BTreeMap<String, McpServerDefinition>>,
    pub projects: Option<BTreeMap<String, ProjectConfig>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ProjectConfig {
    pub mcp_servers: Option<BTreeMap<String, McpServerDefinition>>,
}

/// `<project>/.mcp.json`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct McpProjectConfig {
    pub mcp_servers: Option<BTreeMap<String, McpServerDefinition>>,
}

/// One server entry. All fields are optional on disk; `type`, `url` and `headers` are kept so an
/// update never drops them (port-plan C-3).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct McpServerDefinition {
    #[serde(rename = "type")]
    pub transport: Option<String>,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<BTreeMap<String, Secret>>,
    pub url: Option<String>,
    pub headers: Option<BTreeMap<String, Secret>>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum McpServerStatus {
    Connected,
    NeedsAuth,
    Failed,
    Unreachable,
    Starting,
    #[default]
    Unknown,
}

impl McpServerStatus {
    pub const ALL: [Self; 6] = [
        Self::Connected,
        Self::NeedsAuth,
        Self::Failed,
        Self::Unreachable,
        Self::Starting,
        Self::Unknown,
    ];

    /// The wire value (`connected`, `needsAuth`, …).
    pub const fn raw_value(self) -> &'static str {
        match self {
            Self::Connected => "connected",
            Self::NeedsAuth => "needsAuth",
            Self::Failed => "failed",
            Self::Unreachable => "unreachable",
            Self::Starting => "starting",
            Self::Unknown => "unknown",
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::NeedsAuth => "Needs Auth",
            Self::Failed => "Failed",
            Self::Unreachable => "Unreachable",
            Self::Starting => "Starting",
            Self::Unknown => "Unknown",
        }
    }

    pub const fn accessibility_label(self) -> &'static str {
        match self {
            Self::Connected => "Server is connected and running",
            Self::NeedsAuth => "Server requires authentication",
            Self::Failed => "Server failed to start",
            Self::Unreachable => "Server is unreachable",
            Self::Starting => "Server is starting up",
            Self::Unknown => "Server status is unknown",
        }
    }
}

/// Where a server was configured. The Swift reference app folds the first three into one `user` value; Heimdall keeps them apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum McpServerSource {
    /// Top-level `mcpServers` in `~/.claude.json`.
    User,
    /// `projects.<path>.mcpServers` in `~/.claude.json`.
    Local,
    /// `<project>/.mcp.json`.
    Project,
    Plugin,
    CloudIntegration,
}

/// A server as listed in the app. Secret values are never serialised: IPC sees only key names.
#[derive(Debug, Clone, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub raw_name: String,
    pub scope: ConfigScope,
    pub source: McpServerSource,
    #[serde(rename = "type")]
    pub transport: Option<String>,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env_keys: Vec<String>,
    pub header_keys: Vec<String>,
    pub tools: Vec<String>,
    pub is_wildcard: bool,
    /// Sent separately as `McpStatus` by the status probe (contract).
    #[serde(skip)]
    pub status: McpServerStatus,
    #[serde(skip)]
    pub env: BTreeMap<String, Secret>,
    #[serde(skip)]
    pub headers: BTreeMap<String, Secret>,
}

impl McpServer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        raw_name: impl Into<String>,
        scope: ConfigScope,
        source: McpServerSource,
        def: &McpServerDefinition,
        tools: Vec<String>,
        is_wildcard: bool,
    ) -> Self {
        let env = def.env.clone().unwrap_or_default();
        let headers = def.headers.clone().unwrap_or_default();
        Self {
            id: id.into(),
            name: name.into(),
            raw_name: raw_name.into(),
            scope,
            source,
            transport: def.transport.clone(),
            command: def.command.clone(),
            args: def.args.clone().unwrap_or_default(),
            url: def.url.clone(),
            env_keys: env.keys().cloned().collect(),
            header_keys: headers.keys().cloned().collect(),
            tools,
            is_wildcard,
            status: McpServerStatus::Unknown,
            env,
            headers,
        }
    }
}

/// A secret field's edit (contract `SecretInput`): existing values never round-trip through the
/// webview (ADR 0010), so an unchanged field is `"keep"`, not its old value.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SecretInput {
    Keep,
    /// The raw typed value; wrapped in a [`Secret`] only once the write command stores it.
    Set {
        value: String,
    },
    Remove,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum McpServerInputTransport {
    Stdio,
    Http,
    Sse,
}

/// A server as submitted by the add/edit form (contract `McpServerInput`).
#[derive(Debug, Clone, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct McpServerInput {
    pub name: String,
    #[serde(rename = "type")]
    pub transport: McpServerInputTransport,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub url: Option<String>,
    pub env: BTreeMap<String, SecretInput>,
    pub headers: BTreeMap<String, SecretInput>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn server() -> McpServer {
        let def: McpServerDefinition =
            serde_json::from_str(r#"{"type": "stdio", "command": "node", "args": [], "env": {"API_KEY": "sk-123"}}"#)
                .unwrap();
        McpServer::new(
            "test-server",
            "Test",
            "test",
            ConfigScope::Global,
            McpServerSource::User,
            &def,
            vec![],
            false,
        )
    }

    // Ported from PoirotTests/Models/MCPServerStatusTests.swift. The SF Symbol cases become
    // "every status has an icon" in src/features/config/mcp/McpStatusIndicator.vue (M3-14).
    #[test]
    fn all_cases_have_label() {
        assert!(McpServerStatus::ALL.iter().all(|s| !s.label().is_empty()));
    }
    #[test]
    fn all_cases_have_accessibility_label() {
        assert!(McpServerStatus::ALL.iter().all(|s| !s.accessibility_label().is_empty()));
    }
    #[test]
    fn status_labels() {
        assert_eq!(
            McpServerStatus::ALL.map(McpServerStatus::label),
            [
                "Connected",
                "Needs Auth",
                "Failed",
                "Unreachable",
                "Starting",
                "Unknown"
            ]
        );
    }
    #[test]
    fn status_raw_values() {
        assert_eq!(
            McpServerStatus::ALL.map(McpServerStatus::raw_value),
            ["connected", "needsAuth", "failed", "unreachable", "starting", "unknown"]
        );
        for s in McpServerStatus::ALL {
            assert_eq!(serde_json::to_value(s).unwrap(), s.raw_value());
        }
    }
    #[test]
    fn server_default_status() {
        assert_eq!(server().status, McpServerStatus::Unknown);
    }
    #[test]
    fn server_explicit_status() {
        let s = McpServer {
            status: McpServerStatus::Connected,
            ..server()
        };
        assert_eq!(s.status, McpServerStatus::Connected);
    }
    #[test]
    fn server_mutable_status() {
        let mut s = server();
        s.status = McpServerStatus::NeedsAuth;
        assert_eq!(s.status, McpServerStatus::NeedsAuth);
    }

    #[test]
    fn serialised_server_carries_env_key_names_but_never_values() {
        let json = serde_json::to_string(&server()).unwrap();
        assert!(!json.contains("sk-123"));
        assert!(json.contains(r#""envKeys":["API_KEY"]"#));
        assert!(json.contains(r#""type":"stdio""#));
    }

    #[test]
    fn definitions_keep_type_url_and_headers() {
        let def: McpServerDefinition = serde_json::from_str(
            r#"{"type": "http", "url": "https://x", "headers": {"Authorization": "Bearer t"}, "extra": 1}"#,
        )
        .unwrap();
        assert_eq!(
            (def.transport.as_deref(), def.url.as_deref()),
            (Some("http"), Some("https://x"))
        );
        assert_eq!(def.headers.unwrap()["Authorization"].expose(), "Bearer t");
    }
}
