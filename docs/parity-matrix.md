# Feature-parity matrix

This matrix defines "done" for the port. Each row is one Poirot capability, its Swift reference (paths relative to `$POIROT/Poirot/Sources/` unless noted, verified at commit `a29dd6e`), its Heimdall target, and a status per OS.

**How to update.** Only the gate-review tasks edit this file: M1-59, M2-30, M3-20, M4-14 and M3-62. Other tasks record evidence in `docs/progress-log.md`, and the next gate review copies the status here. New upstream capabilities become new rows (see `docs/spec/upstream-sync.md`); they never reopen a finished milestone.

**Status values.** `NS` not started · `IP` in progress · `AP` at parity · `DG` degraded · `HD` hidden · `RP` replaced by an OS-native or web-native equivalent · `OOS` out of scope. A row that cannot reach `AP` on an OS records exactly one outcome (DG, HD or RP) in the Outcome column.

**Scope.** `MVP` ships in the first release. `Later` ships after it. See port-plan §11.

**Workstreams.** RUST data core · SHELL Tauri shell/OS integration · UI Vue shell and design system · FEAT feature screens · SEC security · TEST testing · REL release.

## A. Scenes and windows

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| A1 | Main window: 1280×820 default, 900×600 minimum, hidden title bar with glass chrome | `App/PoirotApp.swift:38-46`, `App/ContentView.swift:82-157` | `src-tauri/src/window.rs`, `src/layout/AppShell.vue` | MVP | NS | NS | NS | Win/Linux RP: native title bar (P-4). macOS: overlay title bar at parity | SHELL, UI | M1-03, M2-12, M2-09 |
| A2 | Multiple main windows (WindowGroup, Dock "New Window") | `App/PoirotApp.swift:12-22, 38` | — | Later | NS | NS | NS | HD on all OSes: single main window in MVP (P-12) | — | — |
| A3 | About window | `App/AboutView.swift`, `App/PoirotApp.swift:106-110` | `src/features/about/AboutDialog.vue` | MVP | NS | NS | NS | RP: in-app dialog (P-6) | FEAT | M3-17 |
| A4 | Help window | `Views/Help/HelpView.swift`, `App/PoirotApp.swift:112-116` | `src/features/help/HelpPage.vue` | MVP | NS | NS | NS | RP: in-app view (P-6) | FEAT | M3-17 |
| A5 | Settings scene, 5 tabs (General, Appearance, Viewer, Memory, Menu Bar) | `App/SettingsView.swift` | `src/features/settings/SettingsPage.vue` | MVP | NS | NS | NS | RP: in-app route on Mod+, (P-6); Menu Bar tab ships with the tray (Later) | FEAT | M3-16, M3-58, M3-60 |
| A6 | Menu-bar popover: header stats, search, 10 recent sessions | `Views/MenuBar/MenuBarView.swift`, `Views/MenuBar/MenuBarState.swift`, `App/PoirotApp.swift:124-128` | `src-tauri/src/tray.rs`, `src/features/tray/TrayView.vue` | Later | NS | NS | NS | Linux DG: native tray menu with recent sessions; GNOME needs the AppIndicator extension (P-15) | SHELL, FEAT | M1-21, M3-30, M3-56, M3-57, M3-58 |
| A7 | App menu: About, Check for Updates…, font size, Help, Keyboard Shortcuts | `App/PoirotApp.swift:47-104` | `src-tauri/src/menu.rs`, `src/layout/AppMenu.vue` | MVP | NS | NS | NS | Win/Linux RP: in-app menu. macOS: native menu at parity | SHELL, UI | M2-13, M2-09 |
| A8 | Window size and position restored across launches | SwiftUI default scene restoration [assumption] | — | Later | NS | NS | NS | DG on all OSes: default size each launch; no window-state plugin in the locked plugin set | — | — |

