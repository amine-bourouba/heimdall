//! Sub-agents (Models/Config/SubAgent.swift). Icons are SF Symbol names; the webview maps them to Lucide (M2-04).

use serde::{Deserialize, Serialize};
use specta::Type;

use super::scope::ConfigRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AgentScope {
    BuiltIn,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AgentColor {
    Orange,
    Red,
    Blue,
    Green,
    Purple,
    Teal,
}

impl AgentColor {
    pub const ALL: [Self; 6] = [Self::Orange, Self::Red, Self::Blue, Self::Green, Self::Purple, Self::Teal];

    pub fn from_raw(raw: &str) -> Option<Self> {
        serde_json::from_value(serde_json::Value::String(raw.to_string())).ok()
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Orange => "Orange",
            Self::Red => "Red",
            Self::Blue => "Blue",
            Self::Green => "Green",
            Self::Purple => "Purple",
            Self::Teal => "Teal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AgentMemory {
    User,
    None,
}

impl AgentMemory {
    pub const fn label(self) -> &'static str {
        match self {
            Self::User => "Enable (~/.claude/agent-memory/)",
            Self::None => "None (no persistent memory)",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ToolCategory {
    pub id: &'static str,
    pub name: &'static str,
    pub tools: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SubAgent {
    /// File slug for custom agents, fixed id for built-ins.
    pub id: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub tools: Vec<String>,
    pub model: Option<String>,
    pub color: Option<AgentColor>,
    pub prompt: Option<String>,
    /// Display only; never sent back by the webview (ADR 0010).
    pub file_path: Option<String>,
    pub memory: Option<AgentMemory>,
    pub scope: AgentScope,
    /// `None` for built-in agents, which have no file.
    pub config_ref: Option<ConfigRef>,
}

impl SubAgent {
    pub fn is_built_in(&self) -> bool {
        self.scope == AgentScope::BuiltIn
    }

    pub const KNOWN_TOOLS: [&'static str; 12] =
        ["Agent", "AskUserQuestion", "Bash", "Edit", "Glob", "Grep", "NotebookEdit", "Read", "TodoWrite", "WebFetch", "WebSearch", "Write"];

    pub const TOOL_CATEGORIES: [ToolCategory; 4] = [
        ToolCategory { id: "readonly", name: "Read-only tools", tools: &["Glob", "Grep", "Read", "WebFetch", "WebSearch"] },
        ToolCategory { id: "edit", name: "Edit tools", tools: &["Edit", "Write", "NotebookEdit"] },
        ToolCategory { id: "execution", name: "Execution tools", tools: &["Bash", "Agent"] },
        ToolCategory { id: "other", name: "Other tools", tools: &["AskUserQuestion", "TodoWrite"] },
    ];

    /// The four agents Claude Code ships (SubAgent.swift:76-110).
    pub fn built_in() -> Vec<SubAgent> {
        let agent = |id: &str, name: &str, icon: &str, description: &str, tools: &[&str]| SubAgent {
            id: id.into(),
            name: name.into(),
            icon: icon.into(),
            description: description.into(),
            tools: tools.iter().map(|t| t.to_string()).collect(),
            model: None,
            color: None,
            prompt: None,
            file_path: None,
            memory: None,
            scope: AgentScope::BuiltIn,
            config_ref: None,
        };
        vec![
            agent("explore", "Explore", "magnifyingglass", "Fast agent for codebase exploration — find files, search code, answer structural questions",
                  &["Glob", "Grep", "Read", "Bash"]),
            agent("plan", "Plan", "map", "Software architect for designing implementation plans and identifying critical files",
                  &["Glob", "Grep", "Read", "Bash"]),
            agent("bash", "Bash", "terminal", "Command execution specialist for running shell commands, git operations, and builds", &["Bash"]),
            agent("general", "General-purpose", "person.fill", "General-purpose agent for researching questions, searching code, and multi-step tasks",
                  &["Glob", "Grep", "Read", "Write", "Edit", "Bash"]),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(id: &str) -> SubAgent {
        SubAgent::built_in().into_iter().find(|a| a.id == id).unwrap()
    }

    // Ported from PoirotTests/Models/SubAgentTests.swift.
    #[test]
    fn built_in_has_four_agents() {
        assert_eq!(SubAgent::built_in().len(), 4);
    }
    #[test]
    fn built_in_has_expected_ids() {
        let ids: Vec<_> = SubAgent::built_in().into_iter().map(|a| a.id).collect();
        assert_eq!(ids, ["explore", "plan", "bash", "general"]);
    }
    #[test]
    fn built_in_all_have_non_empty_fields() {
        assert!(SubAgent::built_in().iter().all(|a| !a.name.is_empty() && !a.icon.is_empty() && !a.description.is_empty() && !a.tools.is_empty()));
    }
    #[test]
    fn built_in_explore_agent_has_expected_tools() {
        let tools = find("explore").tools;
        assert!(["Glob", "Grep", "Read", "Bash"].iter().all(|t| tools.iter().any(|x| x == t)));
    }
    #[test]
    fn built_in_bash_agent_has_only_bash_tool() {
        assert_eq!(find("bash").tools, ["Bash"]);
    }
    #[test]
    fn built_in_all_are_built_in_scope() {
        assert!(SubAgent::built_in().iter().all(|a| a.is_built_in() && a.scope == AgentScope::BuiltIn && a.file_path.is_none()));
    }
    #[test]
    fn known_tools_is_non_empty() {
        assert!(SubAgent::KNOWN_TOOLS.contains(&"Bash") && SubAgent::KNOWN_TOOLS.contains(&"Read"));
    }

    #[test]
    fn colors_parse_from_frontmatter_values() {
        assert_eq!(AgentColor::from_raw("teal"), Some(AgentColor::Teal));
        assert_eq!(AgentColor::from_raw("magenta"), None);
    }
}
