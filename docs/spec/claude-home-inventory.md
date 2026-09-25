# Inventory: every `~/.claude` and `~/.claude.json` path Poirot touches

Task M0-05. Source: `$POIROT/Poirot/Sources/` at commit `a29dd6e` (all paths below are relative to it). "Owner" is the Heimdall module that takes over the path (ADR 0002).

`~` is the user's home: `homeDirectoryForCurrentUser` on macOS; on Windows Claude Code uses `%USERPROFILE%` [A-1]. `<enc>` is a project folder under `~/.claude/projects/`, named by replacing path separators and other non-alphanumerics with `-` (for example `-Users-me-src-app`, or `c--Users-pc-src-app` on Windows).

## 1. Reads

| # | Path | Format | Fields consumed | Swift reader (file:line) | Heimdall owner |
|---|---|---|---|---|---|
| R1 | `~/.claude/projects/` (list) | directory | child folders only | `Services/SessionLoader.swift:12-56` (`discoverProjects`, `projectDirectoryURLs`) | `projects/discovery.rs` |
| R2 | `~/.claude/projects/<enc>/*.jsonl` (UUID-named, plus `agent-*`) | JSONL | `type`, `message.{role,content,model,usage,id}`, `timestamp` (ISO-8601, fractional seconds), `isSidechain`, `uuid`; blocks `text`, `thinking`, `tool_use` (`id`, `name`, `input`), `tool_result` (`tool_use_id`, `content`, `is_error`); `usage.{input_tokens,output_tokens,cache_read_input_tokens,cache_creation_input_tokens}` | `Services/TranscriptParser.swift:10-139` (full), `:140-234` (header, 8 KiB chunks, 50 lines), `:235-354` (summary) | `transcript/` |
| R3 | `~/.claude/projects/<enc>/sessions-index.json` | JSON | `entries[].{sessionId, created, isSidechain, projectPath, firstPrompt, agentId}`; top-level `projectPath` | `Services/SessionLoader.swift:214-245` | `projects/sessions_index.rs` |
| R4 | same file, second reader | JSON | top-level `originalPath` only | `Views/Todos/TodosOverviewView.swift:368-376` | `projects/sessions_index.rs` (discrepancy D-1) |
| R5 | `~/.claude/projects/<enc>/<sessionId>/subagents/agent-*.jsonl` (excluding `agent-acompact-*`) | JSONL | as R2 | `Services/SessionLoader.swift:249-294` | `projects/subagents.rs` |
| R6 | `~/.claude/projects/<enc>/<sessionId>/subagents/agent-*.meta.json` | JSON | `agentType`, `description` | `Services/SessionLoader.swift:296-301` | `projects/subagents.rs` |
| R7 | Real project folders on disk (probing) | directory existence | — | `Services/SessionLoader.swift:314-353` (`decodeProjectPath`, display name only) | `paths.rs` (last-resort fallback) |
| R8 | `~/.claude/projects/<enc>/memory/*.md` | Markdown | whole file; `MEMORY.md` sorts first | `Services/Config/ClaudeConfigLoader.swift:421-451`; counts `:463-487`, `:519-532` | `config/read/memory.rs` |
| R9 | User-chosen memory folders (outside `~/.claude`) | Markdown | whole file | `Services/Config/ClaudeConfigLoader.swift:489-517` | `config/read/memory.rs` |
| R10 | `~/.claude/history.jsonl` | JSONL | `display`, `pastedContents`, `timestamp` (ms epoch), `project` | `Services/HistoryLoader.swift:16-57`, `:117-130` | `data/history.rs` |
| R11 | `~/.claude/todos/<sessionId>[-agent-<agentId>].json` | JSON array | `content`, `status` (`pending`, `in_progress`, `completed`), `activeForm` | `Services/TodoLoader.swift:17-76` | `data/todos.rs` |
| R12 | `~/.claude/todos/` (list, cross-project) | directory | file names | `Views/Todos/TodosOverviewView.swift:115-122` (watch), `Services/TodoLoader.swift:51-76` | `data/todos.rs` |
| R13 | `~/.claude/debug/<sessionId>.txt` | text | `<ISO8601> [LEVEL] message`, multi-line entries | `Services/DebugLogLoader.swift:18-126` (paged `:36-55`), existence `:57-62`, ids `:64-80` | `data/debug_log.rs` |
| R14 | `~/.claude/file-history/<sessionId>/<hash>@v<N>` | raw file content | whole file | `Services/FileHistoryLoader.swift:57-62` | `data/file_history.rs` |
| R15 | Transcript records `file-history-snapshot` (`snapshot.trackedFileBackups`) and `file-history-delta` (`trackingPath`, `backup`) | JSONL | `backupFileName`, `version`, `backupTime` | `Services/FileHistoryLoader.swift:88-165` (finds the transcript by scanning `projects/*/<sessionId>.jsonl`, `:93-110`) | `data/file_history.rs` |
| R16 | `~/.claude/file-history/` (list, capped at 50 folders) | directory | session ids | `Views/Search/SearchOverlayView.swift:1083-1110` | `search/providers.rs` |
| R17 | `~/.claude/usage-data/facets/<sessionId>.json` | JSON | `session_id`, `underlying_goal`, `goal_categories`, `outcome`, `user_satisfaction_counts`, `claude_helpfulness`, `session_type`, `friction_counts`, `friction_detail`, `primary_success`, `brief_summary` | `Services/FacetsLoader.swift:14-51` | `data/facets.rs` |
| R18 | `~/.claude/stats-cache.json` | JSON | `version`, `lastComputedDate`, `dailyActivity[]`, `dailyModelTokens[]`, `modelUsage{}` (incl. `costUSD`), `totalSessions`, `totalMessages`, `longestSession`, `firstSessionDate`, `hourCounts`, `totalSpeculationTimeSavedMs` | `Services/StatsCacheLoader.swift:11-51` | `data/stats_cache.rs` |
| R19 | `~/.claude/projects/*/*.jsonl` (full rescan when the cache is stale) | JSONL | timestamps, usage, model, tool calls, `isSidechain` | `Services/StatsComputer.swift:11-89`, `:122` | `data/stats.rs` |
| R20 | `~/.claude/commands/*.md` and `<project>/.claude/commands/*.md` | Markdown + frontmatter | `description`, `model`, `argument-hint`, `allowed-tools`, `output-style`, body | `Services/Config/ClaudeConfigLoader.swift:10-49` | `config/read/commands.rs` |
| R21 | `~/.claude/plans/*.md` | Markdown | file name → humanised name, body | `Services/Config/ClaudeConfigLoader.swift:51-72` | `config/read/plans.rs` |
| R22 | `~/.claude/skills/<name>/SKILL.md` and project equivalent | Markdown + frontmatter | `name`, `description`, `model`, `allowed-tools`, body | `Services/Config/ClaudeConfigLoader.swift:74-110` | `config/read/skills.rs` |
| R23 | `~/.claude.json` | JSON | top-level `mcpServers`; `projects.<path>.mcpServers` | `Services/Config/ClaudeConfigLoader.swift:111-153` | `config/read/mcp.rs` |
| R24 | `<project>/.mcp.json` | JSON | `mcpServers` | `Services/Config/ClaudeConfigLoader.swift:155-170` | `config/read/mcp.rs` |
| R25 | `~/.claude/plugins/installed_plugins.json` | JSON | `plugins{"<name>@<author>": [{version, installedAt}]}` | `Services/Config/ClaudeConfigLoader.swift:212-300` (plugin MCP servers), `:349-380` (plugins list) | `config/read/plugins.rs`, `config/read/mcp.rs` |
| R26 | `~/.claude/mcp-needs-auth-cache.json` | JSON (keys only) | key set; keys prefixed `claude.ai ` are cloud integrations | `Services/Config/MCPServerStatusChecker.swift:7-11, 89-96`; `Services/Config/ClaudeConfigLoader.swift:302-331` | `config/read/mcp.rs`, `mcp_status.rs` |
| R27 | `~/.claude/output-styles/*.md` (excluding `README.md`) and project equivalent | Markdown + frontmatter | `name`, `description`, body | `Services/Config/ClaudeConfigLoader.swift:382-417` | `config/read/output_styles.rs` |
| R28 | `~/.claude/settings.json` | JSON | `permissions.{allow,deny}` (tool permissions per MCP server, `:333-347`), `enabledPlugins`, `model`, `hooks` | `Services/Config/ClaudeConfigLoader.swift:534-540`, `:554-613`; raw dict `Services/Config/SettingsWriter.swift:359-364` | `config/read/settings.rs`, `config/read/hooks.rs` |
| R29 | `<project>/.claude/settings.json` | JSON | `model`, `hooks` | `Services/Config/ClaudeConfigLoader.swift:542-552`, `:554-613` | `config/read/settings.rs` |
| R30 | `~/.claude/agents/*.md` | Markdown + frontmatter | `name`, `description`, `tools`, `model`, `color`, `memory`, body | `Services/Config/ClaudeConfigLoader.swift:614-648` | `config/read/agents.rs` |
| R31 | `~/.claude.json`, line scan | text | line number of a server key inside top-level `mcpServers` | `Services/Config/SettingsWriter.swift:84-113` | `config/write/mcp.rs` (line lookup) |
| R32 | Picked project folder: `<folder>/.claude/` existence | directory | — | `Views/Config/ConfigProjectPicker.swift:55-70`; stored path re-checked at `App/AppState.swift:223-227` | `commands/prefs.rs` (`pick_config_project`) |
| R33 | `~/.claude/projects/` non-empty (onboarding) | directory | — | `Views/Onboarding/OnboardingView.swift:291-297` | `commands/app.rs` (`detect_claude_cli`) |
| R34 | Sub-agent JSON import file (user-chosen) | JSON | agent fields | `Views/Config/SubAgentsListView.swift:286-300`, `Services/Config/ClaudeConfigLoader.swift:735-757` | `config/write/files.rs` |

