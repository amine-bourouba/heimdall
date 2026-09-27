//! Declares every app command to Tauri's Access Control List (`AppManifest::commands`), which
//! autogenerates an `allow-<command>`/`deny-<command>` permission pair per command — the
//! permission identifiers `capabilities/main.json`/`tray.json` (M1-48) grant by name. This list
//! must exactly match `docs/ipc-contract.md` §6 (`commands::tests::build_rs_lists_exactly_the_contract_commands`
//! compares it against `commands::CONTRACT_COMMAND_NAMES`, the same list M1-37 froze); a build
//! script cannot depend on its own crate's compiled output, so this is its own literal copy, not
//! a shared `const`.

const COMMANDS: &[&str] = &[
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

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMMANDS)),
    )
    .expect("failed to run build script");
}
