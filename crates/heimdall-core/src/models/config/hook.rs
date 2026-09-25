//! Hooks (Models/Config/Hook.swift). Unknown handler keys are kept so a save never drops them
//! (port-plan C-3), but only their names cross IPC: an http hook's extra fields may hold credentials.

use serde::{Deserialize, Serialize};
use specta::Type;

use super::scope::ConfigScope;

/// The 18 hook events Poirot knows, with their settings.json spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
pub enum HookEvent {
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    Notification,
    SubagentStart,
    SubagentStop,
    Stop,
    TeammateIdle,
    TaskCompleted,
    InstructionsLoaded,
    ConfigChange,
    WorktreeCreate,
    WorktreeRemove,
    PreCompact,
}

impl HookEvent {
    pub const ALL: [Self; 18] = [
        Self::PreToolUse, Self::PostToolUse, Self::PostToolUseFailure, Self::PermissionRequest, Self::SessionStart,
        Self::SessionEnd, Self::UserPromptSubmit, Self::Notification, Self::SubagentStart, Self::SubagentStop, Self::Stop,
        Self::TeammateIdle, Self::TaskCompleted, Self::InstructionsLoaded, Self::ConfigChange, Self::WorktreeCreate,
        Self::WorktreeRemove, Self::PreCompact,
    ];