## 2. Writes

Every write below uses `try?`, so failures are silent (ADR 0005). JSON writes use `.prettyPrinted, .sortedKeys, .withoutEscapingSlashes` and `.atomic`.

| # | Target | Change | Swift writer (file:line) | Falls back to an empty document on parse failure? | Heimdall owner |
|---|---|---|---|---|---|
| W1 | `~/.claude/settings.json` | `enabledPlugins.<key>` set or removed | `Services/Config/SettingsWriter.swift:16-32` via `loadSettingsDict` `:359-364` | **Yes** | `config/write/settings.rs` |
| W2 | `~/.claude/settings.json` | remove `mcp__<server>__*` from `permissions.allow` | `SettingsWriter.swift:36-47` | **Yes** (via `loadSettingsDict`) | `config/write/settings.rs` |
| W3 | `~/.claude/settings.json` | `model` | `SettingsWriter.swift:51-55` | **Yes** | `config/write/settings.rs` |
| W4 | `<project>/.claude/settings.json` | `model` set or removed; creates `.claude/` | `SettingsWriter.swift:59-82` | **Yes** (`:65-68`) | `config/write/settings.rs` |
| W5 | `settings.json`, global or project | `hooks.<Event>[i]` save | `SettingsWriter.swift:200-223` via `loadSettingsDictFor` `:330-339` | **Yes**; project scope with no path writes the **global** file (`:318-326`) | `config/write/settings.rs` |
| W6 | same | `hooks.<Event>[i]` delete | `SettingsWriter.swift:227-253` | **Yes** | `config/write/settings.rs` |
| W7 | same | hooks appended from imported JSON | `SettingsWriter.swift:269-292` (no callers) | **Yes** | `config/write/settings.rs` |
| W8 | `~/.claude.json` | add `mcpServers.<name>` (`command`, `args`, `env`) | `SettingsWriter.swift:117-146` | **Yes** (`:123-128`) | `config/write/mcp.rs` |
| W9 | `~/.claude.json` | update `mcpServers.<name>` (rename allowed; drops `type`, `url`, `headers`) | `SettingsWriter.swift:148-177` | No (returns early) | `config/write/mcp.rs` |
| W10 | `~/.claude.json` | remove `mcpServers.<name>` | `SettingsWriter.swift:181-197` | No (returns early) | `config/write/mcp.rs` |
| W11 | `~/.claude/agents/<slug>.md` | frontmatter + body; renaming deletes the old file (`:694`) | `Services/Config/ClaudeConfigLoader.swift:650-699` | n/a (whole file) | `config/write/files.rs` |
| W12 | `~/.claude/agents/new-agent[-N].md` | template | `ClaudeConfigLoader.swift:700-717` | n/a | `config/write/files.rs` |
| W13 | `~/.claude/commands/new-command[-N].md` | template | `ClaudeConfigLoader.swift:758-774` | n/a | `config/write/files.rs` |
| W14 | `~/.claude/skills/new-skill[-N]/SKILL.md` | template (creates folder) | `ClaudeConfigLoader.swift:776-801` | n/a | `config/write/files.rs` |
| W15 | `~/.claude/output-styles/new-style[-N].md` | template | `ClaudeConfigLoader.swift:803-817` | n/a | `config/write/files.rs` |
| W16 | `~/.claude/agents/<slug>.md` from JSON | import | `ClaudeConfigLoader.swift:735-757` → `saveAgent` | n/a | `config/write/files.rs` |
| W17 | `~/.claude/history.jsonl` | remove one line **by line index** parsed from the entry id | `Services/HistoryLoader.swift:60-77` | n/a; see discrepancy D-5 | `data/history_write.rs` |
| W18 | `~/.claude/history.jsonl` | remove lines older than N days | `Services/HistoryLoader.swift:82-105` | n/a; see D-5 | `data/history_write.rs` |
| W19 | `~/.claude/agents/` | folder created just to watch it | `Views/Config/SubAgentsListView.swift:98-105` | n/a | none: Heimdall never creates folders to watch them (ADR 0006) |
| W20 | `~/.claude/debug/` | folder created just to watch it | `App/ContentView.swift:413-429` | n/a | none (ADR 0006) |
| W21 | `~/.claude/commands/`, `agents/`, `output-styles/`, `skills/` | folders created before template writes | `ClaudeConfigLoader.swift:830-832` (`ensureDirectory`) | n/a | `config/write/files.rs` |
| W22 | User-chosen export files (Markdown, patch, CSV ×4, PNG, sub-agent JSON) | whole file | `Services/Export/SessionExporter.swift:284-315`, `Views/Analytics/Export/AnalyticsCSVExporter.swift:80-90`, `Views/Analytics/Export/AnalyticsImageExporter.swift:26-41`, `Views/Config/SubAgentsListView.swift:270-285` | n/a | `commands/export.rs`, `config/write/files.rs` |
| W23 | Preference `customMemoryFolders` (UserDefaults, outside `~/.claude`) | string array | `App/MemorySourcesStore.swift:40-42` | n/a | `src-tauri/src/prefs.rs` |

