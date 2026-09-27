//! Commands in commands/app.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37, real
//! bodies land here (M1-40). Every body runs through `state::blocking` (contract §1). The actual
//! counting/detection logic is pure heimdall-core (`app_shell::{sidebar_counts, cli_status}`),
//! testable there without any Tauri type; this file only resolves the AppHandle-dependent bits
//! (package identity, and a picked-folder project's path, which needs the prefs store).

use std::path::PathBuf;

use heimdall_core::app_shell::{AppInfo, CliStatus, SidebarCounts, cli_status, sidebar_counts};
use heimdall_core::config::read::ProjectScope;
use heimdall_core::models::config::scope::ProjectContext;
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::load_project;
use heimdall_core::{AppError, Result};
use tauri::{AppHandle, Manager};

use crate::state::{AppState, blocking};

/// Resolves a `ProjectContext` to the real project folder on disk (never `~/.claude/projects/<id>`,
/// which only holds transcripts — commands, skills, hooks etc. live in the project's own
/// `<root>/.claude/`). `None` (or an id/picked-folder that fails to resolve) means the global scope.
fn resolve_project_root(
    app: &AppHandle,
    claude_home: &heimdall_core::paths::ClaudeHome,
    context: Option<&ProjectContext>,
) -> Option<PathBuf> {
    match context {
        None => None,
        Some(ProjectContext::PickedFolder) => crate::prefs::load(app)
            .ok()
            .and_then(|p| p.config_project_path)
            .map(PathBuf::from),
        Some(ProjectContext::Project { project_id }) => {
            let dir = claude_home.projects_dir().join(project_id);
            load_project(&dir, TargetOs::current()).value.path.map(PathBuf::from)
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_app_info(app: AppHandle) -> Result<AppInfo> {
    blocking(move || {
        let state = app.state::<AppState>();
        let claude_home = &state.claude_home;
        Ok(AppInfo {
            version: app.package_info().version.to_string(),
            identifier: app.config().identifier.clone(),
            os: TargetOs::current(),
            arch: std::env::consts::ARCH.to_string(),
            claude_home_found: claude_home.root().is_dir(),
        })
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn get_sidebar_counts(app: AppHandle, project: Option<ProjectContext>) -> Result<SidebarCounts> {
    blocking(move || {
        let state = app.state::<AppState>();
        let claude_home = &state.claude_home;
        let root = resolve_project_root(&app, claude_home, project.as_ref());
        let scope = match (root.as_deref(), project.as_ref()) {
            (Some(root), Some(context)) => Some(ProjectScope { root, context }),
            _ => None,
        };
        sidebar_counts(claude_home, scope)
    })
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn detect_claude_cli(app: AppHandle) -> Result<CliStatus> {
    blocking(move || {
        let state = app.state::<AppState>();
        let claude_home = &state.claude_home;
        let home_dir = heimdall_core::paths::user_home().ok_or_else(|| AppError::not_found("no home folder"))?;
        let path_var = std::env::var("PATH").unwrap_or_default();
        let pathext = std::env::var("PATHEXT").unwrap_or_default();
        Ok(cli_status(
            claude_home,
            &home_dir,
            &path_var,
            &pathext,
            TargetOs::current(),
            &|p| p.exists(),
        ))
    })
    .await
}
