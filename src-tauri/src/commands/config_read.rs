//! Commands in commands/config_read.rs (docs/ipc-contract.md §6): typed stubs frozen in M1-37,
//! real bodies land here (M1-44). Every body runs through `state::blocking` (contract §1).
//!
//! `get_config_item`'s contract argument name `ref` is spelled `config_ref` here: `ref` is a
//! reserved word, so it cannot be a Rust parameter name.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use heimdall_core::config::read::memory::{self, MemorySource};
use heimdall_core::config::read::models::{models_view, ModelsView};
use heimdall_core::config::read::{agents, commands as command_loader, hooks, output_styles, plans, plugins, settings, skills, ProjectScope};
use heimdall_core::models::config::command::ClaudeCommand;
use heimdall_core::models::config::hook::HookEntry;
use heimdall_core::models::config::memory_file::{MemoryFile, MemoryFileDetail, MemoryFileSummary};
use heimdall_core::models::config::output_style::OutputStyle;
use heimdall_core::models::config::plan::Plan;
use heimdall_core::models::config::plugin::PluginInfo;
use heimdall_core::models::config::scope::{
    ConfigItemDetail, ConfigItemScope, ConfigItemSummary, ConfigKind, ConfigRef, ConfigScope, ProjectContext,
};
use heimdall_core::models::config::skill::ClaudeSkill;
use heimdall_core::models::config::sub_agent::{AgentScope, SubAgent};
use heimdall_core::paths::TargetOs;
use heimdall_core::projects::discovery::{discover, load_project};
use heimdall_core::{AppError, Loaded, Result};
use tauri::{AppHandle, Manager};

use crate::state::{blocking, AppState};

fn config_item_scope(scope: ConfigScope) -> ConfigItemScope {
    match scope {
        ConfigScope::Global => ConfigItemScope::Global,
        ConfigScope::Project => ConfigItemScope::Project,
    }
}

fn agent_item_scope(scope: AgentScope) -> ConfigItemScope {
    match scope {
        AgentScope::BuiltIn => ConfigItemScope::BuiltIn,
        AgentScope::Global => ConfigItemScope::Global,
    }
}

fn detail(summary: ConfigItemSummary, body: String, display_path: String) -> ConfigItemDetail {
    ConfigItemDetail { config_ref: summary.config_ref, name: summary.name, description: summary.description, scope: summary.scope, meta: summary.meta, body, display_path }
}

fn command_summary(c: &ClaudeCommand) -> ConfigItemSummary {
    let mut meta = BTreeMap::new();
    if let Some(m) = &c.model {
        meta.insert("model".into(), m.clone());
    }
    if let Some(h) = &c.argument_hint {
        meta.insert("argumentHint".into(), h.clone());
    }
    if let Some(t) = &c.allowed_tools {
        meta.insert("allowedTools".into(), t.clone());
    }
    if let Some(o) = &c.output_style {
        meta.insert("outputStyle".into(), o.clone());
    }
    ConfigItemSummary { config_ref: c.config_ref.clone(), name: c.name.clone(), description: c.description.clone(), scope: config_item_scope(c.scope), meta }
}

fn skill_summary(s: &ClaudeSkill) -> ConfigItemSummary {
    let mut meta = BTreeMap::new();
    if let Some(m) = &s.model {
        meta.insert("model".into(), m.clone());
    }
    if let Some(t) = &s.allowed_tools {
        meta.insert("allowedTools".into(), t.clone());
    }
    ConfigItemSummary { config_ref: s.config_ref.clone(), name: s.name.clone(), description: s.description.clone(), scope: config_item_scope(s.scope), meta }
}

fn plan_summary(p: &Plan) -> ConfigItemSummary {
    ConfigItemSummary { config_ref: p.config_ref.clone(), name: p.name.clone(), description: String::new(), scope: ConfigItemScope::Global, meta: BTreeMap::new() }
}

fn output_style_summary(o: &OutputStyle) -> ConfigItemSummary {
    ConfigItemSummary { config_ref: o.config_ref.clone(), name: o.name.clone(), description: o.description.clone(), scope: config_item_scope(o.scope), meta: BTreeMap::new() }
}

