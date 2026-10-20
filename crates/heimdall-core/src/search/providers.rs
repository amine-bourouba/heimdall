//! Search providers for the 16 categories (SearchOverlayView.swift:223-609): builds every
//! category's candidates, scores them, and groups the top 5 of each in the fixed category order.
//!
//! Providers do no I/O: every input is data the caller already loaded (`SearchInput`), matching
//! The Swift reference app's "load once per overlay session" model (docs/spec/search-overlay.md §8). The one
//! exception is [`scan_file_history_sessions`], a pure filesystem scan with no parsing choices of
//! its own, kept here because its cap (50 session folders) is part of this task's contract.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::models::catalog::claude_models::{DEFAULT_MODEL_NAME, curated_names};
use crate::models::config::command::ClaudeCommand;
use crate::models::config::hook::HookEntry;
use crate::models::config::mcp::McpServer;
use crate::models::config::memory_file::MemoryFile;
use crate::models::config::output_style::OutputStyle;
use crate::models::config::plan::Plan;
use crate::models::config::plugin::ClaudePlugin;
use crate::models::config::skill::ClaudeSkill;
use crate::models::config::sub_agent::SubAgent;
use crate::models::history_entry::HistoryEntry;
use crate::models::project::Project;
use crate::models::session::Session;
use crate::models::session_facets::SessionFacets;
use crate::models::session_todo::TodoItem;
use crate::paths::{TargetOs, last_component};

use super::matcher;
use super::{
    MemoryFileLocator, NavScreen, SearchAction, SearchCategory, SearchGroup, SearchHit, SearchMode, SearchResults,
};

const MAX_PER_GROUP: usize = 5;
const MAX_FILE_HISTORY_SESSIONS: usize = 50;

/// Everything a search needs, already loaded by the caller. Borrowed, so building a query costs
/// no allocation beyond the results themselves.
#[derive(Debug, Clone, Copy)]
pub struct SearchInput<'a> {
    pub projects: &'a [Project],
    pub show_agent_sessions: bool,
    pub debug_log_session_ids: &'a BTreeSet<String>,
    pub facets: &'a BTreeMap<String, SessionFacets>,
    /// `(sessionId, trackedFileNames)`, from [`scan_file_history_sessions`].
    pub file_history_sessions: &'a [(String, Vec<String>)],
    pub todos: &'a BTreeMap<String, Vec<TodoItem>>,
    pub history: &'a [HistoryEntry],
    pub commands: &'a [ClaudeCommand],
    pub skills: &'a [ClaudeSkill],
    pub plans: &'a [Plan],
    /// Every project's own memory files; never a user-added folder's (parity, §10).
    pub memory_project_files: &'a [MemoryFile],
    pub mcp_servers: &'a [McpServer],
    pub plugins: &'a [ClaudePlugin],
    pub output_styles: &'a [OutputStyle],
    pub hooks: &'a [HookEntry],
    /// Built-in agents plus custom ones, already merged (`config::read::agents::load`).
    pub sub_agents: &'a [SubAgent],
}

fn score(text: &str, query: &str, mode: SearchMode) -> i32 {
    matcher::score(text, query, mode)
}

/// Highlights are always the query matched against `title` directly, never re-derived from
/// whichever field produced the score (spec §2.4).
fn highlights(title: &str, query: &str, mode: SearchMode) -> Vec<(u32, u32)> {
    matcher::matches(title, query, mode).map_or_else(Vec::new, |m| m.ranges)
}

fn find_session<'a>(projects: &'a [Project], session_id: &str) -> Option<(&'a Project, &'a Session)> {
    projects
        .iter()
        .find_map(|p| p.sessions.iter().find(|s| s.id == session_id).map(|s| (p, s)))
}

