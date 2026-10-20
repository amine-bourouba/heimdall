# Universal search (⌘K) — behaviour spec

Task M0-07. Source of truth: `$SWIFT_REF/Poirot/Sources/Views/Search/SearchOverlayView.swift`,
`Utilities/HighlightedText.swift`, `Models/Provider/NavigationItem.swift` (commit `a29dd6e`). Every
claim below cites a Swift `file:line`. Cross-references into other Swift files (confirmatory only)
are marked as such. Vocabulary matches `docs/ipc-contract.md` §5 (`SearchMode`, `SearchHit`,
`SearchAction`, `SearchCategory`, `SearchResults`) and the `search` command (`ipc-contract.md:249`).

This spec is for two implementers: a Rust agent building `search/matcher.rs` and
`search/providers.rs` (`M1-29`, `M1-30`), and a Vue agent building `SearchOverlay.vue` (`M3-01`).

## 1. The 16 categories, in Swift enum order

Order is fixed by `SearchCategory: Int, CaseIterable` (`SearchOverlayView.swift:5-21`); display
order always follows this declaration order because the final grouping step iterates
`SearchCategory.allCases` (`SearchOverlayView.swift:601`), independent of the order each
category's candidates were built in the function body (`:223-597`). A category with zero matches
is omitted entirely — there are no empty-section headers (`:602-603`).

`loadConfigItems()` (`:1038-1080`) loads every non-session data source **once**, when the overlay
first appears (`.task` at `:675`), threading `appState.effectiveConfigProjectPath` (confirmed in
`AppState.swift:243`) into the five loaders that accept a `projectPath`. Sessions instead read
live from `appState.projects` (an `@Environment` reference), so only session-derived categories
(Sessions, Debug logs, Facets, File changes when the session resolves) reflect changes made while
the overlay is open; the other categories are a snapshot for the overlay's lifetime.