## B. Screens, sheets and overlays

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| B1 | Sidebar nav items with live counts | `Views/Components/SidebarView.swift`, `App/AppState.swift:247-268` | `src/layout/Sidebar.vue`, `get_sidebar_counts` | MVP | NS | NS | NS | — | UI, RUST | M1-40, M2-09 |
| B2 | Sessions browser: projects, sort, search, agent toggle | `Views/Session/SessionsNavigationView.swift` | `src/features/session/SessionsNavigation.vue` | MVP | NS | NS | NS | — | FEAT | M1-14, M1-41, M2-14, M2-15 |
| B3 | Project sessions, grid or list | `Views/Project/ProjectSessionsView.swift` | `src/features/session/ProjectSessions.vue` | MVP | NS | NS | NS | — | FEAT | M2-16 |
| B4 | Transcript view: virtualised paging (20), tool filter, session search, expand all | `Views/Session/SessionDetailView.swift` | `src/features/session/SessionDetail.vue` | MVP | NS | NS | NS | — | FEAT | M1-12, M1-41, M2-20 |
| B5 | Tool-call timeline | `Views/Session/SessionTimelineView.swift` | `SessionTimeline.vue` | MVP | NS | NS | NS | — | FEAT | M2-21 |
| B6 | Conversation context card | `Views/Session/ConversationContextCard.swift` | `ConversationContextCard.vue` | MVP | NS | NS | NS | — | FEAT | M1-08, M2-21 |
| B7 | Tool block: 80-line truncation, show all, bash line, copy | `Views/Session/ToolBlockView.swift` | `blocks/ToolBlock.vue` | MVP | NS | NS | NS | Open-in-editor/terminal buttons are Later (L2, L3) | FEAT | M2-18, M3-55 |
| B8 | Thinking block: 50-line truncation, default expanded | `Views/Session/ThinkingBlockView.swift` | `blocks/ThinkingBlock.vue` | MVP | NS | NS | NS | — | FEAT | M2-17 |
| B9 | System content (system-reminder segments) | `Views/Session/SystemContentView.swift`, `Utilities/SystemContentParser.swift` | `blocks/SystemContent.vue`, `transcript/system_content.rs` | MVP | NS | NS | NS | — | FEAT, RUST | M1-12, M2-17 |
| B10 | Edit diff: gutter, 5000-line cap, copy, save as patch | `Views/Session/EditDiffView.swift`, `Utilities/LineDiff.swift` | `blocks/EditDiff.vue`, `diff.rs` | MVP | NS | NS | NS | Copy-as-image is C18 (Later) | FEAT, RUST | M1-35, M2-19, M3-10 |
| B11 | Session facets card (AI summary) | `Views/Session/SessionFacetsCard.swift` | `SessionFacetsCard.vue` | MVP | NS | NS | NS | — | FEAT | M1-18, M3-06 |
| B12 | Session todos | `Views/Session/SessionTodosView.swift` | `SessionTodos.vue` | MVP | NS | NS | NS | — | FEAT | M1-16, M3-06 |
| B13 | File history viewer with diffs | `Views/Session/FileHistoryView.swift` | `FileHistory.vue` | MVP | NS | NS | NS | — | FEAT | M1-19, M3-07 |
| B14 | Debug log sheet, paged | `Views/Session/DebugLogView.swift` | `DebugLogSheet.vue` | MVP | NS | NS | NS | — | FEAT | M1-17, M3-08 |
| B15 | Export sheet: Markdown with 4 toggles, copy or save; patch export | `Views/Session/ExportOptionsView.swift`, `Services/Export/SessionExporter.swift` | `ExportOptionsDialog.vue`, `export/session_markdown.rs` | MVP | NS | NS | NS | — | FEAT, RUST | M1-36, M3-09, M3-10 |
| B16 | Session toolbar: copy resume command, copy id, file history, debug log, export | `Views/Session/SessionToolbarContent.swift` | `SessionToolbar.vue` | MVP | NS | NS | NS | Open terminal and reveal are Later (C17, L2) | FEAT | M2-22, M3-55 |
| B17 | TODOs overview across projects | `Views/Todos/TodosOverviewView.swift` | `src/features/todos/TodosPage.vue` | MVP | NS | NS | NS | — | FEAT | M3-05 |
| B18 | History list with date buckets and copy | `Views/History/HistoryListView.swift` | `src/features/history/HistoryPage.vue` | MVP | NS | NS | NS | — | FEAT | M1-15, M3-02 |
| B19 | History delete: one entry, older than N days | `Views/History/HistoryListView.swift` | `data/history_write.rs`, `HistoryPage.vue` | Later | NS | NS | NS | — | RUST, FEAT | M3-43, M3-46, M3-51 |
| B20 | Analytics dashboard: range picker, stat cards, 6 charts, heatmap | `Views/Analytics/AnalyticsDashboardView.swift`, `AnalyticsViewModel.swift` | `src/features/analytics/` | Later | NS | NS | NS | — | FEAT | M1-20, M3-30, M3-31, M3-32, M3-33, M3-34 |
| B21 | Analytics CSV export (4 kinds) | `Views/Analytics/Export/AnalyticsCSVExporter.swift` | `export/analytics_csv.rs` | Later | NS | NS | NS | — | RUST | M3-36, M3-32 |
| B22 | Analytics PNG export (branded, forced dark, 2x) | `Views/Analytics/Export/AnalyticsImageExporter.swift`, `AnalyticsSnapshotView.swift` | decided by spike | Later | NS | NS | NS | Open risk R-7 until the spike decides | FEAT | M3-37, M3-38, M3-39 |
| B23 | Commands list and detail | `Views/Config/CommandsListView.swift` | `src/features/config/commands/` | MVP | NS | NS | NS | MVP read-only; create/delete Later (F9, F11) | FEAT | M1-24, M3-12, M3-50 |
| B24 | Skills list and detail | `Views/Config/SkillsListView.swift` | `src/features/config/skills/` | MVP | NS | NS | NS | MVP read-only; create/delete Later | FEAT | M1-24, M3-12, M3-50 |
| B25 | Plans list and detail | `Views/Plans/PlansListView.swift`, `Views/Plans/PlanDetailView.swift` | `src/features/plans/` | MVP | NS | NS | NS | — | FEAT | M1-24, M3-03 |
| B26 | MCP servers list (5 sources) with live status dot | `Views/Config/MCPServersListView.swift`, `MCPServerStatusIndicator.swift` | `src/features/config/mcp/` | MVP | NS | NS | NS | MVP read-only; env/header values never shown | FEAT | M1-26, M1-28, M1-45, M3-14 |
| B27 | MCP setup wizard: add, edit, delete | `Views/Config/MCPSetupWizardView.swift` | `McpSetupWizard.vue` | Later | NS | NS | NS | Ports with fixes (args list, full masking) | FEAT, RUST | M3-41, M3-45, M3-48 |
| B28 | Models: catalog and discovered; set default/per-project | `Views/Config/ModelsListView.swift` | `src/features/config/models/` | MVP | NS | NS | NS | MVP read-only; setting models Later (F3, F4) | FEAT | M1-27, M3-13, M3-50 |
| B29 | Sub-agents: built-in and custom, create/edit, JSON import/export | `Views/Config/SubAgentsListView.swift`, `SubAgentFormView.swift` | `src/features/config/subagents/` | MVP | NS | NS | NS | MVP read-only; CRUD Later (F8, F10) | FEAT | M1-24, M3-14, M3-49 |
| B30 | Memory: project memory and custom folders | `Views/Memory/MemoryListView.swift`, `MemoryDetailView.swift` | `src/features/memory/` | MVP | NS | NS | NS | — | FEAT | M1-27, M3-04 |
| B31 | Hooks by event and scope; create/edit; JSON import/export | `Views/Config/HooksListView.swift`, `HookFormView.swift` | `src/features/config/hooks/` | MVP | NS | NS | NS | MVP read-only plus copy as JSON; CRUD Later (F5, F6) | FEAT | M1-25, M3-13, M3-47 |
| B32 | Plugins: enable, disable, remove | `Views/Config/PluginsListView.swift` | `src/features/config/plugins/` | MVP | NS | NS | NS | MVP read-only; writes Later (F1) | FEAT | M1-26, M3-13, M3-50 |
| B33 | Output styles list and detail | `Views/Config/OutputStylesListView.swift` | `src/features/config/output-styles/` | MVP | NS | NS | NS | MVP read-only; create/delete Later | FEAT | M1-24, M3-12, M3-50 |
| B34 | Universal search overlay (⌘K) | `Views/Search/SearchOverlayView.swift` | `src/features/search/SearchOverlay.vue`, `search/` | MVP | NS | NS | NS | Details in section S | FEAT, RUST | M1-29, M1-30, M1-46, M3-01 |
| B35 | Onboarding: 3 pages, not dismissible until complete | `Views/Onboarding/OnboardingView.swift` | `src/features/onboarding/Onboarding.vue` | MVP | NS | NS | NS | — | FEAT | M1-31, M1-40, M3-15 |
| B36 | Shortcut help sheet | `Views/Help/ShortcutHelpView.swift` | `src/features/help/ShortcutHelp.vue` | MVP | NS | NS | NS | Full list instead of Poirot's 9 rows (P-7) | FEAT | M3-17 |
| B37 | Toast overlay: single queue, 4 s / 10 s with URL | `Views/Components/ToastOverlay.swift` | `src/components/ToastOverlay.vue`, `src/stores/toasts.ts` | MVP | NS | NS | NS | — | UI | M2-05, M2-07 |
| B38 | Config project picker | `Views/Config/ConfigProjectPicker.swift` | `ConfigProjectPicker.vue`, `pick_config_project` | MVP | NS | NS | NS | — | FEAT, RUST | M1-39, M3-11 |
| B39 | Config detail toolbar: copy name/content | `Views/Config/ConfigDetailToolbar.swift` | `ConfigDetailToolbar.vue` | MVP | NS | NS | NS | Reveal and open-in-editor are Later (C17, L3) | FEAT | M3-11, M3-55 |

