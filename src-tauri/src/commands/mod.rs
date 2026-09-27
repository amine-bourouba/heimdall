//! IPC command registration through tauri-specta (ADR 0004).
//!
//! Every command in docs/ipc-contract.md §6 is registered here as a typed stub (M1-37); each
//! returns `not implemented` until its owning task lands. `src/ipc/bindings.ts` is generated from
//! this builder by `scripts/check-bindings.mjs`.

pub mod analytics;
pub mod app;
pub mod config_read;
pub mod config_write;
pub mod export;
pub mod history;
pub mod launch;
pub mod mcp;
pub mod prefs;
pub mod search;
pub mod session_data;
pub mod sessions;
pub mod tray;
pub mod update;

pub fn builder() -> tauri_specta::Builder<tauri::Wry> {
    tauri_specta::Builder::<tauri::Wry>::new()
        // Every u64/i64 field on the wire (token and message counts, timestamps-as-ms elsewhere)
        // is a small counter that never approaches 2^53; Specta's BigInt-precision guard would
        // otherwise refuse the export outright (ponytail: ceiling is 2^53, not worth a
        // narrower-int pass across every model for headroom this domain never needs).
        .dangerously_cast_bigints_to_number()
        .commands(tauri_specta::collect_commands![
            app::get_app_info,
            app::get_sidebar_counts,
            app::detect_claude_cli,
            prefs::get_prefs,
            prefs::update_prefs,
            prefs::add_memory_folder,
            prefs::remove_memory_folder,
            prefs::pick_config_project,
            sessions::list_projects,
            sessions::list_sessions,
            sessions::get_session,
            sessions::get_session_messages,
            sessions::list_recent_sessions,
            sessions::delete_session,
            sessions::delete_project,
            session_data::get_session_todos,
            session_data::list_all_todos,
            session_data::get_file_history,
            session_data::get_file_version,
            session_data::diff_file_versions,
            session_data::diff_lines,
            session_data::get_debug_log_page,
            session_data::get_session_facets,
            history::list_history,
            history::delete_history_entry,
            history::clear_history_older_than,
            config_read::list_config_items,
            config_read::get_config_item,
            config_read::list_hooks,
            config_read::list_plugins,
            config_read::list_models,
            config_read::list_memory_sources,
            config_read::list_memory_files,
            config_read::get_memory_file,
            mcp::list_mcp_servers,
            mcp::get_mcp_statuses,
            mcp::get_mcp_catalog,
            mcp::add_mcp_server,
            mcp::update_mcp_server,
            mcp::remove_mcp_server,
            config_write::set_plugin_enabled,
            config_write::remove_plugin,
            config_write::set_default_model,
            config_write::set_project_model,
            config_write::save_hook,
            config_write::delete_hook,
            config_write::import_hooks,
            config_write::export_hooks,
            config_write::save_agent,
            config_write::import_agent,
            config_write::export_agent,
            config_write::create_config_template,
            config_write::delete_config_item,
            analytics::get_stats,
            tray::get_tray_snapshot,
            tray::open_in_main_window,
            search::search,
            export::export_session,
            export::export_edit_patch,
            export::export_analytics_csv,
            export::save_png,
            launch::open_external_url,
            launch::open_in_editor,
            launch::open_terminal,
            launch::reveal_in_file_manager,
            update::check_for_update,
            update::install_update,
        ])
        .events(tauri_specta::collect_events![
            crate::events::DataChanged,
            crate::events::WatcherStatus,
            crate::events::MenuAction,
            crate::events::Navigate,
        ])
}

/// Every command name in docs/ipc-contract.md §6, in the same order as the table, for
/// `commands::tests::registered_commands_match_the_contract_table` to compare against.
#[cfg(test)]
const CONTRACT_COMMAND_NAMES: &[&str] = &[
    "get_app_info",
    "get_sidebar_counts",
    "detect_claude_cli",
    "get_prefs",
    "update_prefs",
    "add_memory_folder",
    "remove_memory_folder",
    "pick_config_project",
    "list_projects",
    "list_sessions",
    "get_session",
    "get_session_messages",
    "list_recent_sessions",
    "delete_session",
    "delete_project",
    "get_session_todos",
    "list_all_todos",
    "get_file_history",
    "get_file_version",
    "diff_file_versions",
    "diff_lines",
    "get_debug_log_page",
    "get_session_facets",
    "list_history",
    "delete_history_entry",
    "clear_history_older_than",
    "list_config_items",
    "get_config_item",
    "list_hooks",
    "list_plugins",
    "list_models",
    "list_memory_sources",
    "list_memory_files",
    "get_memory_file",
    "list_mcp_servers",
    "get_mcp_statuses",
    "get_mcp_catalog",
    "add_mcp_server",
    "update_mcp_server",
    "remove_mcp_server",
    "set_plugin_enabled",
    "remove_plugin",
    "set_default_model",
    "set_project_model",
    "save_hook",
    "delete_hook",
    "import_hooks",
    "export_hooks",
    "save_agent",
    "import_agent",
    "export_agent",
    "create_config_template",
    "delete_config_item",
    "get_stats",
    "get_tray_snapshot",
    "open_in_main_window",
    "search",
    "export_session",
    "export_edit_patch",
    "export_analytics_csv",
    "save_png",
    "open_external_url",
    "open_in_editor",
    "open_terminal",
    "reveal_in_file_manager",
    "check_for_update",
    "install_update",
];

#[cfg(test)]
mod tests {
    use super::*;

    /// tauri-specta names the Tauri `invoke` channel after the literal Rust command name, quoted,
    /// inside each generated wrapper function — so every registered command leaves this exact
    /// trace in the export regardless of exporter formatting details (`Commands` has no public
    /// name list to compare against directly).
    fn generated_bindings() -> String {
        let dir = std::env::temp_dir().join(format!("heimdall-bindings-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join("bindings.ts");
        builder()
            .export(specta_typescript::Typescript::default(), &path)
            .expect("export bindings");
        let text = std::fs::read_to_string(&path).expect("read generated bindings");
        std::fs::remove_dir_all(&dir).ok();
        text
    }

    #[test]
    fn registered_commands_match_the_contract_table() {
        let text = generated_bindings();
        let missing: Vec<&str> = CONTRACT_COMMAND_NAMES
            .iter()
            .copied()
            .filter(|name| !text.contains(&format!("\"{name}\"")))
            .collect();
        assert!(
            missing.is_empty(),
            "commands missing from the generated bindings: {missing:?}"
        );
    }

    /// `build.rs` cannot depend on this crate's own compiled output (it runs *to produce* that
    /// output), so its `COMMANDS` list is a separate literal copy, not a shared `const` — this
    /// test is what actually keeps the two from drifting apart (M1-49's acceptance criterion).
    #[test]
    fn build_rs_lists_exactly_the_contract_commands() {
        let build_rs = include_str!("../../build.rs");
        let list_text = build_rs
            .split("const COMMANDS: &[&str] = &[")
            .nth(1)
            .expect("build.rs must declare COMMANDS")
            .split("];")
            .next()
            .expect("COMMANDS list must be closed with `];`");
        let mut from_build_rs: Vec<&str> = list_text
            .lines()
            .filter_map(|line| line.trim().trim_end_matches(',').strip_prefix('"'))
            .filter_map(|s| s.strip_suffix('"'))
            .collect();
        from_build_rs.sort_unstable();
        let mut expected: Vec<&str> = CONTRACT_COMMAND_NAMES.to_vec();
        expected.sort_unstable();
        assert_eq!(from_build_rs, expected);
    }
}