| # | `SearchCategory` case | Label | Data source (`SearchOverlayView.swift`) | Matched fields (all pass through `score()`, `:212-217`, unless noted) | Project-scoped by `effectiveConfigProjectPath`? | `SearchAction` |
|---|---|---|---|---|---|---|
| 1 | `sessions` | SESSIONS | `appState.projects[].sessions`, skip `isSidechain` unless `appState.showAgentSessions` (`:224-226`) | `max(title, project.name, session.id)`; if all score 0, fallback `session.searchableText.contains(loweredQuery) ? 1 : 0` — **always** a literal lowercase substring test, regardless of search mode (`:227-233`; `searchableText` is pre-lowercased and capped at 20,000 chars, `TranscriptParser.swift:134,469`, confirmed) | No | `openSession(session, projectId)` |
| 2 | `debugLogs` | DEBUG LOGS | Sessions whose id ∈ `debugLogSessionIds` (`Set<String>` from `DebugLogLoader().allSessionIds()`, `:1052,1068`) (`:248-250`) | `max(session.title, "debug log")` — the literal phrase "debug log" is scored, so any query matching that phrase surfaces *every* session with a debug log (`:251`) | No | `openDebugLog(sessionId, projectId)` |
| 3 | `facets` | AI SUMMARIES | `allFacets: [String: SessionFacets]` from `FacetsLoader().loadAllFacets()` (`:1074-1076`) | `max(underlyingGoal, briefSummary, max(goalCategories.keys), sessionType)` (`:536-541`) | No | `openSession` if the session id resolves in `appState.projects` (`:544-548`), else `navigateTo(.sessions)` (`:550-554`) |
| 4 | `fileChanges` | FILE CHANGES | `fileHistorySessionFiles`: up to the first **50** entries of `~/.claude/file-history/` (`.prefix(50)`, `:1097`), each scanned via `FileHistoryLoader` for its file names (`:1084-1107`) | Per file: `max(fileName-last-component, full fileName, "file changes")` (`:267-270`) | No | `openSession` if a project/session matching `sessionId` is found, else `navigateTo(.sessions)` (`:273-283`) |
| 5 | `todos` | TODOS | `todoEntries`: `TodoLoader().loadAllTodos()` filtered to non-empty lists (`:1051,1066-1067`) | `max(todo.content, todo.activeForm)` (`:299-301`) | No | `navigateTo(.todos)` — never opens the owning session (`:311`) |
| 6 | `history` | HISTORY | `historyEntries`: `HistoryLoader().loadAll()` (`:1053,1069`) | `max(entry.display, entry.projectName)`; title shown is `entry.snippet` = first 3 non-empty lines of `display` joined by a space (`HistoryEntry.swift:17-22`, confirmed) (`:317-318`) | No | `navigateTo(.history)` |
| 7 | `commands` | COMMANDS | `commands: [ClaudeCommand]` from `ClaudeConfigLoader.loadCommands(projectPath:)` (`:1045,1059`) | `max(name, description)` (`:333-334`) | **Yes** | `openDetail(.commands, ConfigDetailInfo(name, body, filePath, scope))` |
| 8 | `skills` | SKILLS | `skills: [ClaudeSkill]` from `loadSkills(projectPath:)` (`:1046,1060`) | `max(name, description)` (`:354-355`) | **Yes** | `openDetail(.skills, …)` |
| 9 | `plans` | PLANS | `plans: [Plan]` from `loadPlans()` — no project path (`:1047,1061`) | `max(name, content.prefix(200))` (`:375-376`) | No | `openDetail(.plans, …)` |
| 10 | `memory` | MEMORY | `memoryFiles`: **only** `ClaudeConfigLoader.projectsWithMemory().flatMap { loadMemoryFiles(projectDirName:) }` (`:1054-1056`) — every project's memory, globally, but never `loadMemoryFiles(customFolder:)` (`ClaudeConfigLoader.swift:489`, confirmed unused here) | `name` score, else `content.localizedCaseInsensitiveContains(q) ? 1 : 0` fallback — again always a literal substring test regardless of mode (`:571-575`) | No (reads every project's memory; not the "current" project only) | `openDetail(.memory, …)` |
| 11 | `mcpServers` | MCP SERVERS | `mcpServers: [MCPServer]` from `loadMCPServers(projectPath:)` + `MCPServerStatusChecker.resolveStatuses` (`:1041-1043,1062`) | `max(name, max(tools), status.label)` (`:396-398`) | **Yes** | `navigateTo(.mcpServers)` |
| 12 | `plugins` | PLUGINS | `plugins: [ClaudePlugin]` from `loadPlugins()` — no project path (`:1049,1063`) | `max(name, author)` (`:414-415`) | No | `navigateTo(.plugins)` |
| 13 | `outputStyles` | OUTPUT STYLES | `outputStyles: [OutputStyle]` from `loadOutputStyles(projectPath:)` (`:1050,1064`) | `max(name, description)` (`:430-431`) | **Yes** | `openDetail(.outputStyles, …)` |
| 14 | `hooks` | HOOKS | `hookEntries: [HookEntry]` from `loadHooks(projectPath:)` (`:1071-1073`) | `max(firstHandler?.displayCommand, matcher, event.label)` (`:468-472`) | **Yes** | `navigateTo(.hooks)` |
| 15 | `models` | MODELS | `provider.supportedModels` (`:451`) — curated catalog, kept at parity per assumption A-6 (`docs/port-plan.md:111`) | `score(model)` on the raw model id string (`:452`) | No | `navigateTo(.models)` |
| 16 | `subAgents` | SUB-AGENTS | `SubAgent.builtIn + ClaudeConfigLoader.loadCustomAgents()` (`:487`) | `max(name, description)` (`:489`) | No | `navigateTo(.subAgents)` |

`SearchCategory.navItem` (`:44-63`) is a computed property that maps each category to a
`NavigationItem`; it is **dead code** — nothing in this file calls `.navItem` (confirmed by
search). The real navigation targets are the hardcoded `NavigationItem` literals passed to each
`.navigateTo`/`.openDetail` call above, which happen to agree with what `.navItem` would return.

## 2. Matching: fuzzy vs exact ("Match Word")

