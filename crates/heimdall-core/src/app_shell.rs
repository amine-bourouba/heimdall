//! Wire types with no single owning domain: app identity, CLI/update status, sidebar counts and
//! the menu-action event payload (docs/ipc-contract.md §5, §7). Implemented by M1-40 and M4-04.

use std::path::Path;

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::Result;
use crate::config::read::ProjectScope;
use crate::config::read::{agents, commands, hooks, mcp, output_styles, plans, plugins, skills};
use crate::data::{history, todos};
use crate::models::catalog::claude_models;
use crate::os_integration::cli_locate::find_claude_cli;
use crate::paths::{ClaudeHome, TargetOs};

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

/// One count per sidebar section (`AppState.computeSidebarCounts`, `App/AppState.swift:247-268`).
/// `project` scopes the per-project-aware loaders (commands, skills, output styles, hooks, MCP
/// servers) to that project's own `<root>/.claude/`; `None` is the global scope every other count
/// already uses regardless (todos, history, plans, memory, models and sub-agents have no
/// per-project half in the Swift reference app either).
pub fn sidebar_counts(claude_home: &ClaudeHome, project: Option<ProjectScope>) -> Result<SidebarCounts> {
    let home_root = claude_home.root();
    let project_root = project.map(|s| s.root);
    Ok(SidebarCounts {
        todos: todos::load_all(&claude_home.todos_dir())?
            .value
            .values()
            .map(|items| items.len() as u32)
            .sum(),
        history: history::entry_count(&claude_home.history_file())?,
        commands: commands::load(home_root, project).value.len() as u32,
        skills: skills::load(home_root, project).value.len() as u32,
        plans: plans::load(home_root).value.len() as u32,
        memory: total_memory_file_count(claude_home),
        mcp_servers: mcp::load(claude_home, project_root.and_then(Path::to_str), TargetOs::current())
            .value
            .len() as u32,
        models: claude_models::curated().len() as u32,
        hooks: hooks::load(home_root, project_root).value.len() as u32,
        sub_agents: agents::load(home_root).value.len() as u32,
        plugins: plugins::load(claude_home).value.len() as u32,
        output_styles: output_styles::load(home_root, project).value.len() as u32,
    })
}

/// Total memory files across every project's `~/.claude/projects/<id>/memory/` folder (the Swift reference app's
/// `ClaudeConfigLoader.totalMemoryFileCount`: every project, always, with no project scoping).
fn total_memory_file_count(claude_home: &ClaudeHome) -> u32 {
    let Ok(entries) = std::fs::read_dir(claude_home.projects_dir()) else {
        return 0;
    };
    entries
        .flatten()
        .map(|entry| crate::config::read::memory::folder_file_count(&entry.path().join("memory")))
        .sum()
}

