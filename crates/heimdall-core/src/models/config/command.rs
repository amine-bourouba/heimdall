//! Slash commands (Models/Config/ClaudeCommand.swift).

use serde::Serialize;
use specta::Type;

use super::scope::ConfigScope;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeCommand {
    pub id: String,
    pub name: String,
    pub description: String,
    pub model: Option<String>,
    pub argument_hint: Option<String>,
    pub allowed_tools: Option<String>,
    pub output_style: Option<String>,
    pub body: String,
    /// Display only; never sent back by the webview (ADR 0010).
    pub file_path: String,
    pub scope: ConfigScope,
}