`SearchMode` (`:93-112`) has two cases, persisted as `@AppStorage("globalSearchMode")`
(`:139-144`, default `.fuzzy`). Swift's raw values are `"fuzzy"` and `"matchWord"`; Heimdall's
`SearchMode` (`ipc-contract.md:148`) renames the second to `"exact"` — same behaviour, new label.
`score(text)` (`:212-217`) dispatches on mode: fuzzy → `HighlightedText.fuzzyMatch(text, query:)?.score ?? 0`; exact → `HighlightedText.substringMatch(text, query:)  ?? 0`.

### 2.1 Exact ("Match Word") — `HighlightedText.substringMatch` (`HighlightedText.swift:29-46`)

Case-insensitive **contiguous substring** search (`lower.range(of: qLower)`, `:33`). Returns `nil`
(no match) if the query is not a substring; returns `0` for an empty query (`:30`). When found:

- Base score **100**.
- **+40** if the match starts at position 0 (`:36-37`).
- else **+20** if the character immediately before the match is one of `{space, -, /, ., _, :}`
  (`:39-41`) — note this boundary set includes `:`, unlike the fuzzy separator set below.
- **−min(len(text)/20, 30)** — integer division, length penalty capped at 30 (`:44`).

### 2.2 Fuzzy — `HighlightedText.fuzzyMatch` (`HighlightedText.swift:67-122`)

Ordered-subsequence match (typo-tolerant, scattered characters allowed) over lowercased text and
query. Empty query → `FuzzyResult(indices: [], score: 0)` (`:68`). Query length < 2 chars falls
back to a plain case-insensitive substring search, returning a single index at the match start and
a fixed **score 10** (`:70-76`) — i.e. one-character queries are never fuzzy.

For query length ≥ 2, scan `text` left to right; each time the current text character equals the
next unmatched query character:

- **+10** per matched character (`:90`).
- **+8** if this match is at text index 0 (`:92-93`).
- else **+5** if it is immediately consecutive with the previous match (`:94-96`).
- else **+3** if the character immediately before it is one of `{space, -, /, ., _}` — no `:`
  here, unlike exact mode (`:97-102`).
- **−(gap − 1)** where gap is the distance from the previous match, applied whenever a previous
  match exists, on top of any bonus above (`:104-107`).

If the query is not fully consumed as a subsequence, the match fails (`nil`, `:115`). Otherwise the
match is rejected (`nil`) unless `score ≥ query.count * 5` (`:117-119`) — this rejects noisy,
widely-scattered hits.

### 2.3 Worked examples (traced against the algorithm above; the first two are asserted by
`$SWIFT_REF/PoirotTests/Utilities/HighlightedTextTests.swift`)

| Query | Text | Fuzzy | Exact ("Match Word") |
|---|---|---|---|
| `ssh` | `swordfish` | Matches. Indices `[0,7,8]` (`s`,`s`,`h`). Score: `s`@0 → 10+8=18; `s`@7 → 10, not consecutive, char before (`i`) not a boundary, gap=6 → 18+10−6=22; `h`@8 → 10, consecutive (+5), gap=0 → 22+15=37. `37 ≥ 3×5=15` → **accepted, score 37**. | `lower.range(of:"ssh")` — no contiguous `"ssh"` substring in `"swordfish"` → **`nil`** (`HighlightedTextTests.swift:13-17`). This is *the* case the two modes disagree on. |
| `ssh` | `ssh config` | trivially matches, indices `[0,1,2]`, all consecutive + start bonus | start-of-string substring: `100+40=140`, length 10 → `10/20=0` → **140** |
| `ssh` | `fix ssh now` | matches with a gap before the run | boundary match (preceded by space at index 3): `100+20=120`, length 11 → **120** |
| `ssh` | `misshapen` | matches (`s`,`s`,`h` inside `misshapen`) | mid-word match (preceded by `i`, not a boundary): `100+0=100`, length 9 → **100** |

The third block (`ssh config` > `fix ssh now` > `misshapen`, i.e. **140 > 120 > 100**) matches
`HighlightedTextTests.swift:20-27` (`ranksPrefixOverBoundaryOverMid`), confirming prefix beats
word-boundary beats mid-word for exact mode.