## C. Shared components and cross-cutting behaviour

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| C1 | MarqueeText (hover scroll, reduced-motion aware) | `Views/Components/MarqueeText.swift` | `MarqueeText.vue` | MVP | NS | NS | NS | — | UI | M2-07 |
| C2 | Gauge ring (used by the context card only) | `Views/Components/UsageGaugeRing.swift`, `Models/Usage/UsageWindow.swift` | `GaugeRing.vue` | MVP | NS | NS | NS | — | UI | M2-07 |
| C3 | Shimmer and bespoke skeletons (session, config, analytics) | `ShimmerModifier.swift`, `SessionSkeletonView.swift`, `ConfigSkeletonView.swift`, `AnalyticsShimmerView.swift` | `ShimmerBlock.vue` and per-feature skeletons | MVP | NS | NS | NS | Analytics skeleton ships with analytics (Later) | UI, FEAT | M2-07, M2-20, M3-11, M3-32 |
| C4 | InfoTooltipButton (click to open) | `Views/Components/InfoTooltipButton.swift` | `InfoTooltipButton.vue` | MVP | NS | NS | NS | — | UI | M2-07 |
| C5 | CardChrome (hover and selected) | `Views/Components/CardChrome.swift` | `CardChrome.vue` | MVP | NS | NS | NS | — | UI | M2-07 |
| C6 | Glass background | `Views/Components/GlassBackground.swift` | `GlassPanel.vue` (`backdrop-filter`) | MVP | NS | NS | NS | Linux DG if WebKitGTK lacks `backdrop-filter`: opaque fallback [A-11] | UI | M2-02, M2-07 |
| C7 | Config filter field | `Views/Config/ConfigFilterField.swift` | `FilterField.vue` | MVP | NS | NS | NS | — | UI | M2-07 |
| C8 | 20 colour themes, 3 adaptive | `Theme/PoirotTheme.swift`, `Views/Components/ThemePicker.swift` | `src/styles/tokens.css`, `ThemePicker.vue` | MVP | NS | NS | NS | Text colours become explicit per theme (C-9) | UI, FEAT | M2-02, M2-05, M3-16 |
| C9 | Font scale 0.75–1.5 in 0.05 steps | `App/AppState.swift:72-80, 428-440` | `src/stores/theme.ts`, `--font-scale` | MVP | NS | NS | NS | — | UI | M2-05, M2-08 |
| C10 | Markdown rendering and syntax highlighting | `Theme/PoirotMarkdownTheme.swift` (MarkdownUI, HighlightSwift) | `src/lib/markdown/` (markdown-it, DOMPurify, Shiki) | MVP | NS | NS | NS | RP: new libraries with a stricter sanitiser (ADR 0010) | SEC | M2-10 |
| C11 | Icons and symbol effects | SF Symbols across `Views/` | Lucide via `AppIcon.vue`, `src/styles/motion.css` | MVP | NS | NS | NS | RP: Lucide and CSS animations | UI | M2-04 |
| C12 | Keyboard navigation: arrows, Return, layered and input-aware Esc | `Views/Components/KeyboardNavigationModifier.swift` | `useKeyboardNav.ts` | MVP | NS | NS | NS | — | UI | M2-08, M3-19 |
| C13 | Back/forward navigation history | `App/AppState.swift:82-190` | Vue Router, `useNavigation.ts` | MVP | NS | NS | NS | — | UI | M2-06 |
| C14 | Reduced motion and the `showAnimations` switch | reduce-motion checks across `Views/` | `usePrefersReducedMotion.ts` | MVP | NS | NS | NS | — | UI | M2-04 |
| C15 | Localisation | `Poirot/Resources/Localizable.xcstrings` (English only, 401 strings) | English strings | MVP | NS | NS | NS | OOS: no i18n framework until a second language is wanted | — | — |
| C16 | Copy to clipboard (about 25 call sites) | `NSPasteboard` across `Views/` | clipboard-manager `writeText` | MVP | NS | NS | NS | RP: clipboard plugin | SEC, FEAT | M1-48, M2-17, M2-18, M3-02 |
| C17 | Reveal in Finder (3 call sites) | `ConfigDetailToolbar.swift:86`, `MCPServersListView.swift:258`, `SessionToolbarContent.swift:141` | `reveal_in_file_manager` | Later | NS | NS | NS | Linux DG: opens the parent folder | SHELL, RUST, FEAT | M3-52, M3-53, M3-55 |
| C18 | Copy diff as image | `Views/Session/EditDiffView.swift:220-250` | decided by spike | Later | NS | NS | NS | Open risk R-7 | FEAT | M3-37, M3-39 |