/// `detect_claude_cli`: whether the `claude` binary is on `PATH` (or in one of ADR 0003's
/// fallback locations) and whether any project has ever been recorded (`OnboardingView.swift:281-297`
/// checked six hardcoded macOS paths and a directory listing; this reuses the cross-platform PATH
/// search M1-31 already built instead of the Swift reference app's OS-specific literal list).
pub fn cli_status(
    claude_home: &ClaudeHome,
    home_dir: &Path,
    path_var: &str,
    pathext: &str,
    os: TargetOs,
    exists: &dyn Fn(&Path) -> bool,
) -> CliStatus {
    let cli_path = find_claude_cli(path_var, pathext, home_dir, os, exists).map(|p| p.display().to_string());
    let sessions_exist = std::fs::read_dir(claude_home.projects_dir()).is_ok_and(|mut d| d.next().is_some());
    CliStatus {
        cli_path,
        sessions_exist,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::config::scope::ProjectContext;

    fn fixture_home() -> ClaudeHome {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/synthetic");
        ClaudeHome::new(root.join("claude-home"), root.join("claude.json"))
    }

    // Ported from the Swift reference's own test suite (App): `AppState.computeSidebarCounts` has no dedicated Swift test
    // suite (checked: no AppStateTests.swift exists), so this cross-checks the formula against
    // each loader called directly, plus concrete counts read off the fixture tree itself.
    #[test]
    fn sidebar_counts_matches_fixtures_synthetic() {
        let home = fixture_home();
        let counts = sidebar_counts(&home, None).unwrap();

        assert_eq!(counts.commands, commands::load(home.root(), None).value.len() as u32);
        assert_eq!(counts.skills, skills::load(home.root(), None).value.len() as u32);
        assert_eq!(
            counts.output_styles,
            output_styles::load(home.root(), None).value.len() as u32
        );
        assert_eq!(counts.plans, plans::load(home.root()).value.len() as u32);
        assert_eq!(counts.sub_agents, agents::load(home.root()).value.len() as u32);
        assert_eq!(counts.plugins, plugins::load(&home).value.len() as u32);
        assert_eq!(
            counts.mcp_servers,
            mcp::load(&home, None, TargetOs::current()).value.len() as u32
        );
        assert_eq!(counts.hooks, hooks::load(home.root(), None).value.len() as u32);
        assert_eq!(counts.history, history::entry_count(&home.history_file()).unwrap());
        let todos_total: u32 = todos::load_all(&home.todos_dir())
            .unwrap()
            .value
            .values()
            .map(|items| items.len() as u32)
            .sum();
        assert_eq!(counts.todos, todos_total);

        // Concrete numbers read off the fixture tree (fixtures/synthetic/README.md), so a change
        // to either the fixture or a loader's own behaviour is caught, not just a mismatch between
        // this function and its own building blocks.
        assert_eq!(counts.commands, 1); // commands/review.md
        assert_eq!(counts.skills, 1); // skills/pdf-tools
        assert_eq!(counts.output_styles, 1); // output-styles/concise.md; README.md excluded
        assert_eq!(counts.plans, 1); // plans/refactor-auth-module.md
        assert_eq!(counts.sub_agents, 5); // 4 built-in + agents/reviewer.md
        assert_eq!(counts.plugins, 2); // plugins/installed_plugins.json
        assert_eq!(counts.mcp_servers, 3); // claude.json's top-level github + sentry, plus the project-scoped local-db
        assert_eq!(counts.models, claude_models::curated().len() as u32);
        assert!(counts.todos >= 1 && counts.history >= 1);
    }

    #[test]
    fn sidebar_counts_scoped_to_a_project_adds_that_projects_own_items() {
        let home = fixture_home();
        let project_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/synthetic/project-repo");
        let context = ProjectContext::PickedFolder;
        let scope = ProjectScope {
            root: &project_root,
            context: &context,
        };
        let global = sidebar_counts(&home, None).unwrap();
        let scoped = sidebar_counts(&home, Some(scope)).unwrap();

        // project-repo/.claude/ adds one command and one output style on top of the global ones;
        // every other count (todos, history, plans, memory, models, sub-agents) is unaffected by scope.
        assert_eq!(scoped.commands, global.commands + 1);
        assert_eq!(scoped.output_styles, global.output_styles + 1);
        assert_eq!(
            (
                scoped.todos,
                scoped.history,
                scoped.plans,
                scoped.sub_agents,
                scoped.models
            ),
            (
                global.todos,
                global.history,
                global.plans,
                global.sub_agents,
                global.models
            )
        );
    }

    #[test]
    fn cli_status_reports_the_found_path_via_path_search() {
        let home = fixture_home();
        let status = cli_status(
            &home,
            Path::new("/home/fixture"),
            "/fake/bin",
            "",
            TargetOs::Linux,
            &|p| p == Path::new("/fake/bin/claude"),
        );
        // A literal, not `Path::new("/fake/bin").join("claude")`: that join uses the *host's*
        // separator, which only happens to be "/" when this test itself runs on a non-Windows
        // host — exactly the cross-host CI mismatch a real ubuntu-22.04/macos-latest run caught
        // in cli_locate.rs's own tests (M1-59's gate review). `os` here is hardcoded to Linux, so
        // the expected separator is always "/", regardless of what host runs this test.
        assert_eq!(status.cli_path, Some("/fake/bin/claude".to_string()));
        assert!(status.sessions_exist); // fixtures/synthetic/claude-home/projects has one project
    }

    #[test]
    fn cli_status_reports_none_when_the_cli_is_nowhere_found() {
        let home = fixture_home();
        let status = cli_status(&home, Path::new("/home/fixture"), "", "", TargetOs::Linux, &|_| false);
        assert_eq!(status.cli_path, None);
    }

    #[test]
    fn cli_status_reports_no_sessions_for_an_empty_projects_dir() {
        let dir = tempfile::tempdir().unwrap();
        let home = ClaudeHome::new(dir.path(), dir.path().join(".claude.json"));
        let status = cli_status(&home, Path::new("/home/fixture"), "", "", TargetOs::Linux, &|_| false);
        assert!(!status.sessions_exist);
    }
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