fn agent_summary(a: &SubAgent) -> ConfigItemSummary {
    let mut meta = BTreeMap::new();
    if let Some(m) = &a.model {
        meta.insert("model".into(), m.clone());
    }
    if let Some(c) = a.color {
        meta.insert("color".into(), format!("{c:?}"));
    }
    if !a.tools.is_empty() {
        meta.insert("tools".into(), a.tools.join(","));
    }
    let config_ref = a.config_ref.clone().unwrap_or_else(|| ConfigRef { kind: ConfigKind::Agent, project: None, slug: a.id.clone() });
    ConfigItemSummary { config_ref, name: a.name.clone(), description: a.description.clone(), scope: agent_item_scope(a.scope), meta }
}

/// Validates a project scope and resolves it to the project's real folder on disk (never
/// `~/.claude/projects/<id>`, which only holds transcripts) — `Project` must be a known project
/// id, `PickedFolder` must have an actual folder behind it (`pick_config_project`); anything else
/// is `invalidInput` (M1-44 acceptance criterion), not `notFound`, since it is the scope
/// descriptor itself that is malformed, not a specific item that happens to be missing.
fn resolve_project_scope<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, context: &ProjectContext) -> Result<PathBuf> {
    match context {
        ProjectContext::PickedFolder => crate::prefs::load(app)?
            .config_project_path
            .map(PathBuf::from)
            .ok_or_else(|| AppError::invalid_input("no folder has been picked via pick_config_project")),
        ProjectContext::Project { project_id } => {
            let name = heimdall_core::paths::plain_file_name(project_id).map_err(|_| AppError::invalid_input(format!("not a valid project id: {project_id}")))?;
            let dir = state.claude_home.projects_dir().join(name);
            if !dir.is_dir() {
                return Err(AppError::invalid_input(format!("unknown project: {project_id}")));
            }
            load_project(&dir, TargetOs::current()).value.path.map(PathBuf::from).ok_or_else(|| AppError::invalid_input(format!("project {project_id} has no resolved path")))
        }
    }
}

fn list_config_items_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, kind: ConfigKind, project: Option<ProjectContext>) -> Result<Loaded<Vec<ConfigItemSummary>>> {
    let claude_home = &state.claude_home;
    let root = project.as_ref().map(|context| resolve_project_scope(app, state, context)).transpose()?;
    let scope = root.as_deref().zip(project.as_ref()).map(|(root, context)| ProjectScope { root, context });

    Ok(match kind {
        ConfigKind::Command => {
            let loaded = command_loader::load(claude_home.root(), scope);
            Loaded::new(loaded.value.iter().map(command_summary).collect(), loaded.skipped)
        }
        ConfigKind::Skill => {
            let loaded = skills::load(claude_home.root(), scope);
            Loaded::new(loaded.value.iter().map(skill_summary).collect(), loaded.skipped)
        }
        // Plans and agents have no per-project half in Poirot either (M1-24); the project scope is
        // still validated above, but has nothing further to apply here.
        ConfigKind::Plan => {
            let loaded = plans::load(claude_home.root());
            Loaded::new(loaded.value.iter().map(plan_summary).collect(), loaded.skipped)
        }
        ConfigKind::OutputStyle => {
            let loaded = output_styles::load(claude_home.root(), scope);
            Loaded::new(loaded.value.iter().map(output_style_summary).collect(), loaded.skipped)
        }
        ConfigKind::Agent => {
            let loaded = agents::load(claude_home.root());
            Loaded::new(loaded.value.iter().map(agent_summary).collect(), loaded.skipped)
        }
    })
}

#[tauri::command]
#[specta::specta]
pub async fn list_config_items(app: AppHandle, kind: ConfigKind, project: Option<ProjectContext>) -> Result<Loaded<Vec<ConfigItemSummary>>> {
    blocking(move || list_config_items_inner(&app, &app.state::<AppState>(), kind, project)).await
}