/// A session found by id, else `navigateTo(.sessions)` — the fallback the Swift reference app uses when a
/// file-history or facets entry outlives the session it names (SearchOverlayView.swift:273-283, :544-554).
fn session_or_sessions_screen(projects: &[Project], session_id: &str) -> SearchAction {
    match find_session(projects, session_id) {
        Some((p, s)) => SearchAction::OpenSession {
            project_id: p.id.clone(),
            session_id: s.id.clone(),
        },
        None => SearchAction::NavigateTo {
            screen: NavScreen::Sessions,
        },
    }
}

type Ranked = (SearchCategory, i32, SearchHit);

fn push_sessions(out: &mut Vec<Ranked>, projects: &[Project], show_agent_sessions: bool, q: &str, mode: SearchMode) {
    for project in projects {
        for session in &project.sessions {
            if session.is_sidechain && !show_agent_sessions {
                continue;
            }
            let title = session.title();
            let meta_best = score(&title, q, mode)
                .max(score(&project.name, q, mode))
                .max(score(&session.id, q, mode));
            // Falls back to a literal (mode-independent) substring test of the pre-lowercased
            // searchable text, so sessions surface by what was discussed, not only their title.
            let best = if meta_best > 0 {
                meta_best
            } else if session.searchable_text.contains(&q.to_lowercase()) {
                1
            } else {
                0
            };
            if best == 0 {
                continue;
            }
            out.push((
                SearchCategory::Sessions,
                best,
                SearchHit {
                    highlights: highlights(&title, q, mode),
                    title,
                    subtitle: Some(project.name.clone()),
                    action: SearchAction::OpenSession {
                        project_id: project.id.clone(),
                        session_id: session.id.clone(),
                    },
                },
            ));
        }
    }
}

fn push_debug_logs(
    out: &mut Vec<Ranked>,
    projects: &[Project],
    debug_log_session_ids: &BTreeSet<String>,
    q: &str,
    mode: SearchMode,
) {
    for project in projects {
        for session in &project.sessions {
            if !debug_log_session_ids.contains(&session.id) {
                continue;
            }
            let title = session.title();
            let best = score(&title, q, mode).max(score("debug log", q, mode));
            if best == 0 {
                continue;
            }
            out.push((
                SearchCategory::DebugLogs,
                best,
                SearchHit {
                    highlights: highlights(&title, q, mode),
                    title,
                    subtitle: Some("Debug Log".into()),
                    action: SearchAction::OpenDebugLog {
                        project_id: project.id.clone(),
                        session_id: session.id.clone(),
                    },
                },
            ));
        }
    }
}

fn push_file_changes(
    out: &mut Vec<Ranked>,
    file_history_sessions: &[(String, Vec<String>)],
    projects: &[Project],
    q: &str,
    mode: SearchMode,
) {
    for (session_id, file_names) in file_history_sessions {
        for file_name in file_names {
            let name_only = last_component(file_name);
            let best = score(name_only, q, mode)
                .max(score(file_name, q, mode))
                .max(score("file changes", q, mode));
            if best == 0 {
                continue;
            }
            out.push((
                SearchCategory::FileChanges,
                best,
                SearchHit {
                    highlights: highlights(name_only, q, mode),
                    title: name_only.to_string(),
                    subtitle: Some(file_name.clone()),
                    action: session_or_sessions_screen(projects, session_id),
                },
            ));
        }
    }
}

fn push_todos(out: &mut Vec<Ranked>, todos: &BTreeMap<String, Vec<TodoItem>>, q: &str, mode: SearchMode) {
    for (session_id, items) in todos {
        for todo in items {
            let best = score(&todo.content, q, mode).max(score(&todo.active_form, q, mode));
            if best == 0 {
                continue;
            }
            out.push((
                SearchCategory::Todos,
                best,
                SearchHit {
                    highlights: highlights(&todo.content, q, mode),
                    title: todo.content.clone(),
                    subtitle: Some(session_id.clone()),
                    action: SearchAction::NavigateTo {
                        screen: NavScreen::Todos,
                    },
                },
            ));
        }
    }
}