## 3. Deletes

| # | Target | Swift (file:line) | Heimdall owner |
|---|---|---|---|
| X1 | `~/.claude/projects/<enc>/` (whole project) | `App/AppState.swift:379-396` (`deleteProject`) | `projects/delete.rs` |
| X2 | `~/.claude/projects/<enc>/<sessionId>.jsonl` | `App/AppState.swift:441-447` (`deleteSession`) | `projects/delete.rs` |
| X3 | `~/.claude/todos/<sessionId>.json` and `<sessionId>-agent-*.json` | `Services/TodoLoader.swift:80-101` | `projects/delete.rs` |
| X4 | Config file (or the parent folder for `SKILL.md`) | `Services/Config/ClaudeConfigLoader.swift:819-829` (`deleteConfigFile(at:)`, takes a raw path) | `config/write/files.rs` (takes a `ConfigRef`) |
| X5 | Old agent file on rename | `Services/Config/ClaudeConfigLoader.swift:694` | `config/write/files.rs` |

## 4. Watchers

All use `Services/FileWatcher.swift`: one folder or file, non-recursive, `.write, .rename, .delete, .extend` (`:29`), 1 s debounce (`:58-67`).

| # | Watched path | Call site | Effect | Heimdall area (ADR 0006) |
|---|---|---|---|---|
| F1 | `~/.claude/projects` | `App/ContentView.swift:110-116` | full project rescan | `projects`, `sessions`, `transcript` |
| F2 | `~/.claude/usage-data/facets` (only if it exists) | `App/ContentView.swift:118-130` | same full rescan | `facets` |
| F3 | `~/.claude/debug` (created if missing) | `App/ContentView.swift:413-429` | same full rescan | `debugLogs` |
| F4 | `~/.claude/projects` | `Views/Memory/MemoryListView.swift:337-339` | reload memory sources | `memory` |
| F5 | each `~/.claude/projects/<enc>/memory` | `Views/Memory/MemoryListView.swift:342-350` | reload memory files | `memory` |
| F6 | each user-chosen memory folder | `Views/Memory/MemoryListView.swift:353-356` | reload memory files | `memory` |
| F7 | `~/.claude/settings.json` (file) | `Views/Config/HooksListView.swift:262-272` | reload hooks | `settings`, `hooks` |
| F8 | `~/.claude.json` (file) | `Views/Config/MCPServersListView.swift:118-128` | reload servers | `mcp` |
| F9 | `~/.claude/mcp-needs-auth-cache.json` (file) | `Views/Config/MCPServersListView.swift:130-136` | reload statuses | `mcp` |
| F10 | `~/.claude/agents` (created if missing) | `Views/Config/SubAgentsListView.swift:96-106` | reload custom agents | `agents` |
| F11 | `~/.claude/history.jsonl` (file) | `Views/History/HistoryListView.swift:139-149` | reload history | `history` |
| F12 | `~/.claude/plans` | `Views/Plans/PlansListView.swift:115-124` | reload plans | `plans` |
| F13 | `~/.claude/todos` | `Views/Todos/TodosOverviewView.swift:113-123` | reload todos | `todos` |