fn get_config_item_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, config_ref: ConfigRef) -> Result<ConfigItemDetail> {
    let claude_home = &state.claude_home;
    let root = config_ref.project.as_ref().map(|context| resolve_project_scope(app, state, context)).transpose()?;
    let scope = root.as_deref().zip(config_ref.project.as_ref()).map(|(root, context)| ProjectScope { root, context });
    let not_found = || AppError::not_found(format!("no {:?} named {}", config_ref.kind, config_ref.slug));

    match config_ref.kind {
        ConfigKind::Command => {
            let item = command_loader::load(claude_home.root(), scope).value.into_iter().find(|c| c.config_ref == config_ref).ok_or_else(not_found)?;
            Ok(detail(command_summary(&item), item.body, item.file_path))
        }
        ConfigKind::Skill => {
            let item = skills::load(claude_home.root(), scope).value.into_iter().find(|s| s.config_ref == config_ref).ok_or_else(not_found)?;
            Ok(detail(skill_summary(&item), item.body, item.file_path))
        }
        ConfigKind::Plan => {
            let item = plans::load(claude_home.root()).value.into_iter().find(|p| p.config_ref == config_ref).ok_or_else(not_found)?;
            let display_path = item.file.display().to_string();
            Ok(detail(plan_summary(&item), item.content, display_path))
        }
        ConfigKind::OutputStyle => {
            let item = output_styles::load(claude_home.root(), scope).value.into_iter().find(|o| o.config_ref == config_ref).ok_or_else(not_found)?;
            Ok(detail(output_style_summary(&item), item.body, item.file_path))
        }
        ConfigKind::Agent => {
            let item = agents::load(claude_home.root())
                .value
                .into_iter()
                .find(|a| a.config_ref.as_ref() == Some(&config_ref) || (a.config_ref.is_none() && a.id == config_ref.slug))
                .ok_or_else(not_found)?;
            let display_path = item.file_path.clone().unwrap_or_default();
            let body = item.prompt.clone().unwrap_or_default();
            Ok(detail(agent_summary(&item), body, display_path))
        }
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_config_item(app: AppHandle, config_ref: ConfigRef) -> Result<ConfigItemDetail> {
    blocking(move || get_config_item_inner(&app, &app.state::<AppState>(), config_ref)).await
}

fn list_hooks_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, project: Option<ProjectContext>) -> Result<Loaded<Vec<HookEntry>>> {
    let root = project.as_ref().map(|context| resolve_project_scope(app, state, context)).transpose()?;
    Ok(hooks::load(state.claude_home.root(), root.as_deref()))
}

#[tauri::command]
#[specta::specta]
pub async fn list_hooks(app: AppHandle, project: Option<ProjectContext>) -> Result<Loaded<Vec<HookEntry>>> {
    blocking(move || list_hooks_inner(&app, &app.state::<AppState>(), project)).await
}

fn plugin_info(p: heimdall_core::models::config::plugin::ClaudePlugin) -> PluginInfo {
    PluginInfo {
        key: p.id,
        name: p.name,
        author: p.author,
        version: (!p.version.is_empty()).then_some(p.version),
        installed_at: (!p.installed_at.is_empty()).then_some(p.installed_at),
        enabled: p.is_enabled,
    }
}

fn list_plugins_inner(state: &AppState) -> Result<Vec<PluginInfo>> {
    Ok(plugins::load(&state.claude_home).value.into_iter().map(plugin_info).collect())
}

#[tauri::command]
#[specta::specta]
pub async fn list_plugins(app: AppHandle) -> Result<Vec<PluginInfo>> {
    blocking(move || list_plugins_inner(&app.state::<AppState>())).await
}

fn list_models_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, project: Option<ProjectContext>) -> Result<ModelsView> {
    let claude_home = &state.claude_home;
    let discovered: Vec<String> = discover(&claude_home.projects_dir(), TargetOs::current())?
        .value
        .iter()
        .flat_map(|p| p.sessions.iter().filter_map(|s| s.model.clone()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    let settings_model = settings::load(&claude_home.settings_file())?.and_then(|s| s.model);
    let project_model = match &project {
        None => None,
        Some(context) => settings::project_model(&resolve_project_scope(app, state, context)?),
    };
    Ok(models_view(&discovered, settings_model.as_deref(), project_model.as_deref()))
}

#[tauri::command]
#[specta::specta]
pub async fn list_models(app: AppHandle, project: Option<ProjectContext>) -> Result<ModelsView> {
    blocking(move || list_models_inner(&app, &app.state::<AppState>(), project)).await
}

fn list_memory_sources_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState) -> Result<Vec<MemorySource>> {
    let folders = crate::prefs::load(app)?.custom_memory_folders;
    Ok(memory::sources(&state.claude_home.projects_dir(), &folders))
}

#[tauri::command]
#[specta::specta]
pub async fn list_memory_sources(app: AppHandle) -> Result<Vec<MemorySource>> {
    blocking(move || list_memory_sources_inner(&app, &app.state::<AppState>())).await
}

/// Every memory file of the source `sourceId` names — `project:<id>` or `folder:<index>` into
/// `Prefs.customMemoryFolders` (contract `MemorySourceId`).
fn memory_files_for_source<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, source_id: &str) -> Result<Vec<MemoryFile>> {
    if let Some(project_id) = source_id.strip_prefix("project:") {
        Ok(memory::load_project(&state.claude_home.projects_dir(), project_id)?.value)
    } else if let Some(index) = source_id.strip_prefix("folder:").and_then(|n| n.parse::<usize>().ok()) {
        let folders = crate::prefs::load(app)?.custom_memory_folders;
        let folder = folders.get(index).ok_or_else(|| AppError::not_found(format!("no memory source at {source_id}")))?;
        Ok(memory::load_folder(Path::new(folder)).value)
    } else {
        Err(AppError::invalid_input(format!("not a memory source id: {source_id}")))
    }
}