## S. Universal search

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| S1 | Category: Sessions (title, project, id, substring fallback) | `Views/Search/SearchOverlayView.swift:5-64` | `search/providers.rs` | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S2 | Category: Debug logs (existence) | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S3 | Category: AI summaries (facets) | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S4 | Category: File changes (capped at 50 folders) | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S5 | Category: TODOs | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S6 | Category: History | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S7 | Category: Commands | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S8 | Category: Skills | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S9 | Category: Plans | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S10 | Category: Memory (project memory only) | same, `loadConfigItems()` :1038-1080 | same | MVP | NS | NS | NS | Custom folders not searched, kept at parity [A-6] | RUST, FEAT | M1-30, M3-01 |
| S11 | Category: MCP servers | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S12 | Category: Plugins | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S13 | Category: Output styles | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S14 | Category: Hooks | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S15 | Category: Models (curated catalog only) | same | same | MVP | NS | NS | NS | Kept at parity [A-6] | RUST, FEAT | M1-30, M3-01 |
| S16 | Category: Sub-agents | same | same | MVP | NS | NS | NS | — | RUST, FEAT | M1-30, M3-01 |
| S17 | Quick action: Toggle Menu Bar Icon | same | `SearchOverlay.vue` | Later | NS | NS | NS | HD until the tray ships | FEAT | M3-01, M3-58 |
| S18 | Quick action: Add MCP Server | same | `SearchOverlay.vue` | Later | NS | NS | NS | HD until the wizard ships | FEAT | M3-01, M3-48 |
| S19 | Fuzzy and exact ("Match Word") modes, persisted | same, `Utilities/HighlightedText.swift` | `search/matcher.rs`, `globalSearchMode` | MVP | NS | NS | NS | — | RUST, FEAT | M1-29, M3-01 |
| S20 | Empty query: nav screens and 5 most recent sessions; max 5 per group | same | `SearchOverlay.vue`, `list_recent_sessions` | MVP | NS | NS | NS | — | FEAT | M1-41, M3-01 |

## E. Reads under `~/.claude` and `~/.claude.json`