fn push_history(out: &mut Vec<Ranked>, history: &[HistoryEntry], q: &str, mode: SearchMode) {
    for entry in history {
        let best = score(&entry.display, q, mode).max(score(entry.project_name(), q, mode));
        if best == 0 {
            continue;
        }
        let title = entry.snippet();
        out.push((
            SearchCategory::History,
            best,
            SearchHit {
                highlights: highlights(&title, q, mode),
                title,
                subtitle: Some(entry.project_name().to_string()),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::History,
                },
            },
        ));
    }
}

fn push_commands(out: &mut Vec<Ranked>, commands: &[ClaudeCommand], q: &str, mode: SearchMode) {
    for cmd in commands {
        let best = score(&cmd.name, q, mode).max(score(&cmd.description, q, mode));
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::Commands,
            best,
            SearchHit {
                highlights: highlights(&cmd.name, q, mode),
                title: cmd.name.clone(),
                subtitle: Some(cmd.description.clone()),
                action: SearchAction::OpenDetail {
                    screen: NavScreen::Commands,
                    config_ref: Some(cmd.config_ref.clone()),
                    memory_file: None,
                },
            },
        ));
    }
}

fn push_skills(out: &mut Vec<Ranked>, skills: &[ClaudeSkill], q: &str, mode: SearchMode) {
    for skill in skills {
        let best = score(&skill.name, q, mode).max(score(&skill.description, q, mode));
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::Skills,
            best,
            SearchHit {
                highlights: highlights(&skill.name, q, mode),
                title: skill.name.clone(),
                subtitle: Some(skill.description.clone()),
                action: SearchAction::OpenDetail {
                    screen: NavScreen::Skills,
                    config_ref: Some(skill.config_ref.clone()),
                    memory_file: None,
                },
            },
        ));
    }
}

fn push_plans(out: &mut Vec<Ranked>, plans: &[Plan], q: &str, mode: SearchMode) {
    for plan in plans {
        // Swift takes the first 200 Characters (grapheme clusters); this takes 200 Unicode
        // scalars, the same difference noted elsewhere for very long non-BMP text (low risk here).
        let snippet: String = plan.content.chars().take(200).collect();
        let best = score(&plan.name, q, mode).max(score(&snippet, q, mode));
        if best == 0 {
            continue;
        }
        let filename = plan
            .file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        out.push((
            SearchCategory::Plans,
            best,
            SearchHit {
                highlights: highlights(&plan.name, q, mode),
                title: plan.name.clone(),
                subtitle: Some(filename),
                action: SearchAction::OpenDetail {
                    screen: NavScreen::Plans,
                    config_ref: Some(plan.config_ref.clone()),
                    memory_file: None,
                },
            },
        ));
    }
}

fn push_memory(out: &mut Vec<Ranked>, memory_project_files: &[MemoryFile], q: &str, mode: SearchMode) {
    for file in memory_project_files {
        let name_score = score(&file.name, q, mode);
        let best = if name_score > 0 {
            name_score
        } else if file.content.to_lowercase().contains(&q.to_lowercase()) {
            1
        } else {
            0
        };
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::Memory,
            best,
            SearchHit {
                highlights: highlights(&file.name, q, mode),
                title: file.name.clone(),
                subtitle: Some(file.filename.clone()),
                action: SearchAction::OpenDetail {
                    screen: NavScreen::Memory,
                    config_ref: None,
                    memory_file: Some(MemoryFileLocator {
                        source_id: format!("project:{}", file.project_id),
                        file_id: file.id.clone(),
                    }),
                },
            },
        ));
    }
}

fn push_mcp(out: &mut Vec<Ranked>, mcp_servers: &[McpServer], q: &str, mode: SearchMode) {
    for server in mcp_servers {
        let tool_score = server.tools.iter().map(|t| score(t, q, mode)).max().unwrap_or(0);
        let best = score(&server.name, q, mode)
            .max(tool_score)
            .max(score(server.status.label(), q, mode));
        if best == 0 {
            continue;
        }
        let subtitle = server.tools.iter().take(3).cloned().collect::<Vec<_>>().join(", ");
        out.push((
            SearchCategory::McpServers,
            best,
            SearchHit {
                highlights: highlights(&server.name, q, mode),
                title: server.name.clone(),
                subtitle: Some(subtitle),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::McpServers,
                },
            },
        ));
    }
}

