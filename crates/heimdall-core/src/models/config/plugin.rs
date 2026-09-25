//! Installed plugins (Models/Config/ClaudePlugin.swift).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ClaudePlugin {
    /// `<name>@<author>`, the key used in `enabledPlugins`.
    pub id: String,
    pub name: String,
    pub author: String,
    pub version: String,
    pub installed_at: String,
    pub is_enabled: bool,
}