| ID | Path pattern | Swift loader | Heimdall module | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| E1 | `projects/<encoded>/*.jsonl` | `Services/SessionLoader.swift`, `Services/TranscriptParser.swift` | `projects/`, `transcript/` | MVP | AP | AP | AP | Project path from `cwd` first (C-6) | RUST | M1-12, M1-13, M1-14 |
| E2 | `projects/<encoded>/sessions-index.json` | `SessionLoader.swift:214-245` | `projects/sessions_index.rs` | MVP | AP | AP | AP | Accepts `projectPath` and `originalPath` (C-7) | RUST | M1-14 |
| E3 | `projects/<encoded>/<sessionId>/subagents/agent-*.jsonl` and `.meta.json` | `SessionLoader.swift:249-301` | `projects/subagents.rs` | MVP | AP | AP | AP | — | RUST | M1-14 |
| E4 | `projects/<encoded>/memory/*.md` | `Services/Config/ClaudeConfigLoader.swift:421-451` | `config/read/memory.rs` | MVP | AP | AP | AP | — | RUST | M1-27 |
| E5 | `history.jsonl` | `Services/HistoryLoader.swift` | `data/history.rs` | MVP | AP | AP | AP | — | RUST | M1-15 |
| E6 | `todos/<sessionId>[-agent-<id>].json` | `Services/TodoLoader.swift` | `data/todos.rs` | MVP | AP | AP | AP | — | RUST | M1-16 |
| E7 | `debug/<sessionId>.txt` | `Services/DebugLogLoader.swift` | `data/debug_log.rs` | MVP | AP | AP | AP | — | RUST | M1-17 |
| E8 | `file-history/<sessionId>/<hash@vN>` | `Services/FileHistoryLoader.swift` | `data/file_history.rs` | MVP | AP | AP | AP | — | RUST | M1-19 |
| E9 | `usage-data/facets/<sessionId>.json` | `Services/FacetsLoader.swift` | `data/facets.rs` | MVP | AP | AP | AP | — | RUST | M1-18 |
| E10 | `stats-cache.json` (used only when not stale) | `Services/StatsCacheLoader.swift`, `Services/StatsComputer.swift` | `data/stats_cache.rs`, `data/stats.rs` | Later | AP | AP | AP | — | RUST | M1-20 |
| E11 | `commands/*.md`, global and project | `ClaudeConfigLoader.loadCommands` | `config/read/commands.rs` | MVP | AP | AP | AP | — | RUST | M1-24 |
| E12 | `skills/<name>/SKILL.md`, global and project | `ClaudeConfigLoader.loadSkills` | `config/read/skills.rs` | MVP | AP | AP | AP | — | RUST | M1-24 |
| E13 | `plans/*.md` | `ClaudeConfigLoader.loadPlans` | `config/read/plans.rs` | MVP | AP | AP | AP | — | RUST | M1-24 |
| E14 | `output-styles/*.md` except README, global and project | `ClaudeConfigLoader.loadOutputStyles` | `config/read/output_styles.rs` | MVP | AP | AP | AP | — | RUST | M1-24 |
| E15 | `agents/*.md` | `ClaudeConfigLoader.loadCustomAgents` | `config/read/agents.rs` | MVP | AP | AP | AP | — | RUST | M1-24 |
| E16 | `settings.json`, global and project (permissions, enabledPlugins, model, hooks) | `ClaudeConfigLoader.loadSettings`, `loadHooks` | `config/read/settings.rs`, `hooks.rs` | MVP | AP | AP | AP | — | RUST | M1-25 |
| E17 | `~/.claude.json` (`mcpServers`, `projects.<path>.mcpServers`) | `ClaudeConfigLoader.loadClaudeConfig` | `config/read/mcp.rs` | MVP | AP | AP | AP | Secrets held as `Secret` | RUST | M1-26 |
| E18 | `<project>/.mcp.json` | `ClaudeConfigLoader.swift:155-170` | `config/read/mcp.rs` | MVP | AP | AP | AP | — | RUST | M1-26 |
| E19 | `plugins/installed_plugins.json` | `ClaudeConfigLoader.loadPlugins`, `loadPluginMCPServers` | `config/read/plugins.rs` | MVP | AP | AP | AP | — | RUST | M1-26 |
| E20 | `mcp-needs-auth-cache.json` (keys only) | `Services/Config/MCPServerStatusChecker.swift:7-11`, `ClaudeConfigLoader.swift:302-331` | `config/read/mcp.rs`, `mcp_status.rs` | MVP | AP | AP | AP | — | RUST | M1-26, M1-28 |
| E21 | `<project>/.claude/settings.json` model | `ClaudeConfigLoader.loadProjectModel` | `config/read/settings.rs` | MVP | AP | AP | AP | — | RUST | M1-25 |
| E22 | User-chosen memory folders | `ClaudeConfigLoader.loadMemoryFiles(customFolder:)` | `config/read/memory.rs` | MVP | AP | AP | AP | — | RUST | M1-27 |

## F. Writes

Every write to a Claude Code file goes through the safe-write primitive (ADR 0009, task M1-22) and needs security-auditor sign-off.