The doc comment at `SearchOverlayView.swift:91-92` ("`ssh` also matches `se**ss**ion`") is an
imprecise paraphrase of this same fuzzy-noise concern — see Open questions §11.

### 2.4 Highlight ranges

Two different functions compute highlight ranges, and **the choice is driven by search mode, not
by which field actually produced the score**:

- Exact — `HighlightedText.attributedString(text, query:)` (`HighlightedText.swift:6-21`): finds
  *every* non-overlapping case-insensitive occurrence of the full query string in `text`, in a
  loop, and highlights each occurrence as one contiguous range `[start, end)`.
- Fuzzy — `HighlightedText.fuzzyAttributedString(text, query:)` (`:136-150`): re-runs
  `fuzzyMatch(text, query:)` and highlights each matched **character individually** — i.e. one
  `[i, i+1)` range per matched character, not one range spanning the whole match.

`UniversalResultRow.highlightedTitle` (`SearchOverlayView.swift:1121-1125`) always highlights
`result.title` only, re-deriving the match from `result.title` + the raw `query` — **never** the
field that actually produced `result.score` (title vs. project name vs. session id vs. content
fallback). Consequence: when a row is present only because of a fallback content match (Sessions
`searchableText` contains, Memory `content` contains — both always score 1) or because a
non-title field scored highest, the title typically renders with **no highlight** at all, because
the match function finds nothing in the title itself. Implementers should treat `SearchHit.title`
highlighting as "does the query match the title", independent of "why did this row match".

Also note: the row's `query` argument is the **raw, un-debounced** `@State query` (`:851-853`,
`:1114`), not `debouncedQuery` — so during the 200 ms debounce window the highlight recomputes
against the latest keystrokes even though the underlying result set is still one keystroke behind
(§8).

Concrete example, text `"ssh config"`, query `"ssh"` (both modes match here, §2.3): exact mode
produces **one range `[0,3)`** (the whole word); fuzzy mode produces **three ranges
`[0,1),[1,2),[2,3)`** (one per matched character) — visually near-identical for a contiguous hit
like this one, but structurally different, which matters once matches are scattered (e.g. the
`swordfish` example above would fuzzy-highlight `[0,1),[7,8),[8,9)` and not highlight at all in
exact mode, since exact mode found no match).

**Mapping to `SearchHit.highlights: [number, number][]`** (`ipc-contract.md:149`, UTF‑16 offsets
into `title`): produce one `[start, end)` pair per exact-mode occurrence, or one `[i, i+1)` pair
per fuzzy-matched character. Offsets must be **UTF‑16 code-unit** offsets (what a JS string index
means), not Rust byte or `char` offsets — Swift's `AttributedString`/`String.Index` offsets used
here are grapheme-cluster based, so neither side's native indexing lines up exactly; the Rust
matcher must count UTF‑16 units explicitly when building ranges.

## 3. Empty-query state (`query.isEmpty`)

Rendered by `emptyStateContent` (`:863-906`), gated by `if query.isEmpty` in `searchResultsList`
(`:811`).

1. **"SCREENS"** section (`:865`): `configNavItems` = `NavigationItem.allItems` with `id != "sessions"`
   filtered out (`:616-620`), so **13 items** in `NavigationItem.allItems` declaration order
   (`NavigationItem.swift:114-129`): Analytics, TODOs, History, Commands, Skills, Plans, Memory,
   Hooks, MCP Servers, Plugins, Output Styles, Models, Sub-agents. Each row shows
   `appState.sidebarCounts[navItem.id]` as a trailing count when present (`:873`). Selecting
   (click or Enter) sets `appState.selectedNav` and dismisses (`:877-881`, `:1020-1024`).
2. **"RECENT"** section (`:883-904`), shown only if non-empty: the **5 most recent sessions**
   across all projects, `appState.projects.flatMap sessions`, sorted by `session.startedAt`
   descending, `prefix(5)` (`:622-629`). Selecting sets `selectedProject`, `selectedSession`,
   `selectedNav = .sessions`, dismisses (`:897-903`, `:1025-1035`).

Total empty-state row count = `emptyItemCount = configNavItems.count + recentSessions.count` = 13
(+ up to 5) = up to 18 (`:631-633`); this is the bound used for the Down-arrow clamp (§6).