Not watched by Poirot but covered by Heimdall's recursive watcher: `commands/`, `skills/`, `output-styles/`, `plugins/`, `stats-cache.json` and `file-history/`. Project-level `.claude/` folders and `.mcp.json` files sit outside `~/.claude`; Heimdall re-reads them when a screen opens instead of watching them.

## 5. Other home-directory uses (not file access)

| Site | Use | Heimdall |
|---|---|---|
| `Utilities/EditorLauncher.swift:50-58` | expands `~` in launch paths | `os_integration/launch.rs` works on absolute canonical paths only |
| `Views/Onboarding/OnboardingView.swift:281-289` | probes `/usr/local/bin/claude`, `/opt/homebrew/bin/claude`, `~/.npm/bin/claude`, `~/.local/bin/claude` | `os_integration/cli_locate.rs` (PATH search) |
| `App/MemorySourcesStore.swift:46-48` | preview-only UserDefaults suite `fyi.poirot.preview.memory` | none |

Every hit of `grep -rn -E "homeDirectoryForCurrentUser|NSHomeDirectory" Poirot/Sources` at `a29dd6e` is covered above: `App/AppState.swift:375` (X1), `Services/Config/ClaudeConfigLoader.swift:5, 113` (R20–R30), `Services/Config/MCPServerStatusChecker.swift:8` (R26), `Services/Config/SettingsWriter.swift:5, 11` (W1–W10), `Services/DebugLogLoader.swift:185` (R13), `Services/FacetsLoader.swift:54` (R17), `Services/FileHistoryLoader.swift:67, 72` (R14, R15), `Services/HistoryLoader.swift:133` (R10), `Services/SessionLoader.swift:66` (R1), `Services/StatsCacheLoader.swift:6` (R18), `Services/TodoLoader.swift:114` (R11), `Utilities/EditorLauncher.swift:54` (§5), `Views/History/HistoryListView.swift:141` (F11), `Views/Memory/MemoryListView.swift:326` (F4–F6), `Views/Onboarding/OnboardingView.swift:285, 286, 292` (§5, R33), `Views/Plans/PlansListView.swift:117` (F12), `Views/Search/SearchOverlayView.swift:1087` (R16), `Views/Todos/TodosOverviewView.swift:115` (F13).

