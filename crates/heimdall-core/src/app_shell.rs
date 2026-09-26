//! Wire types with no single owning domain: app identity, CLI/update status, sidebar counts and
//! the menu-action event payload (docs/ipc-contract.md §5, §7). Implemented by M1-40 and M4-04.

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::paths::TargetOs;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub identifier: String,
    pub os: TargetOs,
    pub arch: String,
    pub claude_home_found: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct CliStatus {
    pub cli_path: Option<String>,
    pub sessions_exist: bool,
}

/// One count per sidebar section (contract `SidebarCounts`, a `Record` with these exact keys).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SidebarCounts {
    pub todos: u32,
    pub history: u32,
    pub commands: u32,
    pub skills: u32,
    pub plans: u32,
    pub memory: u32,
    pub mcp_servers: u32,
    pub models: u32,
    pub hooks: u32,
    pub sub_agents: u32,
    pub plugins: u32,
    pub output_styles: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatus {
    pub available: bool,
    pub current_version: String,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub notes_url: Option<String>,
}

/// A menu command routed through the `menu-action` event (contract `AppAction`, ADR 0006).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum AppAction {
    About,
    CheckUpdates,
    Settings,
    Help,
    Shortcuts,
    Search,
    Back,
    Forward,
    ToolFilter,
    FontIncrease,
    FontDecrease,
    FontReset,
}
