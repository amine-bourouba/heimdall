//! Config scope (Models/Config/ConfigScope.swift): `~/.claude/` or `<project>/.claude/`.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigScope {
    Global,
    Project,
}

/// The kinds of file-backed config items (contract `ConfigKind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ConfigKind {
    Command,
    Skill,
    Plan,
    OutputStyle,
    Agent,
}

/// Which project's `.claude/` folder a request is about (contract `ProjectContext`). `PickedFolder`
/// is `Prefs.configProjectPath`, set only by the folder picker.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ProjectContext {
    #[serde(rename_all = "camelCase")]
    Project { project_id: String },
    PickedFolder,
}

/// How the webview names a config item (contract `ConfigRef`): never a path. `project` is `None`
/// for global items. The slug is a file stem (a folder name for skills) with no separator, and is
/// resolved only under its scope's root (`config::read::item_path`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ConfigRef {
    pub kind: ConfigKind,
    pub project: Option<ProjectContext>,
    pub slug: String,
}
