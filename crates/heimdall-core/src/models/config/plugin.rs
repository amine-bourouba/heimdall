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

/// The wire shape of a plugin (contract `PluginInfo`): `key` is [`ClaudePlugin::id`], `enabled` is
/// `is_enabled`, and the two disk-optional fields become `Option`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub key: String,
    pub name: String,
    pub author: String,
    pub version: Option<String>,
    pub installed_at: Option<String>,
    pub enabled: bool,
}