| ID | Target and change | Swift reference | Heimdall module | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| F1 | `settings.json` `enabledPlugins.<key>` toggle or remove | `Services/Config/SettingsWriter.swift:14-32` | `config/write/settings.rs` | Later | NS | NS | NS | Fails closed on parse error (C-1) | RUST, FEAT | M1-22, M3-40, M3-44, M3-50 |
| F2 | Strip `mcp__<server>__*` from `permissions.allow` | `SettingsWriter.swift:34-47` | `config/write/settings.rs` | Later | NS | NS | NS | — | RUST | M3-40, M3-41 |
| F3 | `settings.json` default `model` | `SettingsWriter.swift:49-55` | `config/write/settings.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-40, M3-44, M3-50 |
| F4 | `<project>/.claude/settings.json` `model` | `SettingsWriter.swift:57-82` | `config/write/settings.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-40, M3-44, M3-50 |
| F5 | `hooks.<Event>` save and delete, either scope | `SettingsWriter.swift:198-253` | `config/write/settings.rs` | Later | NS | NS | NS | No silent fallback to the global file (C-2) | RUST, FEAT | M3-40, M3-44, M3-47 |
| F6 | Hooks import (append) | `SettingsWriter.swift:269-292` (no callers today) | `config/write/settings.rs` | Later | NS | NS | NS | Behind a native confirmation (ADR 0010) | RUST, FEAT | M3-40, M3-44, M3-47 |
| F7 | `~/.claude.json` `mcpServers.<name>` add, update, remove | `SettingsWriter.swift:115-197` | `config/write/mcp.rs` | Later | NS | NS | NS | Keeps type/url/headers (C-3) | RUST, FEAT | M3-41, M3-45, M3-48 |
| F8 | `agents/<slug>.md` save; rename deletes the old file | `ClaudeConfigLoader.swift:650-699` | `config/write/files.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-42, M3-44, M3-49 |
| F9 | Template scaffolds: new agent, command, skill, output style | `ClaudeConfigLoader.swift:700-818` | `config/write/files.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-42, M3-44, M3-50 |
| F10 | Sub-agent JSON import | `ClaudeConfigLoader.swift:735-757` | `config/write/files.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-42, M3-44, M3-49 |
| F11 | Delete config file (skill: parent folder) | `ClaudeConfigLoader.swift:819-829` | `config/write/files.rs` | Later | NS | NS | NS | Takes a ConfigRef, not a path | RUST, FEAT | M3-42, M3-44, M3-50 |
| F12 | `history.jsonl` remove one line or lines older than N days | `Views/History/HistoryListView.swift`, `Services/HistoryLoader.swift` | `data/history_write.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-43, M3-46, M3-51 |
| F13 | Delete project folder, session `.jsonl` and its todos | `App/AppState.swift:380-460` | `projects/delete.rs` | Later | NS | NS | NS | — | RUST, FEAT | M3-43, M3-46, M3-51 |
| F14 | User-chosen files: session Markdown and patch; analytics CSV and PNG | `Services/Export/SessionExporter.swift:280-346`, `AnalyticsCSVExporter.swift:82`, `AnalyticsImageExporter.swift:29` | `commands/export.rs` (Rust-opened dialogs) | MVP | NS | NS | NS | Analytics exports are Later | RUST | M3-09, M3-36, M3-39 |
| F15 | Preference `customMemoryFolders` | `App/MemorySourcesStore.swift` | `src-tauri/src/prefs.rs` (app config dir, never `~/.claude`) | MVP | NS | NS | NS | — | RUST | M1-39 |

## G. File watching and caching

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| G1 | Projects watcher | `App/ContentView.swift:112` | one recursive watcher, `src-tauri/src/watcher.rs` | MVP | NS | IP | IP | RP: recursive watch replaces 3+N single-folder watchers (ADR 0006). M1-59: real CI (first ever run of `tests/watcher.rs`) found modify/rename/delete on a pre-existing watched file undetected on Linux/macOS; Windows cannot run at all (`STATUS_ENTRYPOINT_NOT_FOUND`). Follow-up task needed | RUST | M1-47 |
| G2 | Facets watcher | `App/ContentView.swift:120` | same | MVP | NS | IP | IP | Same as G1 | RUST | M1-47 |
| G3 | Debug-log watcher | `App/ContentView.swift:424` | same; no longer creates `~/.claude/debug` | MVP | NS | IP | IP | Same as G1 | RUST | M1-47 |
| G4 | Memory watchers: projects, each memory folder, each custom folder | `Views/Memory/MemoryListView.swift:337-354` | same, re-registered on prefs change | MVP | NS | IP | IP | Same as G1; a folder newly registered via `set_custom_folders` after `init` also missed its first write in the same real run | RUST | M1-47 |
| G5 | Transcript cache invalidation | `App/AppState.swift:303`, `clearCache()` :370 (test-only caller) | `src-tauri/src/state.rs`, validated by (mtime, size) | MVP | NS | AP | AP | Fixes a stale-cache gap. The `Cache` data structure's own unit tests (hit/miss/clear) pass on Linux/macOS, crash on Windows (M1-59); full end-to-end invalidation still depends on G1's own open defect | RUST | M1-38 |
| G6 | Watch-limit fallback (inotify cap, ReadDirectoryChangesW overflow) | none | `watcher-status` event, per-folder fallback | MVP | NS | IP | IP | New behaviour. Same open defect as G1 | RUST | M1-47 |
| G7 | Per-screen watchers: `settings.json` (hooks), `~/.claude.json` and the MCP auth cache, `agents/`, `history.jsonl`, `plans/`, `todos/` | `Views/Config/HooksListView.swift:262-272`, `Views/Config/MCPServersListView.swift:118-136`, `Views/Config/SubAgentsListView.swift:96-106`, `Views/History/HistoryListView.swift:139-149`, `Views/Plans/PlansListView.swift:115-124`, `Views/Todos/TodosOverviewView.swift:113-123` | same recursive watcher; no folder is created just to watch it | MVP | NS | IP | IP | RP: one recursive watcher (C-22). Same open defect as G1 — a safe-write to a top-level file (`settings.json`) was detected correctly in the same real run, so this row's own failure mode is unconfirmed either way beyond G1's | RUST | M1-47 |