## 6. Cross-reference to the parity matrix

| Parity row | Inventory rows |
|---|---|
| E1 | R1, R2 |
| E2 | R3, R4 |
| E3 | R5, R6 |
| E4 | R8 |
| E5 | R10 |
| E6 | R11, R12 |
| E7 | R13 |
| E8 | R14, R15, R16 |
| E9 | R17 |
| E10 | R18, R19 |
| E11 | R20 |
| E12 | R22 |
| E13 | R21 |
| E14 | R27 |
| E15 | R30 |
| E16 | R28 |
| E17 | R23, R31 |
| E18 | R24 |
| E19 | R25 |
| E20 | R26 |
| E21 | R29 |
| E22 | R9 |
| F1 | W1 |
| F2 | W2 |
| F3 | W3 |
| F4 | W4 |
| F5 | W5, W6 |
| F6 | W7 |
| F7 | W8, W9, W10 |
| F8 | W11, X5 |
| F9 | W12, W13, W14, W15, W21 |
| F10 | W16, R34 |
| F11 | X4 |
| F12 | W17, W18 |
| F13 | X1, X2, X3 |
| F14 | W22 |
| F15 | W23 |
| G1 | F1 |
| G2 | F2 |
| G3 | F3, W20 |
| G4 | F4, F5, F6 |
| G5 | (in-memory cache, `App/AppState.swift:303, 370-372`) |
| G6 | (new in Heimdall) |
| G7 | F7, F8, F9, F10, F11, F12, F13, W19 |