fn list_memory_files_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, source_id: &str) -> Result<Vec<MemoryFileSummary>> {
    Ok(memory_files_for_source(app, state, source_id)?
        .into_iter()
        .map(|f| {
            let is_main = f.is_main();
            MemoryFileSummary { id: f.id, source_id: source_id.to_string(), name: f.name, filename: f.filename, is_main }
        })
        .collect())
}

#[tauri::command]
#[specta::specta]
pub async fn list_memory_files(app: AppHandle, source_id: String) -> Result<Vec<MemoryFileSummary>> {
    blocking(move || list_memory_files_inner(&app, &app.state::<AppState>(), &source_id)).await
}

fn get_memory_file_inner<R: tauri::Runtime>(app: &AppHandle<R>, state: &AppState, source_id: &str, file_id: &str) -> Result<MemoryFileDetail> {
    let file = memory_files_for_source(app, state, source_id)?
        .into_iter()
        .find(|f| f.id == file_id)
        .ok_or_else(|| AppError::not_found(format!("no memory file {file_id}")))?;
    let display_path = file.file.display().to_string();
    let is_main = file.is_main();
    Ok(MemoryFileDetail { id: file.id, source_id: source_id.to_string(), name: file.name, filename: file.filename, is_main, content: file.content, display_path })
}