    pub fn raw_value(self) -> &'static str {
        match self {
            Self::PreToolUse => "PreToolUse",
            Self::PostToolUse => "PostToolUse",
            Self::PostToolUseFailure => "PostToolUseFailure",
            Self::PermissionRequest => "PermissionRequest",
            Self::SessionStart => "SessionStart",
            Self::SessionEnd => "SessionEnd",
            Self::UserPromptSubmit => "UserPromptSubmit",
            Self::Notification => "Notification",
            Self::SubagentStart => "SubagentStart",
            Self::SubagentStop => "SubagentStop",
            Self::Stop => "Stop",
            Self::TeammateIdle => "TeammateIdle",
            Self::TaskCompleted => "TaskCompleted",
            Self::InstructionsLoaded => "InstructionsLoaded",
            Self::ConfigChange => "ConfigChange",
            Self::WorktreeCreate => "WorktreeCreate",
            Self::WorktreeRemove => "WorktreeRemove",
            Self::PreCompact => "PreCompact",
        }
    }

    pub fn from_raw(raw: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|e| e.raw_value() == raw)
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::PreToolUse => "Before a tool call executes (can block)",
            Self::PostToolUse => "After a tool call succeeds",
            Self::PostToolUseFailure => "After a tool call fails",
            Self::PermissionRequest => "When a permission dialog appears",
            Self::SessionStart => "When a session begins or resumes",
            Self::SessionEnd => "When a session terminates",
            Self::UserPromptSubmit => "When a prompt is submitted",
            Self::Notification => "When a notification is sent",
            Self::SubagentStart => "When a subagent is spawned",
            Self::SubagentStop => "When a subagent finishes",
            Self::Stop => "When Claude finishes responding",
            Self::TeammateIdle => "When a teammate is about to go idle",
            Self::TaskCompleted => "When a task is marked completed",
            Self::InstructionsLoaded => "When CLAUDE.md or rules are loaded",
            Self::ConfigChange => "When a config file changes",
            Self::WorktreeCreate => "When a worktree is being created",
            Self::WorktreeRemove => "When a worktree is being removed",
            Self::PreCompact => "Before context compaction",
        }
    }

    /// Events whose hooks take a matcher (Hook.swift:52-60).
    pub fn supports_matcher(self) -> bool {
        !matches!(
            self,
            Self::UserPromptSubmit | Self::Stop | Self::TeammateIdle | Self::TaskCompleted | Self::WorktreeCreate
                | Self::WorktreeRemove | Self::InstructionsLoaded
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum HookHandlerType {
    Command,
    Http,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HookHandler {
    #[serde(rename = "type")]
    pub handler_type: HookHandlerType,
    pub command: Option<String>,
    pub url: Option<String>,
    pub timeout: Option<u64>,
    pub status_message: Option<String>,
    /// Names of keys Heimdall does not model; their values stay in Rust.
    pub extra_keys: Vec<String>,
    #[serde(skip)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl HookHandler {
    /// Parse one handler object from settings.json. Returns `None` for an unknown handler type.
    pub fn from_json(obj: &serde_json::Map<String, serde_json::Value>) -> Option<Self> {
        let handler_type = match obj.get("type").and_then(|v| v.as_str()) {
            Some("command") => HookHandlerType::Command,
            Some("http") => HookHandlerType::Http,
            _ => return None,
        };
        let s = |k: &str| obj.get(k).and_then(|v| v.as_str()).map(String::from);
        let extra: serde_json::Map<_, _> = obj
            .iter()
            .filter(|(k, _)| !matches!(k.as_str(), "type" | "command" | "url" | "timeout" | "statusMessage"))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        Some(Self {
            handler_type,
            command: s("command"),
            url: s("url"),
            timeout: obj.get("timeout").and_then(|v| v.as_u64()),
            status_message: s("statusMessage"),
            extra_keys: extra.keys().cloned().collect(),
            extra,
        })
    }

    /// The settings.json object, unknown keys included.
    pub fn to_json(&self) -> serde_json::Map<String, serde_json::Value> {
        use serde_json::Value;
        let mut obj = self.extra.clone();
        let kind = match self.handler_type {
            HookHandlerType::Command => "command",
            HookHandlerType::Http => "http",
        };
        obj.insert("type".into(), kind.into());
        let fields = [("command", self.command.clone().map(Value::from)), ("url", self.url.clone().map(Value::from)),
                      ("timeout", self.timeout.map(Value::from)), ("statusMessage", self.status_message.clone().map(Value::from))];
        for (k, v) in fields {
            if let Some(v) = v {
                obj.insert(k.into(), v);
            }
        }
        obj
    }

    pub fn display_command(&self) -> &str {
        match self.handler_type {
            HookHandlerType::Command => self.command.as_deref().unwrap_or(""),
            HookHandlerType::Http => self.url.as_deref().unwrap_or(""),
        }
    }

    /// Seconds Claude Code waits when no timeout is set.
    pub fn default_timeout(&self) -> u64 {
        match self.handler_type {
            HookHandlerType::Command => 600,
            HookHandlerType::Http => 30,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HookMatcherGroup {
    pub matcher: Option<String>,
    pub handlers: Vec<HookHandler>,
}

/// One matcher group, flattened with its event and scope for display (contract `HookEntry`).
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HookEntry {
    /// The settings.json key; kept as text so events newer than Heimdall still show up.
    pub event: String,
    pub matcher_group_index: u32,
    pub matcher: Option<String>,
    pub handlers: Vec<HookHandler>,
    pub scope: ConfigScope,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn events_round_trip_their_raw_names_and_matcher_support() {
        assert_eq!(HookEvent::ALL.len(), 18);
        for e in HookEvent::ALL {
            assert_eq!(HookEvent::from_raw(e.raw_value()), Some(e));
        }
        assert!(HookEvent::PreToolUse.supports_matcher());
        assert!(!HookEvent::Stop.supports_matcher());
        assert_eq!(HookEvent::ALL.iter().filter(|e| e.supports_matcher()).count(), 11);
    }

    #[test]
    fn handler_keeps_unknown_keys_but_only_exposes_their_names() {
        let obj = json!({"type": "http", "url": "https://h", "timeout": 5, "headers": {"Authorization": "secret"}});
        let h = HookHandler::from_json(obj.as_object().unwrap()).unwrap();
        assert_eq!((h.display_command(), h.timeout, h.default_timeout()), ("https://h", Some(5), 30));
        assert_eq!(h.extra_keys, ["headers"]);
        let wire = serde_json::to_string(&h).unwrap();
        assert!(!wire.contains("secret") && wire.contains(r#""extraKeys":["headers"]"#));
    }

    #[test]
    fn handler_round_trips_every_key() {
        let obj = json!({"type": "command", "command": "lint.sh", "timeout": 10, "async": true, "env": {"A": "b"}});
        let h = HookHandler::from_json(obj.as_object().unwrap()).unwrap();
        assert_eq!(serde_json::Value::Object(h.to_json()), obj);
    }

    #[test]
    fn unknown_handler_type_is_not_parsed() {
        assert!(HookHandler::from_json(json!({"type": "websocket"}).as_object().unwrap()).is_none());
    }
}