## 4. Quick actions

Both are synthetic `SearchResult`s appended during `buildSearchResults`, scored by `score()` like
any other row, and therefore only appear once the query matches their label text — they are not
shown in the empty-query state.

| Label | Category slot | Icon | Subtitle | Score source | `SearchAction` | Effect |
|---|---|---|---|---|---|---|
| "Toggle Menu Bar Icon" | `commands` (shares the slot/cap with real Commands, `:509`) | `menubar.rectangle` | "Show or hide the menu bar icon" | `score("Toggle Menu Bar Icon")` (`:505`) | `.toggleMenuBar` | Flips `UserDefaults.standard.bool(forKey: "showMenuBarIcon")` (default `true`) (`:1005-1008`), then dismisses |
| "Add MCP Server" | `mcpServers` (shares the slot/cap with real MCP servers, `:524`) | `plus.circle` | "Open the MCP server setup wizard" | `score("Add MCP Server")` (`:521`) | `.addMCPServer` | Sets `selectedNav = .mcpServers`, then after a 100 ms `Task.sleep` sets `appState.configAddTrigger = UUID()` to open the add-server wizard (`:1010-1015`), then dismisses |

Per the parity matrix, both quick actions are deferred in Heimdall (`docs/parity-matrix.md:113-114`,
rows S17/S18 — "HD until the tray ships" / "HD until the wizard ships") until their owning
features (`M3-58` tray, `M3-48` MCP wizard) land; this spec documents the full Swift behaviour so
those later tasks can wire it up without re-reading Swift.

## 5. `SearchAction` variants (Swift, `SearchOverlayView.swift:66-73`)

`navigate(to:)` (`:983-1018`) switches on the action and always ends with `dismiss()` (`:1017`,
sets `appState.isSearchPresented = false`, `:977-981`).

| Swift case | Effect (`:983-1016`) | Heimdall `SearchAction` (`ipc-contract.md:151-155`) |
|---|---|---|
| `openSession(Session, projectId:)` | Sets `selectedProject`, `selectedSession`, `selectedNav = .sessions` | `{ kind: "openSession", projectId, sessionId }` — same shape, ids instead of the inline `Session` object |
| `openDebugLog(sessionId:, projectId:)` | Sets `selectedProject`, `selectedNav = .sessions`, `showDebugLogSessionId` | `{ kind: "openDebugLog", projectId, sessionId }` — matches |
| `navigateTo(NavigationItem)` | Sets `selectedNav` | `{ kind: "navigateTo", screen: NavScreen }` — `NavigationItem.allItems` (14 items incl. Sessions) matches `NavScreen`'s 14 variants (`ipc-contract.md:160-161`) 1:1 |
| `openDetail(NavigationItem, ConfigDetailInfo)` | Sets `selectedNav`, `activeConfigDetail`, pushes onto the nav stack (`AppState.pushConfigDetail`, confirmed `AppState.swift:155`) | `{ kind: "openDetail", screen, ref: ConfigRef \| null, memoryFile: {...} \| null }` — Heimdall sends an **id** (`ConfigRef`/`memoryFile` locator), not the inline `ConfigDetailInfo` (name/markdownContent/filePath/scope); the webview must fetch the detail body via `get_config_item`/`get_memory_file` after navigating. This matches the project's "commands take ids, never content/paths" rule (`AI_INSTRUCTIONS.md`) |
| `.toggleMenuBar` | Flips a `UserDefaults` bool (see §4) | **No IPC equivalent today.** Deferred with the quick action itself (S17) |
| `.addMCPServer` | Navigates + fires a UUID trigger (see §4) | **No IPC equivalent today.** Deferred with the quick action itself (S18) |

## 6. Key bindings