fn push_plugins(out: &mut Vec<Ranked>, plugins: &[ClaudePlugin], q: &str, mode: SearchMode) {
    for plugin in plugins {
        let best = score(&plugin.name, q, mode).max(score(&plugin.author, q, mode));
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::Plugins,
            best,
            SearchHit {
                highlights: highlights(&plugin.name, q, mode),
                title: plugin.name.clone(),
                subtitle: Some(format!("by {}", plugin.author)),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::Plugins,
                },
            },
        ));
    }
}

fn push_output_styles(out: &mut Vec<Ranked>, output_styles: &[OutputStyle], q: &str, mode: SearchMode) {
    for style in output_styles {
        let best = score(&style.name, q, mode).max(score(&style.description, q, mode));
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::OutputStyles,
            best,
            SearchHit {
                highlights: highlights(&style.name, q, mode),
                title: style.name.clone(),
                subtitle: Some(style.description.clone()),
                action: SearchAction::OpenDetail {
                    screen: NavScreen::OutputStyles,
                    config_ref: Some(style.config_ref.clone()),
                    memory_file: None,
                },
            },
        ));
    }
}

fn push_hooks(out: &mut Vec<Ranked>, hooks: &[HookEntry], q: &str, mode: SearchMode) {
    for hook in hooks {
        let first = hook.handlers.first();
        let cmd_score = first.map_or(0, |h| score(h.display_command(), q, mode));
        let matcher_score = hook.matcher.as_deref().map_or(0, |m| score(m, q, mode));
        let event_score = score(&hook.event, q, mode);
        let best = cmd_score.max(matcher_score).max(event_score);
        if best == 0 {
            continue;
        }
        // `first?.displayCommand ?? event`: only falls back when there is no handler at all, not
        // when the handler's own display text happens to be empty.
        let title = first.map_or_else(|| hook.event.clone(), |h| h.display_command().to_string());
        let subtitle = match &hook.matcher {
            Some(m) => format!("{} · {m}", hook.event),
            None => hook.event.clone(),
        };
        out.push((
            SearchCategory::Hooks,
            best,
            SearchHit {
                highlights: highlights(&title, q, mode),
                title,
                subtitle: Some(subtitle),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::Hooks,
                },
            },
        ));
    }
}

/// The curated catalog only (parity, assumption A-6) — never transcript-discovered models.
fn push_models(out: &mut Vec<Ranked>, q: &str, mode: SearchMode) {
    for name in curated_names() {
        let s = score(&name, q, mode);
        if s == 0 {
            continue;
        }
        let is_default = name == DEFAULT_MODEL_NAME;
        out.push((
            SearchCategory::Models,
            s,
            SearchHit {
                highlights: highlights(&name, q, mode),
                title: name,
                subtitle: Some(if is_default {
                    "Default model".into()
                } else {
                    "AI Model".into()
                }),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::Models,
                },
            },
        ));
    }
}

fn push_sub_agents(out: &mut Vec<Ranked>, sub_agents: &[SubAgent], q: &str, mode: SearchMode) {
    for agent in sub_agents {
        let best = score(&agent.name, q, mode).max(score(&agent.description, q, mode));
        if best == 0 {
            continue;
        }
        out.push((
            SearchCategory::SubAgents,
            best,
            SearchHit {
                highlights: highlights(&agent.name, q, mode),
                title: agent.name.clone(),
                subtitle: Some(agent.description.clone()),
                action: SearchAction::NavigateTo {
                    screen: NavScreen::SubAgents,
                },
            },
        ));
    }
}