## 7. Open discrepancies

| # | Discrepancy | Evidence | Resolution |
|---|---|---|---|
| D-1 | Two readers of `sessions-index.json` use different keys for the project path | `TodosOverviewView.swift:372` reads top-level `originalPath`; `SessionLoader.swift:239-240` reads `entries[0].projectPath` or top-level `projectPath` | Heimdall accepts both (M1-14); the Windows and Linux captures (M0-14, M0-15) record which key Claude Code writes |
| D-2 | Poirot never reads the transcript `cwd` field | no `cwd` in `Poirot/Sources` | Heimdall resolves project paths from `cwd` first (ADR 0003); fixtures must contain it |
| D-3 | Folder-name fallbacks turn every `-` into `/` | `SessionLoader.swift:98`, `TodosOverviewView.swift:375` | Retired; decoding is the last resort only (M1-14, M3-05) |
| D-4 | Poirot creates `~/.claude/debug` and `~/.claude/agents` only so it can watch them | W19, W20 | Heimdall never creates folders to watch them (ADR 0006) |
| D-5 | History delete finds the line by the index embedded in the entry id and rewrites the file; lines Claude Code appends between the read and the write are lost, and an index can point at the wrong line after any change | `HistoryLoader.swift:60-77`, `:82-105` | Heimdall ids hash the line content, and the rewrite refuses if the file changed since it was read (M1-15, M3-43) |
| D-6 | The brief lists 4 watcher call sites; there are 13 watcher start sites in 8 files | §4 | Parity row G7 added; one recursive watcher replaces them (M1-47) |
| D-7 | Every write swallows errors and several fall back to an empty document | §2 | ADR 0005, ADR 0009, M1-22 |
