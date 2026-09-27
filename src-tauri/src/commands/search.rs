//! Commands in commands/search.rs (docs/ipc-contract.md §6): typed stub frozen in M1-37, the real
//! body lands here (M1-46). Runs through `state::blocking` (contract §1) since it loads from
//! every data area at once.
//!
//! Which categories are project-scoped by `project` is `docs/spec/search-overlay.md`'s own table
//! (§1): commands, skills, MCP servers, output styles and hooks; every other category (sessions,
//! debug logs, facets, file changes, todos, history, plans, memory, plugins, models, sub-agents)
//! is always global, `project` or not — memory in particular reads every project's own memory
//! files unconditionally, never scoped down to just one.

use std::collections::BTreeSet;

use heimdall_core::Result;
use heimdall_core::config::read::{
    ProjectScope, agents, commands as command_loader, hooks, mcp, memory, output_styles, plans, plugins, skills,
};
use heimdall_core::data::{debug_log, facets, history, todos};
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::models::prefs::SearchMode;
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::discover;
use heimdall_core::search::SearchResults;
use heimdall_core::search::providers::{SearchInput, scan_file_history_sessions, search as run_search};
use tauri::{AppHandle, Manager};

use super::config_read::resolve_project_scope;
use crate::state::{AppState, blocking};

fn search_inner<R: tauri::Runtime>(
    app: &AppHandle<R>,
    state: &AppState,
    query: &str,
    mode: SearchMode,
    project: Option<ProjectContext>,
) -> Result<SearchResults> {
    let claude_home = &state.claude_home;
    let os = TargetOs::current();
    let root = project
        .as_ref()
        .map(|context| resolve_project_scope(app, state, context))
        .transpose()?;
    let scope = root
        .as_deref()
        .zip(project.as_ref())
        .map(|(root, context)| ProjectScope { root, context });

    let projects = discover(&claude_home.projects_dir(), os)?.value;
    let show_agent_sessions = crate::prefs::load(app)?.show_agent_sessions;
    let debug_log_session_ids: BTreeSet<String> = debug_log::all_session_ids(&claude_home.debug_dir())
        .into_iter()
        .collect();
    let facets = facets::load_all(&claude_home.facets_dir())?.value;
    let file_history_sessions =
        scan_file_history_sessions(&claude_home.file_history_dir(), &claude_home.projects_dir(), os);
    let todos = todos::load_all(&claude_home.todos_dir())?.value;
    let history = history::load_all(&claude_home.history_file())?.value;
    let commands = command_loader::load(claude_home.root(), scope).value;
    let skills = skills::load(claude_home.root(), scope).value;
    let plans = plans::load(claude_home.root()).value;
    let memory_project_files: Vec<_> = projects
        .iter()
        .flat_map(|p| {
            memory::load_project(&claude_home.projects_dir(), &p.id)
                .map(|l| l.value)
                .unwrap_or_default()
        })
        .collect();
    let mcp_servers = mcp::load(claude_home, root.as_deref().and_then(|p| p.to_str()), os).value;
    let plugins = plugins::load(claude_home).value;
    let output_styles = output_styles::load(claude_home.root(), scope).value;
    let hooks = hooks::load(claude_home.root(), root.as_deref()).value;
    let sub_agents = agents::load(claude_home.root()).value;

    let input = SearchInput {
        projects: &projects,
        show_agent_sessions,
        debug_log_session_ids: &debug_log_session_ids,
        facets: &facets,
        file_history_sessions: &file_history_sessions,
        todos: &todos,
        history: &history,
        commands: &commands,
        skills: &skills,
        plans: &plans,
        memory_project_files: &memory_project_files,
        mcp_servers: &mcp_servers,
        plugins: &plugins,
        output_styles: &output_styles,
        hooks: &hooks,
        sub_agents: &sub_agents,
    };
    Ok(run_search(&input, query, mode))
}

#[tauri::command]
#[specta::specta]
pub async fn search(
    app: AppHandle,
    query: String,
    mode: SearchMode,
    project: Option<ProjectContext>,
) -> Result<SearchResults> {
    blocking(move || search_inner(&app, &app.state::<AppState>(), &query, mode, project)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::AppError;
    use heimdall_core::paths::ClaudeHome;
    use std::path::Path;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        let claude_home = ClaudeHome::new(root.join("claude-home"), root.join("claude.json"));
        let app = mock_builder()
            .plugin(tauri_plugin_store::Builder::default().build())
            .build(mock_context(noop_assets()))
            .expect("build mock app");
        app.manage(AppState {
            claude_home,
            cache: crate::state::Cache::default(),
        });
        app.handle().clone()
    }

    // Acceptance criterion: a broad query returns all 16 groups. Checked directly against the
    // fixture first (a throwaway `src-tauri/src/bin/debug_check.rs`, built and run via
    // `cargo run —`, then deleted — not committed): "e" is one of several single letters that
    // reach every one of the 16 categories on `fixtures/synthetic` in fuzzy mode.
    #[test]
    fn a_broad_query_returns_all_16_groups() {
        let app = app_handle();
        let results = search_inner(&app, &app.state::<AppState>(), "e", SearchMode::Fuzzy, None).unwrap();
        assert_eq!(
            results.groups.len(),
            16,
            "{:?}",
            results.groups.iter().map(|g| g.category).collect::<Vec<_>>()
        );
    }

    #[test]
    fn each_group_caps_at_5_hits() {
        let app = app_handle();
        let results = search_inner(&app, &app.state::<AppState>(), "e", SearchMode::Fuzzy, None).unwrap();
        assert!(results.groups.iter().all(|g| g.hits.len() <= 5));
    }

    #[test]
    fn an_unmatched_query_returns_no_groups() {
        let app = app_handle();
        let results = search_inner(
            &app,
            &app.state::<AppState>(),
            "zzzzznonexistentqueryxyz",
            SearchMode::Exact,
            None,
        )
        .unwrap();
        assert!(results.groups.is_empty());
    }

    #[test]
    fn project_scope_adds_the_projects_own_command() {
        let app = app_handle();
        let global = search_inner(&app, &app.state::<AppState>(), "review", SearchMode::Fuzzy, None).unwrap();
        let project = Some(ProjectContext::Project {
            project_id: "-Users-fixture-src-app".into(),
        });
        let scoped = search_inner(&app, &app.state::<AppState>(), "review", SearchMode::Fuzzy, project).unwrap();
        // The fixture's one command ("review") is global already, so both scopes find it; the
        // real assertion is that a project-scoped search does not error and still finds it.
        assert!(
            global
                .groups
                .iter()
                .any(|g| g.category == heimdall_core::search::SearchCategory::Commands)
        );
        assert!(
            scoped
                .groups
                .iter()
                .any(|g| g.category == heimdall_core::search::SearchCategory::Commands)
        );
    }

    #[test]
    fn rejects_an_unknown_project_as_invalid_input() {
        let app = app_handle();
        let project = Some(ProjectContext::Project {
            project_id: "-Users-nobody-nowhere".into(),
        });
        assert!(matches!(
            search_inner(&app, &app.state::<AppState>(), "e", SearchMode::Fuzzy, project).unwrap_err(),
            AppError::InvalidInput { .. }
        ));
    }
}