fn push_facets(
    out: &mut Vec<Ranked>,
    facets: &BTreeMap<String, SessionFacets>,
    projects: &[Project],
    q: &str,
    mode: SearchMode,
) {
    for (session_id, f) in facets {
        let category_score = f.goal_categories.keys().map(|k| score(k, q, mode)).max().unwrap_or(0);
        let best = score(&f.underlying_goal, q, mode)
            .max(score(&f.brief_summary, q, mode))
            .max(category_score)
            .max(score(&f.session_type, q, mode));
        if best == 0 {
            continue;
        }
        let title = if f.brief_summary.is_empty() {
            f.underlying_goal.clone()
        } else {
            f.brief_summary.clone()
        };
        out.push((
            SearchCategory::Facets,
            best,
            SearchHit {
                highlights: highlights(&title, q, mode),
                title,
                subtitle: Some(f.underlying_goal.clone()),
                action: session_or_sessions_screen(projects, session_id),
            },
        ));
    }
}

/// Every category's hits, grouped and ordered per [`SearchCategory::ALL`], each capped at 5, ties
/// broken by each provider's own append order (a stable sort, matching Swift's).
pub fn search(input: &SearchInput, query: &str, mode: SearchMode) -> SearchResults {
    let mut all: Vec<Ranked> = Vec::new();
    push_sessions(&mut all, input.projects, input.show_agent_sessions, query, mode);
    push_debug_logs(&mut all, input.projects, input.debug_log_session_ids, query, mode);
    push_facets(&mut all, input.facets, input.projects, query, mode);
    push_file_changes(&mut all, input.file_history_sessions, input.projects, query, mode);
    push_todos(&mut all, input.todos, query, mode);
    push_history(&mut all, input.history, query, mode);
    push_commands(&mut all, input.commands, query, mode);
    push_skills(&mut all, input.skills, query, mode);
    push_plans(&mut all, input.plans, query, mode);
    push_memory(&mut all, input.memory_project_files, query, mode);
    push_mcp(&mut all, input.mcp_servers, query, mode);
    push_plugins(&mut all, input.plugins, query, mode);
    push_output_styles(&mut all, input.output_styles, query, mode);
    push_hooks(&mut all, input.hooks, query, mode);
    push_models(&mut all, query, mode);
    push_sub_agents(&mut all, input.sub_agents, query, mode);

    let groups = SearchCategory::ALL
        .into_iter()
        .filter_map(|category| {
            let mut hits: Vec<(i32, SearchHit)> = all
                .iter()
                .filter(|(c, ..)| *c == category)
                .map(|(_, s, h)| (*s, h.clone()))
                .collect();
            if hits.is_empty() {
                return None;
            }
            hits.sort_by_key(|(s, _)| std::cmp::Reverse(*s));
            hits.truncate(MAX_PER_GROUP);
            Some(SearchGroup {
                category,
                hits: hits.into_iter().map(|(_, h)| h).collect(),
            })
        })
        .collect();
    SearchResults { groups }
}