| Binding | Scope | Effect | Citation |
|---|---|---|---|
| ⌘K | App-wide | Toggles `appState.isSearchPresented` (opens/closes the overlay) | `ContentView.swift:38-40`, key char `"k"` at `:460-464` (confirmed) |
| ↑ | Overlay focused | `selectedIndex -= 1`, floored at 0 | `SearchOverlayView.swift:676-679` |
| ↓ | Overlay focused | `selectedIndex += 1`, capped at `emptyItemCount - 1` when `query.isEmpty`, else `flatResults.count - 1` | `:680-688` |
| Return / Enter | Overlay focused | If `query.isEmpty`: `navigateEmptyState(at: selectedIndex)` (§3). Else: `navigate(to: flatResults[selectedIndex])` if `selectedIndex` is in range, otherwise ignored | `:689-699`, `:1020-1036` |
| Escape | While overlay is presented | Dismisses the overlay. Implemented as a local `NSEvent` monitor on `keyDown` for AppKit virtual keycode 53, installed `onAppear` and removed `onDisappear` (not a SwiftUI `.onKeyPress(.escape)`) — an AppKit implementation detail; the Vue port only needs a standard `keydown "Escape"` listener while the overlay is mounted | `:665-674`, `:716-723` |
| Click on a result / nav-shortcut / recent-session row | Overlay | Same as pressing Enter with that row selected | `:856`, `:877-881`, `:897-903` |
| Click on the dimmed scrim | Overlay | Dismisses | `:641` |
| Selecting a mode in the "Match mode" menu | Overlay | Sets `searchModeRaw`; re-runs the search immediately (no debounce) | `:773-803`, `:713-715` |

Selection index resets to 0 on every query change (`:701`); there is no separate binding for
scrolling — `ScrollViewReader` auto-scrolls the selected row into view on `selectedIndex` change
(`:824-828`).

## 7. Result ordering and grouping

