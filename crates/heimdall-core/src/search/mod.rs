//! Universal search: the matcher (M1-29) and the per-category providers (M1-30).
//!
//! The types below are the search half of the IPC contract (docs/ipc-contract.md §5); `search.rs`
//! (M1-46) is the only place that will call `providers::search` from a Tauri command.

use serde::Serialize;
use specta::Type;

use crate::models::config::scope::ConfigRef;

pub mod matcher;
pub mod providers;

pub use crate::models::prefs::SearchMode;

/// The 16 categories, in the fixed order they are shown (SearchOverlayView.swift:5-21).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum SearchCategory {
    Sessions,
    DebugLogs,
    Facets,
    FileChanges,
    Todos,
    History,
    Commands,
    Skills,
    Plans,
    Memory,
    McpServers,
    Plugins,
    OutputStyles,
    Hooks,
    Models,
    SubAgents,
}

impl SearchCategory {
    pub const ALL: [Self; 16] = [
        Self::Sessions, Self::DebugLogs, Self::Facets, Self::FileChanges, Self::Todos, Self::History, Self::Commands,
        Self::Skills, Self::Plans, Self::Memory, Self::McpServers, Self::Plugins, Self::OutputStyles, Self::Hooks,
        Self::Models, Self::SubAgents,
    ];
}

/// The 14 navigable screens (`NavigationItem.allItems`), used by `navigateTo` and `openDetail`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NavScreen {
    Sessions,
    Todos,
    Commands,
    Skills,
    Plans,
    Memory,
    Hooks,
    McpServers,
    Plugins,
    OutputStyles,
    Models,
    SubAgents,
    Analytics,
    History,
}

/// Locates one memory file for `openDetail`: the source (a project's memory folder; search never
/// covers custom folders, §10) and the file within it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct MemoryFileLocator {
    pub source_id: String,
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum SearchAction {
    #[serde(rename_all = "camelCase")]
    OpenSession { project_id: String, session_id: String },
    #[serde(rename_all = "camelCase")]
    OpenDebugLog { project_id: String, session_id: String },
    NavigateTo { screen: NavScreen },
    #[serde(rename_all = "camelCase")]
    OpenDetail {
        screen: NavScreen,
        #[serde(rename = "ref")]
        config_ref: Option<ConfigRef>,
        memory_file: Option<MemoryFileLocator>,
    },
}

/// One result row. `highlights` always comes from matching `title` against the query directly
/// (never re-derived from whichever field actually produced the score, per
/// docs/spec/search-overlay.md §2.4), so it can be empty even for a hit that scored on another field.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub title: String,
    pub subtitle: Option<String>,
    pub highlights: Vec<(u32, u32)>,
    pub action: SearchAction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchGroup {
    pub category: SearchCategory,
    pub hits: Vec<SearchHit>,
}

/// At most 5 hits per group; a category with no matches is left out entirely.
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchResults {
    pub groups: Vec<SearchGroup>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_and_screen_names_serialise_as_the_contract_spells_them() {
        assert_eq!(serde_json::to_value(SearchCategory::McpServers).unwrap(), "mcpServers");
        assert_eq!(serde_json::to_value(SearchCategory::DebugLogs).unwrap(), "debugLogs");
        assert_eq!(serde_json::to_value(NavScreen::SubAgents).unwrap(), "subAgents");
        assert_eq!(serde_json::to_value(NavScreen::OutputStyles).unwrap(), "outputStyles");
    }

    #[test]
    fn open_detail_serialises_ref_without_the_rust_keyword_escape() {
        let action = SearchAction::OpenDetail { screen: NavScreen::Plans, config_ref: None, memory_file: None };
        let json = serde_json::to_value(action).unwrap();
        assert_eq!((json["kind"].as_str(), json.get("ref")), (Some("openDetail"), Some(&serde_json::Value::Null)));
    }
}