## H. Network

| ID | Feature | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| H1 | Check for updates (manual, GitHub releases API, semver compare) | `Services/UpdateChecker.swift`, `App/PoirotApp.swift:55-79` | updater plugin, `latest.json` on the fork's releases | MVP | NS | NS | NS | RP: signed updater (P-14) | SHELL, RUST, FEAT | M4-03, M4-04, M4-05 |
| H2 | External links: repo, Sponsors, social profile | `App/AboutView.swift` | `open_external_url`, fork links only | MVP | NS | NS | NS | RP: fork links; upstream author's personal links dropped | RUST, FEAT | M2-11, M3-17 |

## I. Preferences

All preferences live in one Rust-owned `Prefs` file in the OS app-config folder (ADR 0007).

| ID | Key | Type and default | Heimdall owner | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| I1 | `showMenuBarIcon` | Bool, true | tray | Later | NS | NS | NS | — | RUST, FEAT | M1-11, M3-58 |
| I2 | `hasCompletedOnboarding` | Bool, false | onboarding | MVP | NS | NS | NS | — | RUST, FEAT | M1-11, M3-15 |
| I3 | `colorTheme` | String, `default` | theme store | MVP | NS | NS | NS | — | UI | M1-11, M2-05, M3-16 |
| I4 | `fontScale` | Double, 1.0 (0.75–1.5) | theme store | MVP | NS | NS | NS | — | UI | M1-11, M2-05 |
| I5 | `sessionLayout` | String, grid | sessions | MVP | NS | NS | NS | — | FEAT | M1-11, M2-16 |
| I6 | `configProjectPath` | String or none | config | MVP | NS | NS | NS | Set only through a Rust folder picker | RUST, FEAT | M1-39, M3-11 |
| I7 | `configLayouts` | Map, {} | config | MVP | NS | NS | NS | — | FEAT | M1-11, M3-11 |
| I8 | `showAgentSessions` | Bool, false | sessions | MVP | NS | NS | NS | — | FEAT | M1-11, M2-15 |
| I9 | `textEditor` | String, vscode | launchers | MVP | NS | NS | NS | Options filtered per OS; Xcode is macOS only | RUST, FEAT | M1-11, M3-16, M3-55 |
| I10 | `preferredTerminal` | String, per-OS default | launchers | MVP | NS | NS | NS | Per-OS option sets (ADR 0003) | RUST, FEAT | M1-11, M3-16, M3-55 |
| I11 | `openTerminalOnBash` | Bool, false | tool block | Later | NS | NS | NS | — | FEAT | M1-11, M3-55 |
| I12 | `claudeCodePath` | String, `/usr/local/bin/claude` today | onboarding | MVP | NS | NS | NS | RP: default none, found by PATH search | RUST | M1-11, M1-31 |
| I13 | `showAnimations` | Bool, true | motion | MVP | NS | NS | NS | — | UI | M1-11, M2-04 |
| I14 | `wrapCodeLines` | Bool, true | blocks | MVP | NS | NS | NS | — | FEAT | M1-11, M2-18 |
| I15 | `autoExpandBlocks` | Bool, true | blocks | MVP | NS | NS | NS | — | FEAT | M1-11, M2-17, M2-18 |
| I16 | `parseMarkdownInResults` | Bool, true | blocks | MVP | NS | NS | NS | — | FEAT | M1-11, M2-18 |
| I17 | `globalSearchMode` | String, fuzzy | search | MVP | NS | NS | NS | — | FEAT | M1-11, M3-01 |
| I18 | `customMemoryFolders` | [String], [] | memory | MVP | NS | NS | NS | — | RUST, FEAT | M1-39, M3-04 |
| I19 | Launch at login (read live, not stored) | `App/SettingsView.swift:320-350` (SMAppService) | autostart plugin | Later | NS | NS | NS | RP: Windows Run key, macOS LaunchAgent, Linux XDG autostart | SEC, FEAT | M3-59, M3-60 |

## J. Keyboard shortcuts

On Windows and Linux, ⌘ becomes Ctrl everywhere, including labels.