- Within a category, candidates are sorted by `score` **descending**
  (`items.sort { $0.score > $1.score }`, `:604`) — Swift's `sort` is stable, so ties keep the
  order the candidates were appended in (which follows the source array's own order; that
  ordering is out of scope for this file — see each category's provider spec).
- Each category is then truncated to its **first 5** results (`let maxPerGroup = 5`, `.prefix(maxPerGroup)`, `:600,607`) — this is the *only* per-category result cap; it applies uniformly to all
  16 categories, including the two quick actions competing inside `commands`/`mcpServers`.
  (`fileChanges`'s "50" is a separate, upstream cap on how many `file-history` session
  directories are scanned to *build* candidates, §1 row 4 — not a display cap.)
- Categories with 0 matches are dropped, never rendered with an empty header (`:602-603`).
- Category (group) order is always the fixed `SearchCategory.allCases` order from §1
  (`:601`), regardless of match counts or which categories are present.
- `flatResults`/`flatIndex` (`:176-188`) linearize the visible groups top-to-bottom for keyboard
  navigation and the `↵` badge — group 0's results occupy indices `[0, count0)`, group 1's occupy
  `[count0, count0+count1)`, etc.

## 8. Debounce and async loading

- **Typing debounce — 200 ms.** `onChange(of: query)` (`:700-709`) resets `selectedIndex` to 0,
  sets `isSearching = true` synchronously if the query is non-empty, cancels any prior
  `debounceTask`, and starts a new one that sleeps 200 ms (cancellable) before copying
  `query → debouncedQuery`.
- **Search trigger.** `onChange(of: debouncedQuery)` (`:710-712`) and `onChange(of: searchModeRaw)`
  (`:713-715`, **not debounced** — a mode switch re-searches immediately with the current
  `debouncedQuery`) both call `triggerSearch()`.
- **`triggerSearch()`** (`:190-204`): cancels any in-flight `searchTask`; trims whitespace from
  `debouncedQuery`; if the trimmed query is empty, synchronously clears `searchResults` and
  `isSearching` and returns. Otherwise spawns a `Task` that calls the synchronous
  `buildSearchResults(query:)`, checks `Task.isCancelled` before committing, then sets
  `searchResults` and `isSearching = false`.
- **Stale results persist during the debounce window.** `searchResults` is not cleared when
  `query` changes — only when a new search actually completes (or the query becomes empty). So
  mid-typing, the UI keeps showing the previous query's results until the debounced search
  finishes; the shimmer (`searchShimmer`, `:929-957`, 5 skeleton rows) is shown only when
  `isSearching && flatResults.isEmpty` (`:813-814`) — i.e. only when there are currently *zero*
  results to keep showing while a new search is pending.
- **No minimum query length.** Any non-empty (post-trim) query searches, including 1 character
  (handled by fuzzy mode's single-char fallback, §2.2).
- **Config-derived category data is loaded once per overlay session**, not re-fetched per
  keystroke or on file-system change (§1 preamble; `:675`, `:1038-1080`). A Heimdall `search`
  command that re-reads its providers on every call (no separate "open overlay" load step) does
  not need to replicate this staleness — it is a strict improvement, not a behaviour to port.

## 9. Persistence of the mode

`@AppStorage("globalSearchMode")` stores the raw string value (`"fuzzy"` or `"matchWord"`) in
`UserDefaults`, default `"fuzzy"` (`:139-140`); reads go through a safe fallback,
`SearchMode(rawValue: searchModeRaw) ?? .fuzzy` (`:142-144`), so an unrecognized/corrupt stored
value silently behaves as fuzzy. Heimdall's equivalent is `Prefs.globalSearchMode: "fuzzy" | "exact"`
(`docs/adr/0007-preferences-storage-and-migration.md:36`, default `fuzzy`) — same persistence
semantics, Rust-owned instead of `UserDefaults`-owned (`AI_INSTRUCTIONS.md`, "Preferences are owned
by Rust").

## 10. Memory: custom folders are not searched

Confirmed at §1 row 10: `loadConfigItems()` builds `memoryFiles` only from
`ClaudeConfigLoader.projectsWithMemory().flatMap { loadMemoryFiles(projectDirName:) }`
(`SearchOverlayView.swift:1054-1056`) — every project's own `.claude/memory`, but it never calls
`ClaudeConfigLoader.loadMemoryFiles(customFolder:)` (`ClaudeConfigLoader.swift:489`, confirmed),
which is the function that reads user-added custom memory folders
(`MemorySourcesStore.foldersKey = "customMemoryFolders"`, confirmed
`MemorySourcesStore.swift:15`). This is an **intentional, already-recorded gap**, not a new
finding: `docs/port-plan.md:111`, assumption **A-6** — "Search skipping custom memory folders...
[is] intentional. Kept at parity (rows S10, S15); ask upstream if it matters." `docs/parity-matrix.md:106`
(row S10) and `docs/tasks/M1-30-search-providers.md:22` already encode this. Heimdall's
`search/providers.rs` should likewise only index `MemorySource`s of kind `"project"`, skipping
`"folder"` sources, to stay at parity.

## 11. Open questions

- **`SearchOverlayView.swift:91-92`'s doc comment is imprecise.** It says fuzzy `"ssh"` "also
  matches `se**ss**ion`", but `"ssh"` is not an ordered subsequence of `"session"` (no `h`). The
  verified example from `HighlightedTextTests.swift:13-17` (`"ssh"` fuzzy-matching `"swordfish"`,
  §2.3) is the one this spec relies on; treat the `SearchOverlayView` comment as rhetorical, not
  literal.
- **`toggleMenuBar` / `addMCPServer` have no `SearchAction` representation in `ipc-contract.md`.**
  This spec documents their full Swift behaviour (§4, §5) so whichever task wires them up (after
  `M3-58`/`M3-48`) can add the IPC variants without re-reading Swift; this spec does not itself
  propose the new variant shapes.
- **Tie-breaking order within a category is inherited from each provider's own array order**,
  which this spec does not define (out of scope — each category's data source is a different
  Swift file / future Rust provider). Flagging so `search/providers.rs` authors pick a
  deterministic order (e.g. the same order `list_sessions`/`list_projects` etc. already use) and
  document it there rather than leaving it as an accident of iteration order.
- **UTF‑16 highlight offsets vs. Swift's grapheme-cluster indexing** (§2.4) — Swift's own
  `AttributedString` offsets are not UTF‑16 code units, so there is no Swift ground truth to
  literally port for non-BMP characters (emoji, combining marks) in titles; the Rust matcher must
  compute UTF‑16 offsets independently. Low risk in practice since matched titles are mostly
  session titles, file names, and config names.
