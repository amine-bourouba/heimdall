# Behaviour spec: configuration screens, forms and the MCP wizard

Status: **Spec, M0-09.** Describes Poirot's eight configuration screens (`Views/Config/*.swift`) so Heimdall's Vue equivalents can be built without re-reading Swift. Every claim below cites `File.swift:line` in `$POIROT/Poirot/Sources/…`; paths are given relative to that `Sources/` root (for example `Views/Config/CommandsListView.swift:27`). Read-only behaviour targets M1 (`docs/ipc-contract.md` §6, `Scope: MVP`); mutating behaviour targets M3 (`Scope: Later`), gated by the safe-write primitive (ADR 0009) and the native-confirmation rule (ADR 0010 rule 6).

Poirot's `ConfigScope` is `global` (`~/.claude/`) or `project` (`<project>/.claude/`) — `Models/Config/ConfigScope.swift:4-7`. The IPC contract's `ConfigKind` (`command | skill | plan | outputStyle | agent`) and `ProjectContext` play the same role for Heimdall (`docs/ipc-contract.md` §4).

## 1. Shared building blocks

Six components are reused by every list screen; they are documented once here instead of per screen.

### 1.1 Screen header (`ConfigScreenHeader.swift:3-57`)

Icon (from the sidebar `ConfigurationItem`) in a tinted rounded square, title, an item-count pill (`dynamicCount` when supplied, else the item's static `count` string), and a one-line description. Not itself filterable or actionable; purely static per-load content, re-rendered whenever the screen's `@State` array changes.

### 1.2 Layout toolbar (`ConfigLayoutToolbar.swift:3-48`)

A native `ToolbarContent`, present in the window chrome (not in the scroll body) on every list screen:

- **Project picker** (`ConfigProjectPicker`, §1.3) — shown only when `showProjectPicker: true`. Used by Commands, Skills, Output Styles, Hooks, MCP Servers (`CommandsListView.swift:145-151`, `HooksListView.swift:81-89`, `MCPServersListView.swift:74-81`). **Not** shown for Models, Plugins, Sub-agents (`ModelsListView.swift:55-56`, `PluginsListView.swift:60-61`, `SubAgentsListView.swift:68-84`) — Models reads/writes project scope through its own inline banner and per-card buttons instead (§7); Plugins and Sub-agents have no project scope at all.
- **Filter field** (`ConfigFilterField`, width 200) — free-text, bound to the screen's `filterQuery` state; placeholder is screen-specific ("Find in Commands…", etc., `CommandsListView.swift:148`, `SkillsListView.swift:136`, `OutputStylesListView.swift:134`, `HooksListView.swift:85`, `MCPServersListView.swift:77`, `ModelsListView.swift:56`, `PluginsListView.swift:61`, `SubAgentsListView.swift:72`). Filtering logic is documented per screen (§2–§9) since the matched fields differ; all screens share one pattern — trim whitespace, fuzzy-match (`HighlightedText.fuzzyMatch`) against 1–2 fields, keep items with the max score `> 0`, sort by score descending (e.g. `CommandsListView.swift:27-40`).
- **Layout toggle** — a single button that flips the screen's persisted layout (`list` ↔ `grid`) via `appState.toggleConfigLayout(for: screenID)` (`ConfigLayoutToolbar.swift:27-36`); icon and animation reflect the *next* state. Present on all eight screens.
- **Add button** (`+`) — shown only when `showAddButton: true`. Present on Commands, Skills, Output Styles, Hooks, MCP Servers, Sub-agents; **absent** on Models and Plugins (`ModelsListView.swift:55-56`, `PluginsListView.swift:60-61` pass no `showAddButton`) because neither screen creates new items in place — Models only toggles defaults, Plugins are installed externally (`claude plugin add` or files under `~/.claude/plugins/`, `PluginsListView.swift:158`). Tapping `+` sets `appState.configAddTrigger = UUID()` (`ConfigLayoutToolbar.swift:39-45`), which every screen's `.onChange(of: appState.configAddTrigger)` observes to start its own add flow (§2–§9).

### 1.3 Project picker (`ConfigProjectPicker.swift:3-76`)

Folder icon (green when a project is picked, else tertiary), the picked folder's name or "No project selected", an "×" to clear (`appState.configProjectPath = nil`), and "Browse…" which opens an `NSOpenPanel` restricted to directories (`ConfigProjectPicker.swift:54-59`). After picking, if the folder has no `.claude/` subdirectory a toast warns "No .claude/ directory found in **name**" (`ConfigProjectPicker.swift:65-73`) — the pick still succeeds (the toast is advisory only, not a validation gate).

### 1.4 Skeleton loading state (`ConfigSkeletonView.swift:3-100`)

Shown before the first `.task` load completes (each screen keeps its own `isLoaded` flag, flips it true after a **fixed 400 ms `Task.sleep`**, not tied to the real load duration — e.g. `CommandsListView.swift:153-160`). Grid skeleton: two columns × 3 shimmering cards; list skeleton: 4 shimmering cards; each card mimics a title bar, two body lines and two badge chips (`ConfigSkeletonView.swift:51-85`). Used by Commands, Skills, Output Styles, Hooks, MCP Servers, Plugins. **Not** used by Models or Sub-agents — those two screens have no `isLoaded` gate at all and render `configContent` immediately (`ModelsListView.swift:44-53`, `SubAgentsListView.swift:57-66`), so on a slow disk they show an empty list flash instead of a skeleton (candidate parity gap, not a "do not port" bug — see §11).

### 1.5 Empty states (`ConfigEmptyState.swift:3-30`)

Centered icon + message + a hint pill. Two flavours per screen: a "no items on disk" state (icon/message/hint naming the folder, e.g. "No commands found" / `~/.claude/commands/`, `CommandsListView.swift:128-133`) and a "no filter matches" state (always `magnifyingglass` / `No … match "query"` / "Try a different search term", e.g. `CommandsListView.swift:134-139`). Models and Sub-agents show only the filter-miss variant (no items-missing state, since built-ins and the curated model catalog are never empty) — `ModelsListView.swift:44-49`, `SubAgentsListView.swift:57-62`.

### 1.6 Badges (`ConfigBadge.swift:3-87`)

- `ConfigBadge` — plain pill, caller supplies text/fg/bg. Used for model tags, tool counts, output-style tags, timeouts, handler counts, plugin versions.
- `ConfigScopeBadge` — globe+"Global" (tertiary) or folder+"Project" (green), driven by `ConfigScope` (`ConfigBadge.swift:18-39`). Shown on every card that has a scope: commands, skills, output styles, hooks. Also reused inside `MCPServerSourceBadge` for `source == .user` servers.
- `MCPServerSourceBadge` — switches on `MCPServerSource` (`user | cloudIntegration | plugin`, `Models/Config/MCPServer.swift:20-27`): `.user` renders the scope badge; `.cloudIntegration` renders "☁ claude.ai" (purple); `.plugin` renders "🧩 Built-in" (blue) (`ConfigBadge.swift:41-77`).
- `ConfigHelpers.formatModel` — strips `claude-` prefix, takes the first two hyphen-separated segments, title-cases them (`ConfigBadge.swift:80-86`); used for the model badge on command/skill cards and detail badges.

### 1.7 Detail view and its toolbar

`ConfigItemDetailView<Badges>` (`ConfigItemDetailView.swift:4-126`) is used **only** by Commands, Skills and Output Styles (the three markdown-bodied, single-file config kinds). It is not used by Hooks, MCP Servers, Models, Plugins or Sub-agents, which stay list-only or use a modal sheet form instead.

- **Header:** icon in a tinted square, title, `ConfigScopeBadge`, and the caller-supplied `badges()` closure (model/tools/argument-hint/output-style for commands, model/tools for skills, description+filename chip for output styles — `CommandsListView.swift:52-85`, `SkillsListView.swift:52-73`, `OutputStylesListView.swift:52-71`).
- **Body:** empty state ("No content") if `markdownBody.isEmpty`; otherwise a `ScrollView` toggling between rendered Markdown (`appState.configDetailFormatted == true`, MarkdownUI with the Poirot theme) and raw monospaced text — both `.textSelection(.enabled)` (`ConfigItemDetailView.swift:93-124`).
- **Navigation:** selecting a card sets `selectedCommand`/`selectedSkill`/`selectedStyle` and pushes `appState.activeConfigDetail` / `appState.pushConfigDetail(...)` (`CommandsListView.swift:234-246`); the list and detail panes cross-fade with a leading/trailing slide (`CommandsListView.swift:86-90`). A `.task(id: appState.activeConfigDetail?.filePath)` plus `.onChange(of: appState.activeConfigDetail)` keep the in-list selection in sync when navigation happens externally (e.g. from search), reloading the list on close (`CommandsListView.swift:92-114`).
- **Detail toolbar** (`ConfigDetailToolbar.swift`, rendered in the app's native toolbar, not the pane itself, whenever `appState.activeConfigDetail != nil`):
  - `ConfigToolbarActions` (`ConfigDetailToolbar.swift:5-97`): **Format toggle** (raw ↔ rendered, flips `appState.configDetailFormatted`, read-only); **Open in `<editor>`** (`EditorLauncher.open(filePath:editor:)`, editor from the `textEditor` `@AppStorage` preference, `ConfigDetailToolbar.swift:47-56`); **Copy Name** (clipboard, `ConfigDetailToolbar.swift:58-69`); **Copy Content** (clipboard, full markdown body, `:71-82`); **Show in Finder** (`NSWorkspace.activateFileViewerSelecting`, `:84-94`). All five show a 1.5–2 s checkmark micro-animation on their icon after firing.
  - `ConfigToolbarDelete` (`ConfigDetailToolbar.swift:99-134`): trash icon → native confirmation dialog ("Delete `name`? … cannot be undone") → `ClaudeConfigLoader.deleteConfigFile(at:)`, decrements the sidebar count, toasts, clears `activeConfigDetail`.
  - `ConfigToolbarClose` (`ConfigDetailToolbar.swift:136-148`): "×" clears `activeConfigDetail`, returning to the list.

**Read-only vs mutating, shared pieces:** layout toggle, filter field, project picker, format toggle, copy name/content are **read-only** (no Claude Code file is touched; clipboard writes need no Rust command beyond the clipboard-manager capability already granted to the main window, `docs/ipc-contract.md` §8). Open-in-editor and Show-in-Finder are **mutating-adjacent**: they don't write files but do launch external processes / OS panels, so in Heimdall they must go through `open_in_editor` / `reveal_in_file_manager` (Later, M3-53) rather than a raw path from the webview (ADR 0010 rule 5, rule 7). Delete is **mutating** (`delete_config_item`, Later, M3-44).

## 2. Commands (`Views/Config/CommandsListView.swift`)

**Data shown.** One card per `ClaudeCommand`: name (prefixed `/`), description (2-line clamp), scope badge, model badge (`ConfigHelpers.formatModel`), tool-count badge (`"N tools"`, from a comma-split of `allowed-tools`), output-style badge, and a per-card **Copy** button that copies `/name` to the clipboard with a 2 s checkmark flash (`CommandsListView.swift:280-355`). Loaded via `ClaudeConfigLoader.loadCommands(projectPath:)`, which reads front-matter (`description`, `model`, `argument-hint`, `allowed-tools`, `output-style`) from every `.md` file in `~/.claude/commands/` (global) and, if a project is picked, `<project>/.claude/commands/`, sorted case-insensitively by name (`Services/Config/ClaudeConfigLoader.swift:10-47`).

**Filters.** Fuzzy match on `name` and `description`, best score wins, sorted descending (`CommandsListView.swift:27-40`).

**Layouts.** Two-column grid (`CommandsListView.swift:184-207`) or single-column list (`:213-232`), both with a staggered shimmer-reveal on first paint (delay capped at 7/9 items).

**Actions.**
- Read-only: open a command → `ConfigItemDetailView` (§1.7); copy `/name` from the card; every §1.7 detail-toolbar action except Delete.
- Mutating (Later): `+` → `createAndOpen()` calls `ClaudeConfigLoader.createCommandTemplate()` (writes a new `commands/new-command[-N].md` with a stub front-matter block, `Services/Config/ClaudeConfigLoader.swift:758-774`), then immediately opens it in the external editor and reloads the list (`CommandsListView.swift:248-258`) — there is **no in-app creation form**, unlike Hooks/MCP/Sub-agents. Delete (via the detail toolbar, §1.7).

**Keyboard.** No in-app form, so ⌘./Return do not apply; `+` triggers the template-and-open flow directly.

## 3. Skills (`Views/Config/SkillsListView.swift`)

Structurally identical to Commands. **Data shown**: name, description (3-line clamp), scope badge, model badge, tool-count badge; per-card Copy button copies the bare skill `name` (no `/` prefix, `SkillsListView.swift:279-284`). Loaded via `ClaudeConfigLoader.loadSkills(projectPath:)`, one entry per subfolder of `~/.claude/skills/` (and the project's) containing a `SKILL.md`, front-matter `name`/`description`/`model`/`allowed-tools` (`Services/Config/ClaudeConfigLoader.swift:74-108`). Detail icon `bolt.fill`, accent color (`SkillsListView.swift:47-48`); detail badges: model + one badge per allowed tool (`:53-72`, no argument-hint/output-style badges — skills don't have those front-matter keys).

**Filters/layouts** identical pattern to Commands (`SkillsListView.swift:27-40`, `:172-220`).

**Actions.** Read-only: open, copy name, detail-toolbar actions. Mutating (Later): `+` → `ClaudeConfigLoader.createSkillTemplate()` creates a new `skills/new-skill[-N]/SKILL.md` folder+stub and opens it externally (`Services/Config/ClaudeConfigLoader.swift:776-801`, `SkillsListView.swift:236-246`); Delete removes the **parent folder**, not just `SKILL.md` (`Services/Config/ClaudeConfigLoader.swift:819-826`).

**Keyboard.** No in-app form.

## 4. Output Styles (`Views/Config/OutputStylesListView.swift`)

**Data shown.** Name, description (2-line clamp), scope badge, and a filename chip (e.g. `concise.md`) instead of model/tool badges — output styles have no model/tools front-matter (`OutputStylesListView.swift:253-297`). Loaded via `ClaudeConfigLoader.loadOutputStyles(projectPath:)`, every `.md` file in `~/.claude/output-styles/` **except `README.md`** (and the project's folder), front-matter `name`/`description` (`Services/Config/ClaudeConfigLoader.swift:382-416`). Detail icon `speaker.wave.3.fill`, red (`OutputStylesListView.swift:47-48`); detail badges show description again (redundant with the header, if non-empty) plus the filename chip (`:52-71`).

**Filters/layouts** identical pattern (`OutputStylesListView.swift:27-40`, `:170-218`).

**Actions.** Read-only: open, detail-toolbar actions (**no per-card copy button**, unlike Commands/Skills). Mutating (Later): `+` → `ClaudeConfigLoader.createOutputStyleTemplate()` creates `output-styles/new-style[-N].md` and opens it externally (`Services/Config/ClaudeConfigLoader.swift:803-817`, `OutputStylesListView.swift:234-244`); Delete via detail toolbar.

**Keyboard.** No in-app form.

## 5. Hooks (`Views/Config/HooksListView.swift`, `Views/Config/HookFormView.swift`)

### 5.1 List

**Data shown.** One card per `HookEntry` = one matcher group inside one `HookEvent`: first handler's icon (terminal for `command`, globe for `http`) + `displayCommand` (the command string or URL, 1-line clamp); optional `matcher:` row (monospace); scope badge; handler-type badge; timeout badge (`"Ns"`) when set; `"N handlers"` badge when `entry.handlerCount > 1`; italic `statusMessage` line when set (`HooksListView.swift:296-369`, fields from `Models/Config/Hook.swift:104-147`). Loaded via `ClaudeConfigLoader.loadHooks(projectPath:)`: reads the `hooks` object from `~/.claude/settings.json` (global) and, if picked, `<project>/.claude/settings.json`, flattening every matcher group under every recognised `HookEvent` into one `HookEntry` each; groups with zero parseable handlers are dropped; **unrecognised event keys and unrecognised handler keys are silently ignored** (`parseHandler` reads only `type/command/url/timeout/statusMessage`, `Services/Config/ClaudeConfigLoader.swift:554-606`). The flat list is sorted by `event.rawValue` string (`:592`); the **grouped display** instead iterates `HookEvent.allCases` in enum-declaration order (`HooksListView.swift:45-51`) — two different orderings for the same data (list order vs. section order), worth keeping as a deliberate choice, not a bug, if the Rust `list_hooks` port keeps a single canonical order plus a client-side `groupBy`.

**Filters.** Fuzzy match across event label, matcher, and first handler's `displayCommand`, best score wins (`HooksListView.swift:27-43`).

**Layouts.** Grid/list, but **grouped by event** first (`HooksListView.swift:161-187, 189-245`); only events with ≥1 filtered entry render a section header (icon, label, entry count pill).

**Actions.**
- Read-only: open the per-card menu (`⋯`) → **Export JSON** copies `SettingsWriter.exportHooksAsJSON([entry])` to the clipboard (`HooksListView.swift:274-281`, serialises through the same `serializeMatcherGroup` used for writes — see §10 item 4, so exported JSON also drops unknown handler keys).
- Mutating (Later): `+` → opens `HookFormView` in create mode (`HooksListView.swift:104-107`); card menu **Edit** → same sheet pre-filled (`:248-251`); card menu **Delete** → confirmation dialog → `SettingsWriter.deleteHook(event:matcherIndex:scope:projectPath:)`, always scope `.global` from this list (`HooksListView.swift:132-144`, note `HookFormView(scope: .global, …)` at `:112-114` — **the in-app add/edit/delete flow is hard-coded to global scope regardless of the picked project**; project-scoped hooks can only be viewed here, never edited or deleted in-app). A `FileWatcher` on the settings directory triggers `appState.refreshID` on external changes (e.g. `claude` CLI edits) so the list picks them up (`HooksListView.swift:263-272`).

### 5.2 Hook form (`HookFormView.swift`)

A 480×520 sheet, header bar (icon, "New Hook"/"Edit Hook", scope pill), scrollable form, action bar.

**Fields** (`HookFormView.swift:83-190`):
1. **Event** — segmented-style `Picker` over all 18 `HookEvent` cases (icon+label), plus a static description line for the selected event (`HookFormView.swift:86-105`, event catalog and descriptions at `Models/Config/Hook.swift:5-86`).
2. **Matcher (regex)** — shown only when `selectedEvent.supportsMatcher` (10 of 18 events support it: PreToolUse, PostToolUse, PostToolUseFailure, PermissionRequest, SessionStart, SessionEnd, Notification, SubagentStart, SubagentStop, ConfigChange, PreCompact — `Models/Config/Hook.swift:52-62`); free text, placeholder `"e.g. Bash|Write"`, optional (`HookFormView.swift:108-121`).
3. **Handler Type** — segmented `command`/`http` (`:124-137`).
4. **Command or URL** — one text field, label and placeholder swap with handler type (`:139-160`).
5. **Timeout (seconds)** — free text parsed with `Int(timeout)`; placeholder shows the type's default ("Default: 600" / "Default: 30") but **an invalid/non-numeric entry silently becomes `nil` (no timeout written), not the shown default** (`HookFormView.swift:162-174`, `:228`).
6. **Status Message** — free text, optional, shown in Claude Code while the hook runs (`:176-189`).

**Validation.** `isValid` requires a non-blank `command` when handler type is `command`, or a non-blank `url` when `http` (`HookFormView.swift:30-37`); the Save/Create button is `.disabled(!isValid)`. No URL-scheme validation, no regex-syntax validation of the matcher, no numeric-range check on timeout.

**Save.** Builds one `HookHandler` and wraps it in a **single-handler** `HookMatcherGroup` (`HookFormView.swift:224-235`), then calls `SettingsWriter.saveHook(event:matcherGroup:existingIndex:scope:projectPath:)` — always `scope: .global` as passed in from the list (§5.1) — and `onSave()` unconditionally (toast + close), with no check that the write actually succeeded (`SettingsWriter.saveHook` returns `Void`; see §10 for the multi-handler data-loss consequence of this).

**Keyboard.** Cancel button is bound to `.cancelAction` (⌘. / Esc); Save/Create is bound to `.defaultAction` (Return), disabled while `!isValid` (`HookFormView.swift:193-208`).

## 6. MCP Servers (`Views/Config/MCPServersListView.swift`, `Views/Config/MCPSetupWizardView.swift`, `Views/Config/MCPServerStatusIndicator.swift`)

### 6.1 List

**Data shown.** One card per `MCPServer`: status dot (`MCPServerStatusIndicator`, §6.1.1), name, transport label (`server.type?.uppercased()`, else `"HTTP"` if `url != nil`, else `"STDIO"` if `command != nil`, else none — `MCPServersListView.swift:301-308`), `MCPServerSourceBadge` (§1.6), a one-line connection-info string (`url`, or `command` + `args` joined with spaces, truncated middle — `:310-316, 387-393`), and up to 5 tool badges with a "+N more" expander (`:395-424`). An info banner above the list explains how to add servers (`+`, `claude mcp add`, or editing `~/.claude.json` directly — `:222-245`). Loaded via `ClaudeConfigLoader.loadMCPServers(projectPath:)` merging, in this override order, (1) top-level `mcpServers` in `~/.claude.json` (scope global, source `.user`), (2) `projects[path].mcpServers` in the same file when a project is picked (scope project, source `.user`, overrides same-named global entries), (3) `<project>/.mcp.json` for names not already present (scope project, source `.user`), (4) plugin-provided servers derived from `settings.json` tool-permission keys prefixed `plugin_` (source `.plugin`, only if the owning plugin is enabled or undeterminable), (5) cloud-integration servers from the MCP auth cache, names prefixed `"claude.ai "` (source `.cloudIntegration`, status forced to `needsAuth`) — `Services/Config/ClaudeConfigLoader.swift:121-330`. Statuses are resolved separately by `MCPServerStatusChecker.resolveStatuses(for:)` after load (`MCPServersListView.swift:277-282`, checker not in this task's inputs).

**6.1.1 Status indicator** (`MCPServerStatusIndicator.swift:5-75`): an 8pt SF Symbol dot, colour + label per `MCPServerStatus`: `connected` = green filled circle, `needsAuth` = orange warning triangle, `failed`/`unreachable` = red (circle-x / bolt-slash), `starting` = blue filled circle with a `.pulse` symbol effect (disabled under Reduce Motion), `unknown` = tertiary question mark. Accessibility label mirrors the tooltip text.

**Filters.** Fuzzy match on name and every tool name, best score wins (`MCPServersListView.swift:32-44`).

**Layouts.** Grid/list, info banner always first (`:154-186, 192-220`).

**Actions.** Per-card action icons (pencil/braces/folder/trash) render **only when `server.source == .user`** — plugin and cloud-integration servers show no actions (`MCPServersListView.swift:344-384`).
- Read-only: expand/collapse the tool list.
- Mutating (Later): `+` → `MCPSetupWizardView` in add mode (`MCPServersListView.swift:101-104`); pencil → wizard in edit mode (`:261-264`); braces ("Open in editor") → `EditorLauncher.open` at the server's line via `SettingsWriter.lineNumber(forMCPServer:)`, a **line-by-line brace-depth text scan** of `~/.claude.json` looking for a top-level `mcpServers` key at indent ≤ 2 and a `"\(serverName)"` prefix match (`SettingsWriter.swift:85-113`, `MCPServersListView.swift:247-254`) — brittle to re-indentation, tabs, or another key containing the same quoted substring; falls back to opening the file with no line number if not found; folder icon ("Show in Finder") reveals `~/.claude.json` itself, not the server (`MCPServersListView.swift:256-259`); trash → confirmation dialog ("This will remove the server definition from `~/.claude.json` and its tool permissions from `settings.json`") → **two separate, non-atomic writes**: `SettingsWriter.removeMCPServer` then `SettingsWriter.removeMCPPermissions` (`MCPServersListView.swift:266-275`) — an interruption between the two leaves orphaned `mcp__<name>__*` permission entries.

Two file watchers keep the list live: one on `~/.claude.json`'s parent directory (`configFileWatcher`, reloads and updates the sidebar count on change), one on `~/.claude/mcp-needs-auth-cache.json` (`authCacheWatcher`, reloads on auth-status change) — `MCPServersListView.swift:117-143`.

### 6.2 MCP setup wizard (`MCPSetupWizardView.swift`)

A 560×600 sheet with three steps, a header bar, a step indicator, and an action bar.

**Step 1 — Select Server** (`MCPSetupWizardView.swift:208-363`). A search field (fuzzy-free — plain `.lowercased().contains()` substring match on name/description/category via `MCPServerCatalog.search`, `Models/Config/MCPServerCatalog.swift:343-351`), then a **Custom Server** row (always first, terminal icon, purple) followed by the catalog grouped by `MCPCatalogCategory` (`Developer Tools`, `Productivity`, `Data & Search`, `Filesystem`, in that enum-declaration order — `Models/Config/MCPServerCatalog.swift:47-61`), each entry showing icon, name, one-line description, and either "No config needed" (green) or `"N vars"` (tertiary). Selecting **Custom Server** sets `useCustom = true`, clears `selectedEntry`/`serverName`, and jumps to Step 2 (`MCPSetupWizardView.swift:268-306`). Selecting a **catalog entry** sets `selectedEntry`, `useCustom = false`, defaults `serverName` to `entry.id` (only if `serverName` is currently empty or still equals the server being edited), pre-populates `envValues[key.id] = ""` for every `entry.envKeys` not already set, and jumps to Step 2 (`:308-363`).

**Full catalog** (`Models/Config/MCPServerCatalog.swift:65-340`), 14 entries: GitHub (1 required sensitive env var, `GITHUB_PERSONAL_ACCESS_TOKEN`), Playwright (0), Puppeteer (0), Sentry (1 required sensitive `SENTRY_AUTH_TOKEN` + 1 optional `SENTRY_ORG`), Linear (1 required sensitive `LINEAR_API_KEY`), Slack (1 required sensitive `SLACK_BOT_TOKEN` + 1 required `SLACK_TEAM_ID`), Notion (1 required sensitive `OPENAPI_MCP_HEADERS`), Google Drive (1 required `GDRIVE_CLIENT_ID` + 1 required sensitive `GDRIVE_CLIENT_SECRET`), Brave Search (1 required sensitive `BRAVE_API_KEY`), Fetch (0), PostgreSQL (1 required sensitive `POSTGRES_CONNECTION_STRING`), SQLite (1 required, non-sensitive `SQLITE_DB_PATH`, but see below — it is excluded from written `env`), Filesystem (1 required, non-sensitive `ALLOWED_DIRS`, comma-separated, excluded from `env` and instead appended as positional args), Memory (0), EverArt (1 required sensitive `EVERART_API_KEY`). Every entry's `command` is `npx`, `args` starts `["-y", "<npm package>"]`.

**Catalog field shape** (`MCPEnvKey`, `Models/Config/MCPServerCatalog.swift:21-44`): `id`, `label`, `description`, `placeholder` (default `""`), `isRequired` (default `true`), `isSensitive` (default `false`).

**Step 2 — Configure** (`MCPSetupWizardView.swift:367-501`). Always: **Server Name** text field (hint "Key used in mcpServers object"). If `useCustom`: **Command** text field (placeholder `"e.g. npx"`) and **Arguments** text field (placeholder `"e.g. -y @scope/server-name"`, hint "Space-separated arguments") — see §10 item 1 for the parsing bug. If a catalog entry is selected: a read-only code block showing `command + args` joined with spaces, then one field per `envKeys` entry — `SecureField` when `isSensitive`, else `TextField` — each labelled with a "Required" (red) or "Optional" (tertiary) pill and a description line below (`:437-494`); "No environment variables required" (green check) when `envKeys` is empty.

**Special-cased catalog IDs** (`MCPSetupWizardView.swift:61-92`): for `filesystem`, the `ALLOWED_DIRS` value is comma-split, trimmed, and appended as extra positional `args` instead of being written to `env`; for `sqlite`, `SQLITE_DB_PATH` is collected but **never written anywhere** — it is excluded from `resolvedEnv` (line 88) and is not appended to `args` (unlike `filesystem`), so filling in the SQLite database path field has no effect on the saved server definition. This is a functional bug independent of the four required "do not port" items; see §10.

**Validation** (`isConfigValid`, `MCPSetupWizardView.swift:99-110`): server name (post-trim) must be non-empty; if custom, the command (post-trim) must be non-empty (arguments may be empty); if catalog, every `envKeys` entry with `isRequired == true` must have a non-blank trimmed value in `envValues`. The Step 2 → Step 3 "Preview" button is `.disabled(!isConfigValid)`.

**Step 3 — Preview & Save** (`MCPSetupWizardView.swift:505-577`). Read-only JSON preview (`previewJSON`, monospaced, `.textSelection(.enabled)`) of `{ "<resolvedServerName>": { "command", "args", "env"? } }`, pretty-printed with sorted keys (`JSONSerialization` options `.prettyPrinted, .sortedKeys, .withoutEscapingSlashes`). `env` is included only if non-empty; **values are masked as `prefix(4) + "***"` only when the catalog entry marks that key `isSensitive`** — see §10 item 2 for the custom-server gap. Two info lines below the block: "This will be written to `~/.claude.json` under the `mcpServers` key" (always) and "Sensitive values will be stored in `~/.claude.json` on disk" (only when `resolvedEnv` is non-empty).

**Save.** `Add Server`/`Save` calls `SettingsWriter.addMCPServer(name:command:args:env:)` (create) or `SettingsWriter.updateMCPServer(originalName:newName:command:args:env:)` (edit) — see §10 item 3 for the field-dropping bug on update — then toasts and dismisses (`MCPSetupWizardView.swift:624-649`). Editing pre-fills from the existing `MCPServer`: if its `rawName` matches a catalog `id`, the catalog path is used and `env` values are copied into `envValues` in the clear; otherwise the custom path is used, `command`/`args` (space-joined) are copied, and the wizard jumps straight to Step 2, skipping the catalog picker (`:651-673`).

**Architecture note for the port.** Poirot keeps every secret (`envValues: [String: String]`) in plain Swift `@State` for the lifetime of the sheet, and previews it client-side. Heimdall's trust boundary (ADR 0010 rule 8) requires `env`/`header` values to never cross IPC as plaintext; `McpServerInput`/`SecretInput` (`docs/ipc-contract.md` §5) model this with `keep`/`set`/`remove` markers instead — the wizard's "populate from editing" and "preview JSON" steps cannot be ported as direct value-copies and need a Rust-side preview/mask command. Flagged here, not as a "do not port" item (see §11).

**Keyboard.** Cancel is `.cancelAction` (⌘. / Esc) on every step (`MCPSetupWizardView.swift:592-594`); the Step 2 "Preview" button and the Step 3 "Add Server"/"Save" button are both `.defaultAction` (Return) — the former disabled while `!isConfigValid`, the latter never disabled (`:602-616`). "Back" (no shortcut) appears on Steps 2–3 (`:583-590`).

## 7. Models (`Views/Config/ModelsListView.swift`)

**Data shown.** One card per `ClaudeModelInfo`: display name, "Global Default" pill (star, accent) when it equals the resolved default, "Project Default" pill (folder, green) when it equals the loaded project override, description, and a checklist of `strengths` (`ModelsListView.swift:233-363`). `allModels` = `ClaudeModelCatalog.models(discoveredIds:)` — a curated list of current models unioned with any model id discovered across every loaded session's `model` field, so a model the user has actually used always appears even if not curated (`ModelsListView.swift:20-23`, catalog data at `Models/ClaudeModelCatalog.swift`, not itself in this task's inputs). An info banner explains that `ANTHROPIC_MODEL` overrides the on-disk default (`ModelsListView.swift:153-179`).

**Filters.** Fuzzy match on `displayName` only (`ModelsListView.swift:25-35`).

**Layouts.** Grid/list, info banner first (`:81-115, 121-151`). **No skeleton** (§1.4) and **no "items missing" empty state** (§1.5) — only the filter-miss state exists, since the catalog is never empty.

**Actions.** Per-card buttons only appear when relevant (`ModelsListView.swift:304-356`): "Set as Default" (hidden if already default), "Set as Project Default" (only when a project is picked and it isn't already the project default), "Clear Project Override" (only when it is the project default).
- Read-only: none beyond browsing/filtering.
- Mutating (Later): "Set as Default" → `SettingsWriter.setDefaultModel(_:)` writes `model` into `~/.claude/settings.json` (via the shared, corruption-prone `loadSettingsDict`/`writeSettingsDict` pair, §10/§11). "Set as Project Default" / "Clear Project Override" → `SettingsWriter.setProjectModel(_:projectPath:)` writes/removes `model` in `<project>/.claude/settings.json`, reading with its own **separate** fallback-to-empty-dict logic (`SettingsWriter.swift:60-81`, cited by ADR 0009).

**Keyboard.** No in-app form; there is no `+` button on this screen at all (§1.2).

## 8. Plugins (`Views/Config/PluginsListView.swift`)

**Data shown.** One card per `ClaudePlugin`: name, `"by <author>"` (only if non-empty), an Enabled/Disabled pill (green dot / tertiary dot) that opens a popover with a single Enable/Disable button, a trash icon, a version badge (`"v<version>"`), and a relative "Installed X ago" string when `installedAt` parses as ISO-8601 (`PluginsListView.swift:209-326`). Loaded via `ClaudeConfigLoader.loadPlugins()` from `~/.claude/plugins/installed_plugins.json` (one entry per version per `name@author` key) cross-referenced with `enabledPlugins` in `settings.json` (`Services/Config/ClaudeConfigLoader.swift:349-378`). An info banner explains plugins are installed via `claude plugin add` or by placing files under `~/.claude/plugins/` (`PluginsListView.swift:152-175`).

**Filters.** Fuzzy match on `name` and `author`, best score wins (`PluginsListView.swift:17-30`).

**Layouts.** Grid/list, info banner first (`:88-118, 124-150`).

**Actions.**
- Read-only: none beyond browsing/filtering (no detail view, no external-editor link).
- Mutating (Later): toggle pill → popover → `SettingsWriter.togglePlugin(key:enabled:)` flips one boolean in `enabledPlugins` inside `settings.json` (`PluginsListView.swift:177-190`); trash → confirmation dialog ("This will remove the plugin from your enabled plugins in settings.json") → `SettingsWriter.removePlugin(key:)` deletes the key from `enabledPlugins` — **note this does not delete the plugin's files, only its `enabledPlugins` entry**, despite the button being labelled "Remove plugin" (`PluginsListView.swift:192-200, 276-284`).

**Keyboard.** No in-app form; no `+` button (§1.2).

## 9. Sub-agents (`Views/Config/SubAgentsListView.swift`, `Views/Config/SubAgentFormView.swift`)

### 9.1 List

**Data shown.** Two sections, "Built-in" then "Custom" (only rendered if non-empty): icon in a colour-tinted square (`AgentColor` → theme colour, default orange), name, model badge (orange, only if set), description (2-line clamp), and a `FlowLayout`-wrapped row of tool badges (`SubAgentsListView.swift:307-458`). `allAgents = SubAgent.builtIn + customAgents` — 4 fixed built-ins (Explore, Plan, Bash, General-purpose, each with a fixed tool list and no model/color/prompt, `Models/Config/SubAgent.swift:76-109`) plus every custom agent loaded from `~/.claude/agents/*.md` front-matter (`name`, `description`, `tools` comma-split, `model`, `color`, `memory`; body becomes `prompt`) via `ClaudeConfigLoader.loadCustomAgents()` (`Services/Config/ClaudeConfigLoader.swift:614-648`). **No project scope** — always global, no project picker in the toolbar (§1.2).

**Filters.** Fuzzy match on `name` and `description` across the combined built-in+custom list, then re-split into `filteredBuiltIn`/`filteredCustom` for section rendering (`SubAgentsListView.swift:27-48`).

**Layouts.** Grid/list, each with up to two sections (`:133-166`). No skeleton (§1.4); only the filter-miss empty state (§1.5) — built-ins guarantee the list is never truly empty.

**Actions.** Built-in cards show only Duplicate and Export in their action row / context menu (no Edit, no Delete); custom cards show Duplicate, Export and Delete, and are also directly tappable to Edit (`SubAgentsListView.swift:362-457`; a right-click context menu duplicates the same actions, `:444-457`).
- Read-only: Export as JSON writes a `NSSavePanel`-chosen `.json` file via `ClaudeConfigLoader.exportAgentAsJSON` (name/description/tools/model/color/prompt/memory, `Services/Config/ClaudeConfigLoader.swift:719-733`) — a **local file write outside `~/.claude`**, not a Claude-config mutation, so it is read-only with respect to Claude Code's own files even though it touches disk.
- Mutating (Later): `+` (toolbar) → `SubAgentFormView` create sheet (`SubAgentsListView.swift:85-87, 110-114`); tap a custom card, or its context-menu "Edit…" → same form pre-filled (`:115-119, 446-449`); "Duplicate" → builds a new `SubAgent` with `" Copy"` appended to the name, a fixed placeholder icon, a new UUID, and **`scope: .global` even if duplicating conceptually** (there is no other scope), then saves immediately with no confirmation or edit step (`SubAgentsListView.swift:256-270`); Delete → confirmation dialog → `ClaudeConfigLoader.deleteConfigFile(at:)`; toolbar "Import Agent from JSON" (square-and-arrow-down icon, always visible, not gated by `showAddButton`) → `NSOpenPanel` → `ClaudeConfigLoader.importAgentFromJSON(data)`, which requires only a `name` key (every other field optional/defaulted) and always forces `scope: .global` (`SubAgentsListView.swift:287-302`, `Services/Config/ClaudeConfigLoader.swift:735-754`) — invalid JSON shows a generic "Invalid agent JSON" toast with no detail. A directory `FileWatcher` on `~/.claude/agents/` keeps the custom list live on external edits (`SubAgentsListView.swift:96-106`).

### 9.2 Sub-agent form (`SubAgentFormView.swift`)

A 520×700 sheet: header ("New Agent"/"Edit Agent"), scrollable form, Cancel/Save action bar.

**Fields** (`SubAgentFormView.swift:105-300`):
1. **Name** — free text.
2. **File** (read-only derived preview) — `~/.claude/agents/<slug>.md` where `<slug>` is the name lowercased, spaces→hyphens, then every character that isn't a letter/digit/hyphen stripped; shows `"—"` while name is blank (`:94-101, 115-124`).
3. **Description** — free text, single line.
4. **Model** — segmented Opus/Sonnet/Haiku, default `"sonnet"` (or the existing agent's raw `model` string, which need not be one of these three — the segmented control then shows no selection).
5. **Color** — 6 filled circles (`AgentColor.allCases`: orange/red/blue/green/purple/teal), selection ring on the active one.
6. **Tools** — "All tools" master checkbox (`selectedTools.count == SubAgent.knownTools.count`), one checkbox per `ToolCategory` (Read-only tools, Edit tools, Execution tools, Other tools — `Models/Config/SubAgent.swift:69-74`) each bound to "all tools in this category selected", and a collapsed `DisclosureGroup` ("Show individual tools") with a 3-column grid of all 12 known tools individually toggleable (`SubAgentFormView.swift:179-265`). Default selection for a new agent: `Glob, Grep, Read, Bash`.
7. **Memory** — plain `Picker`, `AgentMemory.user` ("Enable (~/.claude/agent-memory/)") or `.none` ("None (no persistent memory)") (`:267-279`).
8. **System Prompt** — multi-line `TextEditor`, min height 120, monospaced, becomes the file body below the front-matter (`:281-300`).

**Validation.** `isValid` requires non-blank (trimmed) `name` **and** `descriptionText`; nothing else is validated (no filename-collision check before save, no duplicate-name check, no tool-list non-empty check) — Save/Create is `.disabled(!isValid)` (`SubAgentFormView.swift:44-47, 83-87`).

**Save.** Builds a `SubAgent` (existing `id`/`filePath` preserved when editing, else a new UUID and no `filePath`), always `icon: "person.crop.circle.badge.plus"` (custom agents cannot pick a different icon), `scope: .global`; calls the `onSave` closure, which is `SubAgentsListView.saveAndReload` → `ClaudeConfigLoader.saveAgent(_:)`. `saveAgent` derives the target filename from the (possibly just-edited) name; if it differs from the existing file's stem it writes a **new** file and deletes the old one (rename-by-copy, `Services/Config/ClaudeConfigLoader.swift:650-698`) — so renaming an agent changes its `filePath`/id-on-disk with no explicit warning in the form.

**Keyboard.** Cancel is `.cancelAction` (⌘. / Esc); Save/Create is `.defaultAction` (Return), disabled while `!isValid` (`SubAgentFormView.swift:78-87`).

## 10. Do not port

Four items named explicitly in the task, plus others found while reading the same files. All line numbers are current as of the pinned commit (`a29dd6ee0e0133ef26bd067c164c66b160e13d86`).

1. **Naive space-split of custom MCP args** — `resolvedArgs` splits the raw "Arguments" text field on a single space and drops empty pieces (`Views/Config/MCPSetupWizardView.swift:66-70`):
   ```swift
   return customArgs
       .components(separatedBy: " ")
       .map { $0.trimmingCharacters(in: .whitespaces) }
       .filter { !$0.isEmpty }
   ```
   Any argument containing a space (a quoted path, a JSON literal, a flag value with spaces) is silently split into multiple wrong argv entries. Heimdall's wizard needs a real shell-word tokenizer (or a repeatable "add argument" list UI) instead of a single free-text field.

2. **4-character secret preview that skips custom servers** — the preview JSON masks `env` values only when the *catalog* marks that key `isSensitive`; for a custom server (`selectedEntry == nil`) `isSensitive` always evaluates to `false`, so custom-server secrets are shown **unmasked, in full**, in the on-screen preview (`Views/Config/MCPSetupWizardView.swift:561-562`):
   ```swift
   let isSensitive = selectedEntry?.envKeys.first(where: { $0.id == key })?.isSensitive ?? false
   masked[key] = isSensitive ? String(value.prefix(4)) + "***" : value
   ```
   Even for catalog servers, a 4-character prefix is a weak mask (e.g. a token starting with a fixed public prefix like `ghp_` leaks nothing extra, but a 4-char database password leaks most of it). Heimdall must mask (or simply never render) every secret value client-side, independent of any catalog metadata, per ADR 0010 rule 8.

3. **`updateMCPServer` drops `type`, `url` and `headers`** — the update path only ever writes `command`/`args`/`env` (`Services/Config/SettingsWriter.swift:150-177`):
   ```swift
   var serverDef: [String: Any] = [
       "command": command,
       "args": args,
   ]
   if !env.isEmpty { serverDef["env"] = env }
   servers[newName] = serverDef
   ```
   Editing any server that was originally `type: "http"`/`"sse"`, had a `url`, or had `headers` — none of which the wizard's form can even represent — silently deletes those fields on save. ADR 0009 decision 6 requires field preservation for exactly this reason; Heimdall's `update_mcp_server` must merge into the existing JSON value, touching only the keys the form edits.

4. **Hook handler serialisation drops unknown fields** — `serializeMatcherGroup` only ever writes `type`, `command`/`url`, `timeout`, `statusMessage` (`Services/Config/SettingsWriter.swift:296-314`); the read side (`parseHandler`, `Services/Config/ClaudeConfigLoader.swift:595-606`) is equally narrow, so any handler key Claude Code itself adds in the future (or a hand-edited key) is invisible on read and erased on the next in-app save. This is the exact case the IPC contract's `HookHandler.extraKeys` field (`docs/ipc-contract.md:127`) and ADR 0009 decision 6 exist to fix — Heimdall must round-trip unknown keys, not just avoid overwriting the whole object.

**Additional weaknesses found while reading these files** (not in the task's list, still worth avoiding):

5. **Editing a multi-handler hook silently truncates it to one handler.** `HookFormView` only ever populates from and writes back `entry.firstHandler` (`Views/Config/HookFormView.swift:210-221`) wrapped in a matcher group with exactly one handler (`:224-235`, `handlers: [handler]`). The list itself proves multi-handler groups are real data — it renders an explicit `"N handlers"` badge when `entry.handlerCount > 1` (`Views/Config/HooksListView.swift:349-355`) — so opening such a group in the editor and clicking Save discards every handler after the first with no warning. This is the single most destructive bug found in the config screens; Heimdall's hook form must either support editing all handlers in a group or refuse to open multi-handler groups for editing.

6. **Every settings/config writer fails open on a corrupt or unreadable file, then overwrites it** — already the subject of ADR 0009: `addMCPServer` defaults to `[:]` on parse failure (`Services/Config/SettingsWriter.swift:123-128`), `setProjectModel` does the same (`:60-81`, esp. `64-68`), `loadSettingsDict` (used by plugin toggle/removal, MCP-permission removal, default-model set) does the same (`:359-365`), `loadSettingsDictFor` (used by hook save/delete/import) does the same (`:331-340`). `updateMCPServer` and `removeMCPServer` differ — they `guard`-fail and write nothing on a parse error (`:148-158`, `:181-186`) — so the failure mode is inconsistent across writers in the same file. Every write also serialises with `.sortedKeys` (rewrites the whole file, loses original key order/comments — occurrences at `SettingsWriter.swift:78, 143, 174, 194, 265, 352, 371`) and every write silently swallows I/O errors via `try?` with no return value or error the caller can inspect.

7. **Project-scope hook writes silently target the global file when no project is picked** — `settingsURL(scope:projectPath:)` falls back to `settingsFileURL()` (the global file) if `scope == .project` but `projectPath == nil` (`Services/Config/SettingsWriter.swift:316-329`, esp. `323-324`). Combined with item 6's `[:]` fallback, a hook "saved" while believing it targets a project can end up silently mutating `~/.claude/settings.json` instead. ADR 0009 decision 5 requires this to be a hard `invalidInput` error instead.

8. **Removing an MCP server is two non-atomic writes** — `removeServer` calls `SettingsWriter.removeMCPServer` then `SettingsWriter.removeMCPPermissions` as two independent file rewrites with no rollback (`Views/Config/MCPServersListView.swift:266-275`); an interruption between them (crash, Windows sharing violation) leaves orphaned `mcp__<name>__*` permission entries in `settings.json` referencing a server that no longer exists in `~/.claude.json`.

9. **`SettingsWriter.lineNumber(forMCPServer:)` is a hand-rolled, non-JSON-aware line scanner** (`Services/Config/SettingsWriter.swift:85-113`) — it assumes the top-level `mcpServers` key sits at indent ≤ 2 spaces and matches the server key by string-prefix on `"\"<name>\""`, with a manual `{`/`}` counter to detect leaving the block. Differently-indented or minified JSON, tabs, or a nested object elsewhere in the file that happens to contain the same quoted substring at the wrong depth will misroute "Open in Editor" to the wrong line (or none). A real JSON pointer or a `serde_json` AST walk should replace this in Heimdall.

10. **Silent-success toasts on writers that can silently no-op.** Every screen shows an unconditional success toast right after calling one of the `Void`-returning `SettingsWriter` methods (e.g. `PluginsListView.swift:177-190` toasts "enabled/disabled" even if `togglePlugin` silently failed to write per item 6; `HookFormView.swift:237-249` toasts "Hook updated"/"Hook created" unconditionally). Heimdall's `WriteOutcome` return type (`written | cancelled | unchanged`, `docs/ipc-contract.md:38`) exists specifically so the UI can show accurate feedback instead of an always-optimistic toast.

## 11. Open questions

- **`MCPServer.source` has 3 cases in Swift, `McpServer.source` has 5 in the IPC contract.** `Models/Config/MCPServer.swift:20-27` defines only `user | cloudIntegration | plugin`; `docs/ipc-contract.md:139` lists `"user" | "local" | "project" | "plugin" | "cloudIntegration"`. Swift folds Poirot's three `.user`-sourced load paths — top-level `~/.claude.json` (global), `projects[path].mcpServers` (project override inside the same file), and `<project>/.mcp.json` (project-local file) — into one `source: .user` value, distinguished only by `scope` (`Services/Config/ClaudeConfigLoader.swift:126-170`). The contract's richer enum appears to want those three paths split into `user` / `local` / `project` respectively. Needs a decision before `list_mcp_servers` (M1-45) ships: which of the three Swift load sites maps to which of the five Rust values, and whether `scope` becomes redundant once `source` is split this way.
- **Secret handling shape mismatch in the MCP wizard.** Poirot keeps `env` values in cleartext `@State` for the whole wizard session and masks only in the JSON preview (see §6.2's architecture note and §10 item 2). Heimdall's `McpServerInput`/`SecretInput` (`docs/ipc-contract.md:144-146`) model `keep`/`set`/`remove` instead of a live value. The exact UX for "edit an existing sensitive value" (show a placeholder dot-field with an explicit "change" affordance, vs. Poirot's `SecureField` that silently starts blank-then-typed) is not specified anywhere in Poirot and needs a product decision for M3-45.
- **`ConfigLayout` (list/grid) and `configDetailFormatted` (raw/rendered) persistence** — both are read through `appState` methods (`appState.configLayout(for:)`, `appState.toggleConfigLayout(for:)`, `appState.configDetailFormatted`) that live outside this task's inputs (likely `AppState.swift`, not read here). Heimdall's `Prefs` (ADR 0007) is the natural owner, but the exact key shape (per-screen dictionary vs. one global toggle) isn't visible from the files this task lists and should be confirmed against `AppState.swift` before M1-39/M1-44 lock the `Prefs` schema.
- **Models and Sub-agents skip the skeleton state** (§1.4) and **Models/Plugins have no `+`/creation flow** (§1.2) — confirmed by direct reading, not asserted as bugs, but flagged in case M1-44/M2 wants Heimdall to be more consistent than Poirot here (a product decision, not a parity requirement).
- **HookEvent has 18 cases but only 10 `supportsMatcher`** (`Models/Config/Hook.swift:52-62`) — confirmed from source, listed here only because the task's Inputs didn't separately ask for the full event catalog; useful for building the Vue event picker without re-deriving it from Swift.

## Sources

Every file in the task's Inputs list was read in full: `Views/Config/CommandsListView.swift`, `SkillsListView.swift`, `OutputStylesListView.swift`, `HooksListView.swift`, `HookFormView.swift`, `MCPServersListView.swift`, `MCPSetupWizardView.swift`, `MCPServerStatusIndicator.swift`, `ModelsListView.swift`, `PluginsListView.swift`, `SubAgentsListView.swift`, `SubAgentFormView.swift`, `ConfigDetailToolbar.swift`, `ConfigProjectPicker.swift`, `ConfigItemDetailView.swift`; `Models/Config/MCPServerCatalog.swift`, `Hook.swift`, `SubAgent.swift`; `Services/Config/SettingsWriter.swift` (full, 375 lines) and `ClaudeConfigLoader.swift` (full, 862 lines). Five shared components not in the Inputs list were also read because every list screen depends on them: `ConfigScreenHeader.swift`, `ConfigSkeletonView.swift`, `ConfigLayoutToolbar.swift`, `ConfigEmptyState.swift`, `ConfigBadge.swift` (defines `ConfigBadge`, `ConfigScopeBadge`, `MCPServerSourceBadge`, `ConfigHelpers`); plus `Models/Config/MCPServer.swift`, `Models/Config/ConfigScope.swift`, and the first ~60 lines of `Models/ClaudeModelCatalog.swift` for field shapes referenced by the screens above.