#[tauri::command]
#[specta::specta]
pub async fn get_memory_file(app: AppHandle, source_id: String, file_id: String) -> Result<MemoryFileDetail> {
    blocking(move || get_memory_file_inner(&app, &app.state::<AppState>(), &source_id, &file_id)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use heimdall_core::paths::ClaudeHome;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    const PROJECT: &str = "-Users-fixture-src-app";

    fn app() -> tauri::App<tauri::test::MockRuntime> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/synthetic");
        let claude_home = ClaudeHome::new(root.join("claude-home"), root.join("claude.json"));
        let app = mock_builder().plugin(tauri_plugin_store::Builder::default().build()).build(mock_context(noop_assets())).expect("build mock app");
        app.manage(AppState { claude_home, cache: crate::state::Cache::default() });
        app
    }

    fn app_handle() -> AppHandle<tauri::test::MockRuntime> {
        app().handle().clone()
    }

    #[test]
    fn list_config_items_covers_each_kind_from_the_fixture() {
        let app = app_handle();
        let state = app.state::<AppState>();
        for (kind, expected_slug) in [
            (ConfigKind::Command, "review"),
            (ConfigKind::Skill, "pdf-tools"),
            (ConfigKind::Plan, "refactor-auth-module"),
            (ConfigKind::OutputStyle, "concise"),
        ] {
            let loaded = list_config_items_inner(&app, &state, kind, None).unwrap();
            assert_eq!(loaded.value.len(), 1, "{kind:?}");
            assert_eq!(loaded.value[0].config_ref.slug, expected_slug, "{kind:?}");
        }
        let agents = list_config_items_inner(&app, &state, ConfigKind::Agent, None).unwrap();
        assert_eq!(agents.value.len(), 5); // 4 built-in + agents/reviewer.md
    }

    #[test]
    fn list_config_items_command_meta_carries_model_and_tools() {
        let app = app_handle();
        let loaded = list_config_items_inner(&app, &app.state::<AppState>(), ConfigKind::Command, None).unwrap();
        let meta = &loaded.value[0].meta;
        assert_eq!(meta.get("model").map(String::as_str), Some("claude-opus-4-6"));
        assert_eq!(meta.get("allowedTools").map(String::as_str), Some("Read, Grep, Bash"));
    }

    #[test]
    fn list_config_items_rejects_an_unknown_project_as_invalid_input() {
        let app = app_handle();
        let project = Some(ProjectContext::Project { project_id: "-Users-nobody-nowhere".into() });
        assert!(matches!(list_config_items_inner(&app, &app.state::<AppState>(), ConfigKind::Command, project).unwrap_err(), AppError::InvalidInput { .. }));
    }

    #[test]
    fn list_config_items_rejects_an_unpicked_folder_as_invalid_input() {
        let app = app_handle();
        let project = Some(ProjectContext::PickedFolder);
        assert!(matches!(list_config_items_inner(&app, &app.state::<AppState>(), ConfigKind::Command, project).unwrap_err(), AppError::InvalidInput { .. }));
    }

    #[test]
    fn get_config_item_returns_the_commands_body() {
        let app = app_handle();
        let config_ref = ConfigRef { kind: ConfigKind::Command, project: None, slug: "review".into() };
        let item = get_config_item_inner(&app, &app.state::<AppState>(), config_ref).unwrap();
        assert!(item.body.contains("$ARGUMENTS"));
        assert!(!item.display_path.is_empty());
    }

    #[test]
    fn get_config_item_a_built_in_agent_has_no_display_path_and_a_synthetic_ref() {
        let app = app_handle();
        let config_ref = ConfigRef { kind: ConfigKind::Agent, project: None, slug: "explore".into() };
        let item = get_config_item_inner(&app, &app.state::<AppState>(), config_ref).unwrap();
        assert_eq!((item.scope, item.display_path.as_str()), (ConfigItemScope::BuiltIn, ""));
    }

    #[test]
    fn get_config_item_unknown_slug_is_not_found() {
        let app = app_handle();
        let config_ref = ConfigRef { kind: ConfigKind::Command, project: None, slug: "nonexistent".into() };
        assert!(matches!(get_config_item_inner(&app, &app.state::<AppState>(), config_ref).unwrap_err(), AppError::NotFound { .. }));
    }

    #[test]
    fn list_hooks_reads_both_matcher_groups_and_drops_the_unrecognised_handler_type() {
        let app = app_handle();
        let loaded = list_hooks_inner(&app, &app.state::<AppState>(), None).unwrap();
        assert_eq!(loaded.value.len(), 2);
        let pre_tool_use = loaded.value.iter().find(|h| h.event == "PreToolUse").unwrap();
        // The fixture's third handler is an unrecognised "websocket" type; HookHandler::from_json
        // (M1-25) only knows "command" and "http", so it is dropped, not kept with extra fields.
        assert_eq!(pre_tool_use.handlers.len(), 2);
    }

    #[test]
    fn list_plugins_reflects_settings_json_enabled_state() {
        let app = app_handle();
        let plugins = list_plugins_inner(&app.state::<AppState>()).unwrap();
        assert_eq!(plugins.len(), 2);
        let review = plugins.iter().find(|p| p.key == "code-review-bundle@acme").unwrap();
        let legacy = plugins.iter().find(|p| p.key == "legacy-tools@acme").unwrap();
        assert_eq!((review.enabled, review.version.as_deref()), (true, Some("1.4.0")));
        assert!(!legacy.enabled);
    }

    #[test]
    fn list_models_uses_the_global_settings_model_as_default() {
        let app = app_handle();
        let view = list_models_inner(&app, &app.state::<AppState>(), None).unwrap();
        assert!(!view.models.is_empty());
        assert!(view.default_model.contains("Opus")); // claude-opus-4-6's friendly name
        assert_eq!(view.project_model, None);
    }

    #[test]
    fn list_memory_sources_finds_the_fixture_projects_memory() {
        let app = app_handle();
        let sources = list_memory_sources_inner(&app, &app.state::<AppState>()).unwrap();
        assert!(sources.iter().any(|s| s.id == format!("project:{PROJECT}")));
    }

    #[test]
    fn list_memory_files_and_get_memory_file_round_trip() {
        let app = app_handle();
        let state = app.state::<AppState>();
        let source_id = format!("project:{PROJECT}");
        let files = list_memory_files_inner(&app, &state, &source_id).unwrap();
        assert!(!files.is_empty());
        let detail = get_memory_file_inner(&app, &state, &source_id, &files[0].id).unwrap();
        assert_eq!(detail.filename, files[0].filename);
        assert!(!detail.content.is_empty());
    }

    #[test]
    fn list_memory_files_rejects_a_malformed_source_id() {
        let app = app_handle();
        assert!(matches!(list_memory_files_inner(&app, &app.state::<AppState>(), "not-a-source-id").unwrap_err(), AppError::InvalidInput { .. }));
    }
}