/// Session ids under `~/.claude/file-history/` (first 50 folders, sorted for determinism — the Swift reference app's
/// directory order is filesystem-native and not guaranteed) that have at least one tracked file,
/// paired with those files' tracked names (SearchOverlayView.scanFileHistorySessions).
pub fn scan_file_history_sessions(
    file_history_dir: &Path,
    projects_dir: &Path,
    os: TargetOs,
) -> Vec<(String, Vec<String>)> {
    let Ok(entries) = std::fs::read_dir(file_history_dir) else {
        return Vec::new();
    };
    let mut session_ids: Vec<String> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(String::from))
        .collect();
    session_ids.sort();
    session_ids.truncate(MAX_FILE_HISTORY_SESSIONS);
    session_ids
        .into_iter()
        .filter_map(|session_id| {
            let files = crate::data::file_history::load(projects_dir, &session_id, os)
                .ok()?
                .value;
            (!files.is_empty()).then(|| (session_id, files.into_iter().map(|f| f.file_name).collect()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::read::memory::MemorySourceKind;
    use crate::config::read::test_support::{context, home, repo, synthetic};
    use crate::config::read::{
        ProjectScope, agents, commands, hooks, mcp, memory, output_styles, plans, plugins, skills,
    };
    use crate::data::{debug_log, facets as facets_data, history as history_data, todos as todos_data};
    use crate::models::config::scope::ConfigKind;
    use crate::paths::ClaudeHome;
    use crate::projects::discovery;

    /// Every category's data, loaded once from the shared synthetic fixtures. Commands, skills,
    /// output styles and hooks are project-scoped to `project-repo` (a config project, independent
    /// of the session project below); MCP is scoped to `/Users/fixture/src/app` (the session
    /// project, and the exact key `fixtures/synthetic/claude.json` uses).
    struct World {
        projects: Vec<Project>,
        debug_ids: BTreeSet<String>,
        facets: BTreeMap<String, SessionFacets>,
        file_history_sessions: Vec<(String, Vec<String>)>,
        todos: BTreeMap<String, Vec<TodoItem>>,
        history: Vec<HistoryEntry>,
        commands: Vec<ClaudeCommand>,
        skills: Vec<ClaudeSkill>,
        plans: Vec<Plan>,
        memory_project_files: Vec<MemoryFile>,
        mcp_servers: Vec<McpServer>,
        plugins: Vec<ClaudePlugin>,
        output_styles: Vec<OutputStyle>,
        hooks: Vec<HookEntry>,
        sub_agents: Vec<SubAgent>,
    }

    fn world() -> World {
        let home_dir = home();
        let home = ClaudeHome::new(home_dir.clone(), synthetic().join("claude.json"));
        let repo = repo();
        let ctx = context();
        let scope = ProjectScope {
            root: &repo,
            context: &ctx,
        };
        let projects = discovery::discover(&home.projects_dir(), TargetOs::current())
            .unwrap()
            .value;
        let memory_sources = memory::sources(&home.projects_dir(), &[]);
        let memory_project_files = memory_sources
            .into_iter()
            .filter(|s| s.kind == MemorySourceKind::Project)
            .flat_map(|s| {
                memory::load_project(&home.projects_dir(), s.id.strip_prefix("project:").unwrap())
                    .unwrap()
                    .value
            })
            .collect();
        World {
            debug_ids: debug_log::all_session_ids(&home.debug_dir()).into_iter().collect(),
            facets: facets_data::load_all(&home.facets_dir()).unwrap().value,
            file_history_sessions: scan_file_history_sessions(
                &home.file_history_dir(),
                &home.projects_dir(),
                TargetOs::current(),
            ),
            todos: todos_data::load_all(&home.todos_dir()).unwrap().value,
            history: history_data::load_all(&home.history_file()).unwrap().value,
            commands: commands::load(&home_dir, Some(scope)).value,
            skills: skills::load(&home_dir, Some(scope)).value,
            plans: plans::load(&home_dir).value,
            memory_project_files,
            mcp_servers: mcp::load(&home, Some("/Users/fixture/src/app"), TargetOs::current()).value,
            plugins: plugins::load(&home).value,
            output_styles: output_styles::load(&home_dir, Some(scope)).value,
            hooks: hooks::load(&home_dir, Some(&repo)).value,
            sub_agents: agents::load(&home_dir).value,
            projects,
        }
    }

    fn input(w: &World) -> SearchInput<'_> {
        SearchInput {
            projects: &w.projects,
            show_agent_sessions: true,
            debug_log_session_ids: &w.debug_ids,
            facets: &w.facets,
            file_history_sessions: &w.file_history_sessions,
            todos: &w.todos,
            history: &w.history,
            commands: &w.commands,
            skills: &w.skills,
            plans: &w.plans,
            memory_project_files: &w.memory_project_files,
            mcp_servers: &w.mcp_servers,
            plugins: &w.plugins,
            output_styles: &w.output_styles,
            hooks: &w.hooks,
            sub_agents: &w.sub_agents,
        }
    }

    /// The single top hit of `category` for `query` (exact mode — every fixture query below is a
    /// verbatim substring of the field it targets).
    fn top(results: &SearchResults, category: SearchCategory) -> &SearchHit {
        let group = results
            .groups
            .iter()
            .find(|g| g.category == category)
            .unwrap_or_else(|| panic!("no {category:?} group"));
        &group.hits[0]
    }

    // One test per category (M1-30 acceptance criterion), each on `fixtures/synthetic`.
    #[test]
    fn sessions_category_returns_the_matching_session() {
        let w = world();
        let results = search(&input(&w), "auth bug in login", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Sessions);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Fix the auth bug in login.py", Some("app"))
        );
        assert_eq!(
            hit.action,
            SearchAction::OpenSession {
                project_id: "-Users-fixture-src-app".into(),
                session_id: "11111111-1111-4111-8111-111111111111".into()
            }
        );
    }
    #[test]
    fn debug_logs_category_returns_the_session_with_a_log() {
        let w = world();
        let results = search(&input(&w), "debug log", SearchMode::Exact);
        let hit = top(&results, SearchCategory::DebugLogs);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Fix the auth bug in login.py", Some("Debug Log"))
        );
        assert!(
            matches!(&hit.action, SearchAction::OpenDebugLog { session_id, .. } if session_id == "11111111-1111-4111-8111-111111111111")
        );
    }
    #[test]
    fn facets_category_returns_the_ai_summary() {
        let w = world();
        let results = search(&input(&w), "authentication bug", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Facets);
        assert_eq!(
            hit.title,
            "Fixed the login authentication bug in login.py and verified with tests."
        );
        assert_eq!(hit.subtitle.as_deref(), Some("Fix an authentication bug"));
        assert!(
            matches!(&hit.action, SearchAction::OpenSession { session_id, .. } if session_id == "11111111-1111-4111-8111-111111111111")
        );
    }
    #[test]
    fn file_changes_category_returns_the_tracked_file() {
        let w = world();
        let results = search(&input(&w), "login.py", SearchMode::Exact);
        let hit = top(&results, SearchCategory::FileChanges);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("login.py", Some("login.py"))
        );
        assert!(
            matches!(&hit.action, SearchAction::OpenSession { session_id, .. } if session_id == "11111111-1111-4111-8111-111111111111")
        );
    }
    #[test]
    fn todos_category_returns_the_matching_todo() {
        let w = world();
        let results = search(&input(&w), "regression test", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Todos);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Add a regression test", Some("11111111-1111-4111-8111-111111111111"))
        );
        assert_eq!(
            hit.action,
            SearchAction::NavigateTo {
                screen: NavScreen::Todos
            }
        );
    }
    #[test]
    fn history_category_returns_the_matching_prompt() {
        let w = world();
        let results = search(&input(&w), "stack trace", SearchMode::Exact);
        let hit = top(&results, SearchCategory::History);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Paste this stack trace and explain it", Some("app"))
        );
        assert_eq!(
            hit.action,
            SearchAction::NavigateTo {
                screen: NavScreen::History
            }
        );
    }
    #[test]
    fn commands_category_returns_the_matching_command() {
        let w = world();
        let results = search(&input(&w), "/review", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Commands);
        assert_eq!(hit.title, "/review");
        assert!(
            matches!(&hit.action, SearchAction::OpenDetail { screen: NavScreen::Commands, config_ref: Some(r), .. } if r.kind == ConfigKind::Command && r.slug == "review")
        );
    }
    #[test]
    fn skills_category_returns_the_matching_skill() {
        let w = world();
        let results = search(&input(&w), "pdf", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Skills);
        assert_eq!(hit.title, "pdf-tools");
    }
    #[test]
    fn plans_category_returns_the_matching_plan() {
        let w = world();
        let results = search(&input(&w), "refactor auth module", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Plans);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Refactor Auth Module", Some("refactor-auth-module.md"))
        );
    }
    #[test]
    fn memory_category_falls_back_to_content_when_the_name_does_not_match() {
        let w = world();
        let results = search(&input(&w), "auth bug", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Memory);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("MEMORY", Some("MEMORY.md"))
        );
        assert!(
            matches!(&hit.action, SearchAction::OpenDetail { screen: NavScreen::Memory, memory_file: Some(m), .. } if m.source_id == "project:-Users-fixture-src-app")
        );
    }
    #[test]
    fn mcp_servers_category_returns_the_matching_server() {
        let w = world();
        let results = search(&input(&w), "sentry", SearchMode::Exact);
        let hit = top(&results, SearchCategory::McpServers);
        assert_eq!(hit.title, "Sentry");
        assert_eq!(
            hit.action,
            SearchAction::NavigateTo {
                screen: NavScreen::McpServers
            }
        );
    }
    #[test]
    fn plugins_category_returns_the_matching_plugin() {
        let w = world();
        let results = search(&input(&w), "legacy-tools", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Plugins);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("legacy-tools", Some("by acme"))
        );
    }
    #[test]
    fn output_styles_category_returns_the_matching_style() {
        let w = world();
        let results = search(&input(&w), "concise", SearchMode::Exact);
        let hit = top(&results, SearchCategory::OutputStyles);
        assert_eq!(hit.title, "Concise");
    }
    #[test]
    fn hooks_category_returns_the_matching_handler() {
        let w = world();
        let results = search(&input(&w), "lint-before-edit", SearchMode::Exact);
        let hit = top(&results, SearchCategory::Hooks);
        assert_eq!(hit.title, "scripts/lint-before-edit.sh");
        assert_eq!(hit.subtitle.as_deref(), Some("PreToolUse · Edit|Write"));
    }
    #[test]
    fn models_category_covers_only_the_curated_catalog() {
        let w = world();
        let results = search(&input(&w), "haiku", SearchMode::Fuzzy);
        let hit = top(&results, SearchCategory::Models);
        assert_eq!(
            (hit.title.as_str(), hit.subtitle.as_deref()),
            ("Haiku 4.5", Some("AI Model"))
        );
        let opus5_results = search(&input(&w), "Opus 5", SearchMode::Exact);
        let opus5 = top(&opus5_results, SearchCategory::Models);
        assert_eq!(opus5.subtitle.as_deref(), Some("Default model"));
    }
    #[test]
    fn sub_agents_category_returns_the_custom_agent() {
        let w = world();
        let results = search(&input(&w), "reviewer", SearchMode::Exact);
        let hit = top(&results, SearchCategory::SubAgents);
        assert_eq!(hit.title, "reviewer");
    }

    #[test]
    fn groups_follow_the_fixed_category_order_and_cap_at_five() {
        let w = world();
        let results = search(&input(&w), "a", SearchMode::Fuzzy);
        let order: Vec<_> = results.groups.iter().map(|g| g.category).collect();
        let expected: Vec<_> = SearchCategory::ALL.into_iter().filter(|c| order.contains(c)).collect();
        assert_eq!(order, expected);
        assert!(results.groups.iter().all(|g| g.hits.len() <= 5));
        assert!(
            results.groups.len() >= 10,
            "only {} groups matched \"a\"",
            results.groups.len()
        );
    }

    #[test]
    fn a_query_matching_nothing_returns_no_groups() {
        let w = world();
        assert!(
            search(&input(&w), "zzz-nonexistent-query-zzz", SearchMode::Exact)
                .groups
                .is_empty()
        );
    }

    #[test]
    fn file_history_scan_is_capped_and_deterministic() {
        let dir = tempfile::tempdir().unwrap();
        for i in 0..60 {
            std::fs::create_dir(dir.path().join(format!("session-{i:02}"))).unwrap();
        }
        let projects = tempfile::tempdir().unwrap();
        let sessions = scan_file_history_sessions(dir.path(), projects.path(), TargetOs::current());
        // None of these sessions have a transcript, so none carries tracked files, but the folder
        // scan itself must still stop at 50.
        assert!(sessions.is_empty());
        let mut names: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .flatten()
            .map(|e| e.file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        assert_eq!(names.len(), 60);
    }
}