| ID | Shortcut | Action | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| J1 | ⌘K | Toggle universal search | `useShortcuts` | MVP | NS | NS | NS | — | UI, FEAT | M2-08, M3-01, M3-19 |
| J2 | ⌘[ / ⌘] | Back / forward | `useShortcuts`, `useNavigation` | MVP | NS | NS | NS | — | UI | M2-06, M2-08, M3-19 |
| J3 | ⌘T | Toggle tool filter (session selected) | `useShortcuts`, `SessionDetail.vue` | MVP | NS | NS | NS | — | UI, FEAT | M2-08, M2-20, M3-19 |
| J4 | ⌘+ / ⌘− / ⌘0 | Font size up, down, reset | `useShortcuts`, theme store | MVP | NS | NS | NS | — | UI | M2-05, M2-08, M3-19 |
| J5 | ⌘? | Open Help | `useShortcuts` | MVP | NS | NS | NS | — | UI, FEAT | M2-08, M3-17 |
| J6 | Esc | Layered dismiss | `useKeyboardNav` | MVP | NS | NS | NS | — | UI | M2-08, M3-19 |
| J7 | ↑ / ↓ | Move selection in search results | `SearchOverlay.vue` | MVP | NS | NS | NS | — | FEAT | M3-01, M3-19 |
| J8 | Return | Open selected search result | `SearchOverlay.vue` | MVP | NS | NS | NS | — | FEAT | M3-01, M3-19 |
| J9 | ← / → | Onboarding page navigation | `Onboarding.vue` | MVP | NS | NS | NS | — | FEAT | M3-15, M3-19 |
| J10 | ⌘. / Return | Cancel / save in hook, MCP and sub-agent forms | form components | Later | NS | NS | NS | — | FEAT | M3-47, M3-48, M3-49 |
| J11 | ⌘, | Settings | `useShortcuts`, macOS menu | MVP | NS | NS | NS | — | UI, SHELL | M2-08, M2-13, M3-16 |

## K. Analytics and cost computation

| ID | Rule | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| K1 | `stats-cache.json` is used only when it covers the latest session day | `Services/StatsCacheLoader.swift:22-51` | `data/stats_cache.rs` | Later | NS | NS | NS | — | RUST | M1-20 |
| K2 | Local recomputation over main-thread sessions, always `costUSD: 0` | `Services/StatsComputer.swift` | `data/stats.rs` | Later | NS | NS | NS | — | RUST | M1-20, M1-53 |
| K3 | Context ring from the latest turn's tokens | `Models/ContextUsage.swift` | `models/context_usage.rs` | MVP | NS | NS | NS | — | RUST, FEAT | M1-08, M2-21 |
| K4 | Model display names: curated list merged with discovered ids | `Models/ClaudeModelCatalog.swift` | `models/catalog/claude_models.rs`, `config/read/models.rs` | MVP | NS | NS | NS | Needs manual updates as models ship | RUST | M1-10, M1-27 |
| K5 | API-only message for subscription users | `Views/Analytics/AnalyticsDashboardView.swift` | `AnalyticsDashboard.vue` | Later | NS | NS | NS | — | FEAT | M3-32 |

## L. Platform-specific code

| ID | Item | Swift reference | Heimdall target | Scope | Win | Linux | macOS | Outcome | WS | Tasks |
|---|---|---|---|---|---|---|---|---|---|---|
| L1 | MCP process detection (`pgrep -f`) | `Services/Config/MCPServerStatusChecker.swift:216-236` | `mcp_status.rs` on `sysinfo` | MVP | NS | NS | NS | RP: sysinfo process list | RUST | M1-28 |
| L2 | Terminal launching (AppleScript, bundle ids) | `Utilities/TerminalLauncher.swift` | `os_integration/launch.rs`, `platform/` | Later | NS | NS | NS | RP: per-OS argv; copy command and open at cwd | RUST, SHELL, FEAT | M1-32, M3-52, M3-53, M3-55 |
| L3 | Editor launching (bundle ids, Homebrew PATH) | `Utilities/EditorLauncher.swift` | `os_integration/launch.rs`, `platform/` | Later | NS | NS | NS | RP: editor URL schemes or validated argv; PATH search | RUST, SHELL, FEAT | M1-31, M1-32, M3-52, M3-53, M3-54, M3-55 |
| L4 | Save and open panels (8 call sites) | `NSSavePanel`, `NSOpenPanel` across `App/`, `Services/`, `Views/` | dialog plugin, opened from Rust | MVP | NS | NS | NS | RP: dialog plugin | RUST | M1-39, M3-09, M3-36, M3-44 |
| L5 | Sub-agent JSON import/export file filters | `Views/Config/SubAgentsListView.swift:270-300` | dialog filters (`.json`) | Later | NS | NS | NS | RP: dialog filter | RUST, FEAT | M3-44, M3-49 |
| L6 | Onboarding CLI detection (hard-coded macOS paths) | `Views/Onboarding/OnboardingView.swift:281-297` | `os_integration/cli_locate.rs` | MVP | NS | NS | NS | RP: PATH search with PATHEXT | RUST | M1-31, M3-15 |
| L7 | Loading off the main thread (`Task.detached` in views) | `Views/Search/SearchOverlayView.swift:1038-1080`, `App/AppState.swift:247-268` | `spawn_blocking` in commands | MVP | NS | NS | NS | — | RUST | M1-38 |

## X. Out of scope

| ID | Item | Reason | Decision |
|---|---|---|---|
| X1 | Usage Limits (subscription rate-limit gauges) | Removed upstream in `68b7b0c`; README.md and SHOWCASE.md are stale | User decision 4 |
| X2 | Provider abstraction (`ProviderDescribing`) | One implementation only | User decision 3, ADR 0008 |
| X3 | Homebrew Cask, winget, Scoop, AUR, Flathub | GitHub Releases only at launch | User decision 7 |
| X4 | Windows Authenticode, Apple Developer ID signing, notarisation | Unsigned builds for now | User decision 7 |
| X5 | Intel macOS; Windows and Linux ARM64 | Apple Silicon only; x64 only at launch | User decisions 8 and 9 |
| X6 | i18n framework | Poirot ships English only | Plan (C15) |
