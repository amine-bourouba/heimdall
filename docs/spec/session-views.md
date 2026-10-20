# Session views — behaviour spec

Source: `$SWIFT_REF` at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Every fact below cites a Swift `file:line`. Vocabulary (`ContentBlock`, `SessionSummary`, `SessionDetail`, `MessagePage`, `DiffResult`, `TodoList`, `FileHistory`, `DebugLogPage`, `SessionFacets`, `WriteOutcome`) is `docs/ipc-contract.md` §5. Where a fact depends on a Swift file outside this task's Inputs (e.g. `ContextUsage.swift`, `ExportOptions.swift`), it is called out and left to **Open questions** rather than guessed.

Files read: `SessionsNavigationView.swift`, `ProjectSessionsView.swift`, `SessionDetailView.swift`, `SessionToolbarContent.swift`, `ToolBlockView.swift`, `ThinkingBlockView.swift`, `SystemContentView.swift`, `EditDiffView.swift`, `ConversationContextCard.swift`, `SessionTimelineView.swift`, `SessionFacetsCard.swift`, `SessionTodosView.swift`, `FileHistoryView.swift`, `DebugLogView.swift`, `ExportOptionsView.swift`, `SessionSkeletonView.swift`, plus `App/AppState.swift`, `Views/Components/ToastOverlay.swift`, `Models/SessionFacets.swift`, `Models/DebugLogEntry.swift`, `Models/SessionGroup.swift` for the state/behaviour these views depend on.

## 1. Numeric thresholds

| Constant | Value | Source | Effect |
|---|---|---|---|
| Message page size | 20 | `SessionDetailView.swift:7` (`pageSize`) | Initial `visibleCount`; each "load more" adds another 20 (`SessionDetailView.swift:300`) |
| Tool-result truncation (assistant `ToolBlockView`) | 80 lines | `ToolBlockView.swift:8` (`truncatedLineCount`) | `contentLines.count > 80` → shows first 80 lines + "Show all N lines" toggle (`ToolBlockView.swift:56-66,164-180`) |
| Tool-result truncation (user-bubble `ToolResultBlock`) | 80 lines | `SessionDetailView.swift:530` (`truncatedLineCount`) | Same pattern as above, applied to tool results rendered inside a user message bubble (`SessionDetailView.swift:560-568,633-649`) |
| Thinking-block truncation | 50 lines | `ThinkingBlockView.swift:6` (`truncatedLineCount`) | `lines.count > 50` → shows first 50 lines + "Show all N lines" toggle (`ThinkingBlockView.swift:29-41,98-114`) |
| Diff line cap | 5000 lines/side | `EditDiffView.swift:22` (`maxLines`) | If either `oldString` or `newString` has more than 5000 lines, both sides are truncated to the first 5000 before diffing (`EditDiffView.swift:82-93`) — this is the Swift-side analogue of `DiffResult.truncated`/`maxLineNumber` in the IPC contract |
| Toast duration (no URL) | 4 s | `ToastOverlay.swift:22` | Auto-dismiss timer |
| Toast duration (with URL) | 10 s | `ToastOverlay.swift:22` | Auto-dismiss timer; tapping the toast opens `toast.url` via `NSWorkspace.shared.open` and dismisses immediately (`ToastOverlay.swift:15-20`) — Heimdall must route this through `open_external_url`, never a raw open |
| Debug-log page size | 200 | `DebugLogView.swift:458` (`pageSize`) | Initial load and every "load more" fetch 200 entries (`DebugLogView.swift:460-489`) |
| Scroll-to-bottom trigger | distance from bottom > 100 pt | `SessionDetailView.swift:314-321` | Shows the FAB (`showScrollToBottom`) once the transcript is scrolled more than 100pt from the bottom |
| Scroll-to-bottom FAB size | 36×36 | `SessionDetailView.swift:339` | Also used for the avatar/icon column width (28pt, see below) |
| Bubble width | `(paneWidth − 2×Spacing.xxxl) × 0.75` | `SessionDetailView.swift:277`, `SessionSkeletonView.swift:44` | Both the live transcript and its skeleton use the same 75%-of-available-width formula |
| Avatar / gutter column width | 28 pt | `SessionDetailView.swift:793,816,832`, `SessionSkeletonView.swift:100` | Assistant avatar and the alignment spacer for 2nd+ segments in the same turn |
| Diff gutter width | `max(digits(maxLineNumber), 2) × 8 + 6` pt | `EditDiffView.swift:38-45` | Recomputed per diff from the largest old/new line number |
| Diff image export scale | 2.0 | `EditDiffView.swift:220` | `ImageRenderer.scale` when copying/saving the diff as PNG |
| Sessions list pane width | default 280, clamped 220–380 | `SessionsNavigationView.swift:16-17,56,58` (`@SceneStorage("sessionsListWidth")`, `PoirotSplitDivider(minPosition:220, maxPosition:380)`) | Draggable divider; persisted per window via `SceneStorage` |
| Context sparkline shown | only if `context.series.count > 1` | `ConversationContextCard.swift:19` | Otherwise the sparkline column is omitted entirely |
| Context ring size | 50×50 | `ConversationContextCard.swift:40` | `UsageGaugeRing` |
| Token-mix bar size | 150×6, capsule-clipped | `ConversationContextCard.swift:65-66` | |
| Sparkline size | 150×32 | `ConversationContextCard.swift:116` | |
| Timeline row height / min chart height | 36 pt per row, `max(entries.count × 36, 120)` | `SessionTimelineView.swift:143` | Gantt chart height |
| File-history pane widths | file list: min 220 / ideal 260 / max 320; detail: min 400 / ideal 600 | `FileHistoryView.swift:47,49` | `HSplitView` panes |
| Debug log window size | min 700×500, ideal 800×600 | `DebugLogView.swift:67-72` | |
| Debug log skeleton rows | 15, with a 15-entry width table and a 3-value level-badge width cycle `[45,38,50]` | `DebugLogView.swift:430-448` | See §16 |
| Export popover width | 300 pt | `ExportOptionsView.swift:63` | |
| Session-card reveal stagger (grid) | `min(index,7) × 0.04 s` | `ProjectSessionsView.swift:143-147` | Capped at 0.28 s |
| Session-card reveal stagger (list) | `min(index,9) × 0.03 s` | `ProjectSessionsView.swift:169-173` | Capped at 0.27 s |
| Session-card reveal delay/animation | 50 ms sleep, then 0.4 s ease-out | `ProjectSessionsView.swift:48-52` | Runs once per `project.id` |
| Skeleton message-bubble count | 6, alternating user (even index)/assistant (odd index) | `SessionSkeletonView.swift:48-54` | |
| Skeleton tool-block placeholder | shown only at assistant index 1 and 3 (of 6) | `SessionSkeletonView.swift:126-129` | |
| Font scale range | 0.75–1.5, step 0.05 | `AppState.swift:431-437` | `increaseFontScale`/`decreaseFontScale`/`resetFontScale`; not session-specific but affects all session text |

### Timers (copy/action feedback)

All "copied" icon flips reset after **1.5 s**, uniformly, at: `SessionDetailView.swift:746,961,977` (bubble copy/copy-as-markdown), `ToolBlockView.swift:288,320,354` (markdown copy, copy content, bash command copy), `ThinkingBlockView.swift:171`, `EditDiffView.swift:170,236,247` (save patch, copy image, copy diff text), `DebugLogView.swift:505` (copy full log). Two toolbar buttons use a longer reset:

| Button | Reset delay | Source |
|---|---|---|
| `SessionToolbarActions` "Resume" (copies `<cli> --resume <id>`) | 4 s | `SessionToolbarContent.swift:118-123` |
| `SessionToolbarActions` "Copy File Name" | 4 s | `SessionToolbarContent.swift:132-136` |
| `SessionToolbarActions` "Show in Finder" | 1.5 s | `SessionToolbarContent.swift:143-146` |
| Terminal launch after Bash-command/resume copy, when `openTerminalOnBash` is on | 2 s delay before opening | `SessionToolbarContent.swift:112-117`, `ToolBlockView.swift:347-352` |

## 2. Shared state and preferences these views read

### `AppState` (`App/AppState.swift`) fields used by the session views

| Field | Type / default | Source | Used by |
|---|---|---|---|
| `sessionSearchQuery` | `String`, `""` | `AppState.swift:196` | `SessionDetailView` message filter (`:40-43`); toolbar `SessionToolbarFilterField` binds it (`SessionToolbarContent.swift:39-47`) |
| `isSessionSearchActive` | `Bool`, `false` | `AppState.swift:197` | Set true/false whenever `sessionSearchQuery` becomes non-empty/empty (`SessionToolbarContent.swift:44-46`) |
| `isToolFilterActive` | `Bool`, `false` | `AppState.swift:199` | Shows/hides `SessionDetailView.toolFilterBar` (`SessionDetailView.swift:83-84`); toggled by `SessionToolbarFilter` (`SessionToolbarContent.swift:198-213`), tooltip advertises **⌘T** but no `.keyboardShortcut`/`.onKeyPress` exists in these files — the shortcut is wired at the app-menu layer (`AppAction.toolFilter` in `docs/ipc-contract.md` §7), out of this task's Inputs |
| `activeToolFilters` | `Set<String>`, `[]` | `AppState.swift:200` | Which tool names are checked in the filter bar; clearing `isToolFilterActive` also clears this set (`SessionToolbarContent.swift:200-203`) |
| `allBlocksExpanded` | `Bool`, `false` | `AppState.swift:301` | Toggled by `SessionToolbarExpandCollapse` (`SessionToolbarContent.swift:178-190`). **Only `ToolBlockView` observes live changes** via `.onChange(of: appState.allBlocksExpanded)` (`ToolBlockView.swift:201-203`); `ThinkingBlockView`, `SystemContentView` and the user-bubble `ToolResultBlock` only read the `autoExpandBlocks` default once, in `.onAppear` (see preferences table below) — toggling "Expand/Collapse All" after those blocks have mounted does **not** re-expand them in Swift. This asymmetry must be reproduced or consciously fixed in Vue; flagged again in Open questions. |
| `isShowingFileHistory` | `Bool`, `false` | `AppState.swift:198` | Drives `SessionsNavigationView.detailPane` (`:276`); reset to `false` whenever `selectedSession` changes (`AppState.swift:94`) |
| `showDebugLogSessionId` | `String?`, `nil` | `AppState.swift:219` | Set by `SessionToolbarDebugLog` (`SessionToolbarContent.swift:302-309`) to open `DebugLogView` (presentation site not in this task's Inputs) |
| `showAgentSessions` | `Bool`, persisted `UserDefaults["showAgentSessions"]` | `AppState.swift:293-297` | Sidebar toggle (`SessionsNavigationView.swift:139-156`); when off, `filteredSortedProjects` drops sidechain sessions and prunes empty projects (`AppState.swift:310-320`) |
| `projectSortOption` | `ProjectSortOption`, `.recentActivity` | `AppState.swift:56-68,299` | **Not persisted** (no `didSet`/`UserDefaults`, unlike `showAgentSessions`/`sessionLayout`) — resets to Recent Activity every launch. Drives the sidebar sort `Menu` (`SessionsNavigationView.swift:181-205`) and the sort branch of `filteredSortedProjects` (`AppState.swift:348-359`) |
| `sessionLayout` | `SessionLayout` (`grid`/`list`), persisted `UserDefaults["sessionLayout"]`, default `grid` | `AppState.swift:205-215` | `ProjectSessionsView` grid/list toggle (`ProjectSessionsView.swift:110-121,126-132`) |
| `isLoadingProjects` / `isLoadingMoreProjects` | `Bool`, both `true` initially | `AppState.swift:202-203` | `SessionsNavigationView` loading state (`:76-77`) and the sidebar refresh spinner vs. button (`:161-177`) |
| `isLoadingSession` | `Bool`, `false` | `AppState.swift:204` | Gates the skeleton in `detailPane`: shown when `appState.isLoadingSession || (session.messages.isEmpty && session.fileURL != nil)` (`SessionsNavigationView.swift:279-281`) |
| `selectedSession` | `Session?` | `AppState.swift:92-121` | Central selection; setting it resets `isShowingFileHistory` (`:94`) and pushes/updates navigation history; `SessionsNavigationView` also resets its local `showTimeline` to `false` on change (`SessionsNavigationView.swift:63-65`) |
| `selectedProject` / `currentProject` | `String?` / computed | `AppState.swift:123,305-308` | Drives `ProjectSessionsView` when no session is selected (`SessionsNavigationView.swift:292-294`) |
| `filteredSortedProjects` | computed `[Project]` | `AppState.swift:310-360` | Feeds `SessionsNavigationView`'s own further text-filter (`SessionsNavigationView.swift:19-40`) |
| `refreshID` | `UUID` | `AppState.swift:302` | Regenerated by the sidebar refresh button (`SessionsNavigationView.swift:167-169`); consumer (a `.task(id:)` or similar) is outside this task's Inputs |
| `toastQueue` / `showToast(...)` / `dismissCurrentToast()` | `[Toast]` | `AppState.swift:192,400-425` | FIFO queue; `showToast` wraps each line as Markdown `AttributedString`; append/remove animate over 0.25 s (`AppState.swift:415,422`) — see §1 for the 4 s/10 s auto-dismiss owned by `ToastOverlay` |
| `deleteSession(_:)` | method | `AppState.swift:443-464` | Deletes the file, clears selection/cache, removes the session (and empties the project if it was the last session) — invoked by `SessionToolbarDelete`'s confirmation |

### `UserDefaults`/`@AppStorage` keys read by these views

| Key | Type / default | Read at | Effect |
|---|---|---|---|
| `wrapCodeLines` | `Bool`, `true` | `SessionDetailView.swift:532` (`ToolResultBlock`), `ToolBlockView.swift:15`, `ThinkingBlockView.swift:9` | Toggles `ScrollView(.horizontal)` vs. wrapped `Text` for all three raw/code content viewers; each view has its own independent wrap toggle button but they all read/write the **same** key, so toggling one changes the others too |
| `textEditor` | `String`, `PreferredEditor.vscode.rawValue` | `ToolBlockView.swift:21` | Which editor the file-path "open externally" button launches (`ToolBlockView.swift:97-108`); `PreferredEditor` enum body is outside this task's Inputs |
| `openTerminalOnBash` | `Bool`, `false` | `SessionToolbarContent.swift:62`, `ToolBlockView.swift:23` | If true, copying a Resume command or a Bash tool command also opens the preferred terminal after a 2 s delay |
| `preferredTerminal` | `String`, `PreferredTerminal.terminal.rawValue` | `SessionToolbarContent.swift:65`, `ToolBlockView.swift:25` | Which terminal app `TerminalLauncher.open` targets |
| `parseMarkdownInResults` | `Bool`, `true` | `SystemContentView.swift:93` (`SystemBlockContentView`) | Whether nested system-content field values render as Markdown or plain monospaced text |
| `autoExpandBlocks` | `Bool`, `false` (raw `UserDefaults.standard.bool(forKey:)`, not `@AppStorage`) | `ToolBlockView.swift:199`, `ThinkingBlockView.swift:78`, `SystemContentView.swift:50`, `SessionDetailView.swift:661` (`ToolResultBlock`) | Sets each block's **initial** `isExpanded` on `.onAppear`. See the `allBlocksExpanded` row above for the live-update asymmetry |
| `fontScale` | `Double`, `1.0` | `AppState.swift:72-80` | Global; not session-specific |
| `showAgentSessions` / `sessionLayout` / `configProjectPath` / `configLayouts` | see `AppState` table | `AppState.swift:223-297` | Persisted via `AppState`'s own `didSet` blocks, not `@AppStorage` |

## 3. `SessionsNavigationView.swift`

Two-pane split: sessions list (left, resizable per §1) and a detail pane (right) chosen by a small state machine.

### Sessions list pane (`sessionsListPane`, `:70-109`)

- **Header** (`:111-135`): "All Sessions" label with `rectangle.stack.fill` icon; a capsule badge showing `filteredProjects.reduce(0){ $0 + $1.sessions.count }`; then three controls, in this order:
  1. **Agent-sessions toggle** (`:139-156`): icon `person.2.wave.2` (on) / `person.2` (off), tied to `appState.showAgentSessions`, animated over 0.3 s. Tooltip "Hide/Show agent sessions".
  2. **Refresh button** (`:159-177`): while `appState.isLoadingMoreProjects`, shows a mini `ProgressView` instead of the button; otherwise a button that sets `appState.refreshID = UUID()`. Tooltip "Refresh sessions".
  3. **Sort menu** (`:179-205`): a borderless `Menu` listing `ProjectSortOption.allCases` (Recent Activity / Name / Session Count), checkmark on the active option, sets `appState.projectSortOption` with a 0.3 s animation. Tooltip "Sort projects".
- **Search bar** (`sessionsListSearchBar`, `:207-238`): magnifying-glass icon, `TextField` bound to a **local** `listSearchQuery` (not `appState.sessionSearchQuery` — this filters the sidebar list, not the transcript), clear button (`xmark.circle.fill`) shown only when non-empty.
- **List body** (`:76-106`):
  - `appState.isLoadingProjects` → `loadingState`: small spinner + "Loading sessions…" (`:240-250`), fills available space.
  - else `filteredProjects.isEmpty` → `emptySearchState`: 24pt magnifying-glass (hierarchical) + "No sessions found" (`:252-264`).
  - else: `LazyVStack` of `SessionsProjectSection` per project, each `.id(project.id)`.
- `filteredProjects` (`:19-40`): starts from `appState.filteredSortedProjects`; if `listSearchQuery` is non-empty, keeps a project whole if its name fuzzy-matches, otherwise keeps only sessions whose `title` or `id` fuzzy-matches (via `HighlightedText.fuzzyMatch`), dropping projects with zero matches.

### Project section (`SessionsProjectSection`, `:347-444`)

- **Project header row** (`:396-443`): a chevron button that flips `isCollapsed` (rotates 90°, 0.2 s animation via the parent's `onToggleCollapse`), then a button showing a `shippingbox` icon, the project name (1 line), and a session-count capsule; tapping it sets `appState.selectedSession = nil; appState.selectedProject = project.id`.
- When not collapsed and `showAgentSessions` is on: sessions are grouped via `SessionGroup.group(sessions:)` (`Models/SessionGroup.swift:14-41` — groups by `parentSessionId`, non-sidechain sessions become parents, unmatched sidechains become standalone single-session groups, agents sorted by `startedAt` ascending, groups sorted by parent `startedAt` descending). A group with no agents renders as a plain `SessionsListRow`; a group with agents renders as `SessionGroupRow` with its own local `expandedGroups` set (toggle animates 0.2 s, `:374-384`).
- When `showAgentSessions` is off: every session in the project renders as a flat `SessionsListRow` (`:387-391`).

### Row components

- **`SessionsListRow`** (`:629-774`): title (fuzzy-highlighted against `searchQuery`), sidechain `cpu` icon prefix if `session.isSidechain`, optional agent-count pill (`cpu` icon + count + chevron, purple, only if `agentCount > 0`) that toggles `onToggleAgents`, relative time on the trailing edge; second line: model badge (abbreviated via `formatModel`, e.g. "claude-sonnet-4-5" → "Sonnet 4.5"), token-count badge (`formatTokens`, e.g. 12500 → "12.5k"), turn-count text; third line: `session.firstPrompt` (2-line clamp) if present. Selected/hover states swap background fill. Clicking sets `appState.selectedSession` (no-op if already selected).
- **`AgentSessionRow`** (`:516-625`): indented further (`Spacing.xl + Spacing.lg` leading padding vs. `Spacing.lg` for normal rows); icon and label colour depend on `agentType` (`explore` → blue/`magnifyingglass`, `plan` → green/`map`, else purple/`cpu`); shows `agentDescription` if present, else the fuzzy-highlighted title; second line shows token/turn counts only if `totalTokens > 0`.
- **`PoirotSplitDivider`** (`:448-481`): 1pt-wide draggable rule; hover shows `NSCursor.resizeLeftRight`; drag clamps `position` to `[minPosition, maxPosition]` = `[220, 380]`.

### Detail pane state machine (`detailPane`, `:268-298`)

Evaluated in this order:
1. No `selectedSession` and no `currentProject` → `noSessionSelectedState`: 48pt `rectangle.stack` (hierarchical), "Select a Session", "Choose a session from the list to view its content" (`:325-342`).
2. No `selectedSession` but `currentProject` set → `ProjectSessionsView(project:)` (§4).
3. `selectedSession` set, and:
   a. local `showTimeline == true` **and** the session belongs to a group with agents (`sessionGroup(for:)`, `:42-51`, computed by re-running `SessionGroup.group` per project and matching `parentSessionId ?? id`) → `SessionTimelineView` (§12), with a `timelineToggleButton` trailing accessory.
   b. else `appState.isShowingFileHistory` → `FileHistoryView` (§15).
   c. else `appState.isLoadingSession || (session.messages.isEmpty && session.fileURL != nil)` → `SessionSkeletonView` (§18).
   d. else → `SessionDetailView` (§5), with a `timelineToggleButton` overlaid top-trailing **only if** `sessionGroup(for: session) != nil`.
- `timelineToggleButton` (`:300-323`): 30×30 icon button, `text.bubble` (viewing timeline → back to transcript) / `chart.bar.xaxis` (viewing transcript → to timeline), toggles local `showTimeline` with 0.2 s animation.
- All pane switches use `.transition(.opacity)`.

## 4. `ProjectSessionsView.swift`

Shown when a project (not a specific session) is selected.

- **Header** (`:58-99`): `folder.fill` icon (green tint) in a 36×36 rounded tile, project name, a `"N session(s)"` capsule, and a static caption "Claude Code sessions for this project".
- **Toolbar** (`projectToolbar`, `:101-121`, via `.toolbar`): a `ConfigFilterField` (200pt wide, placeholder "Find in Sessions…", bound to a **local** `filterQuery`) and a layout-toggle button (`list.bullet` when grid is active, `square.grid.2x2` when list is active) that flips `appState.sessionLayout` with a 0.25 s animation.
- **Reveal animation** (`.task(id: project.id)`, `:46-53`): on project change, resets `filterQuery`, hides content (`isRevealed = false`), waits 50 ms, then reveals with a 0.4 s ease-out animation; each card/row applies `shimmerReveal(isRevealed:delay:cornerRadius:)` with the per-index stagger delays from §1.
- **Empty state**: if `filterQuery` is non-empty and `filteredSessions` is empty → `ConfigEmptyState(icon: "magnifyingglass", message: "No sessions match \"<query>\"", hint: "Try a different search term")` (`:34-39`); this component is outside this task's Inputs but its usage signature is captured here.
- **Filtering** (`filteredSessions`, `:12-26`): fuzzy-matches `title`, `preview`, and `id` via `HighlightedText.fuzzyMatch`, taking the best of the three scores per session, keeps only score > 0, sorted by score descending.
- **Content**: `appState.sessionLayout == .grid` → `sessionsGrid` (2-column masonry: `filteredSessions` alternately assigned to column `index % 2`, each column its own `LazyVStack`, `:136-160`) with `SessionCard`; else `sessionsList` (`:164-180`) with `SessionListRow`, both `ScrollView`-wrapped with `Spacing.xxl` horizontal padding.
- **`SessionCard`** (grid, `:185-234`): title (2-line), preview (4-line) if present, `SessionBadgeRow`, a 3pt accent-tinted leading bar (full accent when selected, 30% opacity otherwise), hover/selection background via `cardChrome(isHovered:isSelected:)` (outside this task's Inputs).
- **`SessionListRow`** (list, `:238-284`): same shape but title 1-line, preview 2-line, `Spacing.lg` padding instead of 18pt.
- **`SessionBadgeRow`** (`:288-343`): sidechain badge (`cpu` + `agentId ?? "Agent"`, purple) if `isSidechain`; model badge if `model != nil`; token-count badge if `totalTokens > 0`; always a turn-count badge; trailing relative time. Both card and row share this component.

## 5. `SessionDetailView.swift` — the transcript

### Header (`sessionHeader`, `:129-193`)

`rectangle.stack.fill` icon tile (36×36, accent), title (2-line clamp), then inline badges: model name (capsule) if `session.model != nil`, `"<formattedTokens> tokens"` capsule if `totalTokens > 0`. Second line: `"<project> · <timeAgo> · N turn(s)"` caption. If `ContextUsage.from(session.messages)` is non-empty (formula lives in `ContextUsage.swift`, outside this task's Inputs — see Open questions), a `ConversationContextCard` (§11) is shown below.

### Filtering (`filteredMessages`, `:36-67`)

Two independent filters, both applied to `session.messages`:
1. **Text search**: `appState.sessionSearchQuery` trimmed/lowercased; keeps messages whose `textContent.lowercased()` contains the query.
2. **Tool filter**: if `appState.activeToolFilters` is non-empty, keeps assistant messages containing at least one matching tool (`toolBlocks.contains { filters.contains($0.name) }`) **plus** the immediately preceding user message (for context), by index (`:52-60`).

`sessionToolNames` (`:69-78`) collects the ordered, de-duplicated set of tool names used by assistant messages in the session — this feeds the tool-filter bar's chip list.

### Tool filter bar (`toolFilterBar`, `:197-261`)

Shown only when `appState.isToolFilterActive`. Horizontally scrolling `HStack` of chips, one per name in `sessionToolNames`: icon (`provider.toolIcon(for:)`) + display name (`provider.toolDisplayName(for:)`), active state = accent fill/border, tapping toggles membership in `appState.activeToolFilters`. A trailing "Clear" text button appears when the set is non-empty and empties it. Rendered inside a `GlassBackground` capsule-corner rounded rect. `provider` (a `ProviderDescribing` environment value) is outside this task's Inputs — flagged in Open questions.

### Messages list (`messagesList`, `:265-358`)

- `GeometryReader` + `ScrollViewReader` + `ScrollView`. If `filteredMessages` is empty → `emptyState` (`:360-381`: `text.bubble` icon, "No messages to display", and `session.preview` if present, 6-line clamp). Otherwise:
  - An invisible 1pt anchor `"session-top"`.
  - `bubbleWidth = (paneWidth − 2×Spacing.xxxl) × 0.75`.
  - `visible = messages.prefix(visibleCount)`; `hasMore = visibleCount < messages.count`.
  - A `[toolUseId: ToolResult]` lookup built once from **all** `session.messages` (not just filtered/visible ones) so tool results still resolve under a filter (`buildToolResultLookup`, `:383-391`).
  - `LazyVStack` of `BubbleRow` per visible message, each keyed by array offset (not id — note for Vue `key`).
  - If `hasMore`: an invisible 1pt sentinel whose `.onAppear` sets `isLoadingMore = true` and does `visibleCount += pageSize (20)`; `.onDisappear` sets `isLoadingMore = false`. This is the infinite-scroll trigger — no explicit "Load more" button.
  - A trailing invisible 1pt anchor `"bottom"`.
- Scroll tracking: `.onScrollGeometryChange` computes `distanceFromBottom = contentHeight − offsetY − containerHeight`; `showScrollToBottom = distanceFromBottom > 100`.
- Overlay FAB (bottom-center, `:322-353`): shown when `showScrollToBottom || isLoadingMore`. While loading more, shows a small spinner; otherwise an `arrow.down` icon. Tapping scrolls to the `"bottom"` anchor with a 0.4 s ease-in-out animation. 36×36, glass-background circle.

### Bubble rendering (`BubbleRow`/`UserBubble`/`AssistantBubble`, `:396-922`)

- `role == .system` → renders nothing (`EmptyView`).
- **User bubble** (right-aligned, `:434-523`): header row "You · <time> · #<turnNumber>"; body is `SystemContentParser.parse(message.textContent)` split into `userText` (main text) and `systemBlocks` (see `SystemContentView`, §9) — parser itself is outside this task's Inputs. If `userText` is non-empty: highlighted `AttributedString` when a search query is active, else Markdown-rendered (`isFormatted == true` by default) or plain text (toggle). If `userText` is empty but the message has tool-result blocks, each renders as a `ToolResultBlock` (see truncation constant in §1). Any `systemBlocks` render via `SystemContentView` below the main content. Background `bgElevated`, border, rounded `Radius.md`. Bubble-hover reveals `BubbleActionButtons` top-trailing.
- **Assistant bubble** (left-aligned, `:772-853`): iterates `message.textAndToolSegments` (ordered runs of `.text`/`.tools`/`.thinking` — segment model is outside this task's Inputs but the ordering/avatar rule is captured here) by array offset:
  - `.text(text)` → `AssistantTextBubble` (header "<assistantName> · <time> · #<turnNumber>" only on the first segment; Markdown/plain/highlighted same three-way toggle as the user bubble; background `bgCard`).
  - `.tools(tools)` → filtered by `activeToolFilters` (empty filter = show all); each surviving tool renders as a `ToolBlockView` (§7), passing the matching `ToolResult` from the lookup by `toolUseId`. If the filtered list is empty, the whole row is omitted (no empty tool-block container).
  - `.thinking(text)` → `ThinkingBlockView` (§8).
  - Only the **first** segment in a message gets the avatar (`provider.assistantAvatarLetter` in a 28×28 tile); subsequent segments get a 28pt-wide blank spacer instead, so all segments align under the avatar column.

### Bubble action buttons (`BubbleActionButtons`, `:926-1017`, wrapped by `bubbleActions(...)`, `:1019-1064`)

Hover-revealed, top-trailing overlay on both user and assistant text bubbles (opacity 0.5 idle / 0.15 while hovering the bubble content / 1.0 while hovering the buttons themselves — this three-state opacity pattern recurs in every "hoverable content with overlay buttons" block in this spec, always `isButtonsHovered ? 1.0 : (isContentHovered ? 0.15 : 0.5)`):
1. **Format toggle** — `doc.plaintext`/`doc.richtext`, flips local `isFormatted` (Markdown ⇄ plain text).
2. **Copy as Markdown** — only shown when a `message` is bound (assistant/user text bubbles, not the standalone tool-result block); copies `SessionExporter.messageToMarkdown(message)` (outside this task's Inputs), 1.5 s checkmark.
3. **Copy raw text** — copies the bubble's plain text, 1.5 s checkmark.
User bubbles only show these buttons when `!parsed.userText.isEmpty` (`showActions:`, `SessionDetailView.swift:510`) — a tool-result-only user bubble has no bubble-level action buttons (the `ToolResultBlock` has its own, see §1/§7-style pattern at `:616-650`).

## 6. `SessionToolbarContent.swift` — toolbar catalogue

`SessionToolbar` (`:5-27`) is a `ToolbarItemGroup` at `.primaryAction`, in this exact order. There are **no `.contextMenu` (right-click) modifiers anywhere in the session views** — every action below is a toolbar button, an inline hover-revealed icon button on a bubble/block, or the sidebar's dropdown `Menu` (§3). Vue should implement all of these as visible buttons/menus, not native context menus.

| # | Component | Control | Effect | Preference keys read | Feedback |
|---|---|---|---|---|---|
| 1 | `SessionToolbarFilterField` (`:31-48`) | Text field, 200pt, placeholder "Find in session…" | Binds `appState.sessionSearchQuery`; `onChange` sets `appState.isSessionSearchActive = !query.isEmpty` | — | live filter (§5) |
| 2 | `SessionToolbarActions` → **Resume** (`:73-84,106-124`) | Button, label "Resume"/"Copied", icon `arrow.uturn.forward`/`checkmark` | Copies `"<provider.cliPath> --resume <session.id>"` to the pasteboard; if `openTerminalOnBash`, opens `preferredTerminal` after a 2 s delay; calls `appState.showToast("Copied \`<command>\`")` | `openTerminalOnBash`, `preferredTerminal` | icon+label flips for 4 s |
| 3 | `SessionToolbarActions` → **Copy File Name** (`:85-92,126-137`) | Icon button, `doc.on.doc`/`checkmark` | Copies `session.fileURL.lastPathComponent`; toast "Copied `<name>`" | — | flips for 4 s |
| 4 | `SessionToolbarActions` → **Show in Finder** (`:94-101,139-147`) | Icon button, `folder`/`checkmark` | `NSWorkspace.activateFileViewerSelecting([url])` → Heimdall: `reveal_in_file_manager` | — | flips for 1.5 s |
| 5 | `SessionToolbarFileHistory` (`:250-292`) | Icon button `clock.arrow.2.circlepath`, badge = file count | Toggles `appState.isShowingFileHistory`; `.disabled` when `fileCount == 0`; bounce symbol effect on tap; count loaded via `fileHistoryLoader.loadFileHistory(...).count` on `.task(id: session.id)` | — | tooltip `"File History (<n> files)"` |
| 6 | `SessionToolbarExport` (`:152-169`) | Icon button `square.and.arrow.up` | Toggles a popover showing `ExportOptionsView` (§17) | — | — |
| 7 | `SessionToolbarDebugLog` (`:296-310`) | Icon button `ladybug` | Sets `appState.showDebugLogSessionId = session.id` | — | — |
| 8 | `SessionToolbarExpandCollapse` (`:173-190`) | Icon button, `arrow.down.right.and.arrow.up.left`/`arrow.up.left.and.arrow.down.right` | Toggles `appState.allBlocksExpanded` (see the asymmetric live-update note in §2) | — | tooltip "Collapse All"/"Expand All" |
| 9 | `SessionToolbarFilter` (`:194-214`) | Icon button, `line.3.horizontal.decrease.circle(.fill)` | Toggles `appState.isToolFilterActive`; turning it off also clears `appState.activeToolFilters` | — | tooltip "Filter by Tool (⌘T)" |
| 10 | `SessionToolbarDelete` (`:218-246`) | Icon button `trash` → `confirmationDialog` | Destructive-role "Delete" button calls `appState.deleteSession(session)`; dialog message: "This will permanently delete the session file. This action cannot be undone." | — | native confirmation required (matches ADR 0010 rule 6 in the IPC contract) |
| 11 | `SessionToolbarClose` (`:314-327`) | Icon button `xmark` | `appState.selectedSession = nil; appState.selectedProject = nil` | — | — |

## 7. `ToolBlockView.swift`

Renders one `ToolUse` (+ optional `ToolResult`) inside an assistant bubble's `.tools` segment.

- **Header row** (always visible, `:70-134`): tool icon + display name (`provider.toolIcon/toolDisplayName`); if `tool.isBash` and it has a `command`, shows `"$ <command>"` (1-line, monospace); else if `tool.filePath` is set, shows the path (1-line, monospace, tertiary colour) plus an "open externally" button (`arrow.up.forward.square`) that calls `EditorLauncher.open(filePath:editor:)` using the `textEditor` preference — only shown for non-Bash tools with a file path. Trailing: status pill — Bash tools show "Exit 0"/"Exit 1", others show "Done"/"Error", green/red by `result?.isError`. Chevron rotates 90° when expanded. Tapping the whole header toggles `isExpanded` (0.2 s animation).
- **Expanded body** (`:135-191`), three mutually exclusive branches:
  1. `tool.hasDiffData` with both `oldString`/`newString` → `EditDiffView` (§10).
  2. else non-empty `result?.content` → a code/markdown viewer identical in structure to `ToolResultBlock` (§5's user-bubble variant): `JSONBeautifier.beautify` the content, truncate to 80 lines (see §1) with a "Show all N lines" footer button when truncated, `Markdown`/raw toggle (`isMarkdownContent = !tool.isBash`, i.e. Bash output never gets the Markdown option), line-wrap toggle, and a row of hover-revealed action buttons: bash-command-copy (Bash tools only, also opens terminal if `openTerminalOnBash`), Markdown-render toggle (non-Bash only), copy-as-Markdown (`SessionExporter.toolBlockToMarkdown`), line-wrap toggle, copy-content.
  3. else → `"No output"` in monospace tertiary text.
- Initial `isExpanded` comes from `autoExpandBlocks`; live-updates from `appState.allBlocksExpanded` via `.onChange` (§2).
- Border/background: rounded `Radius.md`, red-tinted border when `isError`, header background `bgElevated`, body background `bgCode`.

## 8. `ThinkingBlockView.swift`

A collapsed-by-default capsule chip, not a full block:

- **Header**: `brain.head.profile` icon (breathe symbol effect while expanded, gated by `!reduceMotion`), "Thinking" label, chevron rotates from −90° (collapsed) to 0° (expanded). Purple-tinted capsule (`purple.opacity(0.08)` fill, `purple.opacity(0.2)` border). Tapping toggles `isExpanded` (0.2 s, or no animation if `accessibilityReduceMotion`).
- **Expanded body**: monospace text, tertiary colour, truncated to 50 lines (§1) with "Show all N lines" footer when truncated; line-wrap toggle and copy button, same hover-opacity pattern as other blocks. Rounded `Radius.sm`, `bgCode` background, purple 15%-opacity border.
- Initial `isExpanded` from `autoExpandBlocks`; **no** `.onChange(of: appState.allBlocksExpanded)` — see the asymmetry note in §2.

## 9. `SystemContentView.swift`

Renders `SystemContentParser.SystemBlock` arrays extracted from a user message's text (parser itself outside this task's Inputs, but the parsed shape — `tagName` + `content` per block — is used directly here).

- A divider, then a collapsed-by-default pill: `info.circle` + `"System context (<count>)"`, chevron rotates 0°/−90°. Initial state from `autoExpandBlocks`; no live `allBlocksExpanded` binding.
- Expanded: one row per block — a small accent capsule with `block.tagName`, then `SystemBlockContentView(content: block.content)`.
- `SystemBlockContentView` recursively re-parses its own content (`SystemContentParser.parse(content)`); if that inner parse yields further `systemBlocks`, it renders the leftover `userText` (if any) plus each nested field (tag name in tertiary micro text + value); otherwise it renders the raw content directly. Every leaf value goes through `JSONBeautifier.beautify` and then, gated by the `parseMarkdownInResults` preference, either `Markdown(...)` or plain tertiary text.

## 10. `EditDiffView.swift`

Renders a two-sided string diff (used both for `ToolBlockView`'s edit tools and `FileHistoryView`'s version comparisons).

- `diffIdentity = "<filePath>|<oldString.hashValue>|<newString.hashValue>"` is the `.task(id:)` key — recomputing only when the actual content changes, not just the view identity (explicit code comment at `:24-25` warns this was a real bug source).
- While the diff task is running (`computedDiff == nil`), shows a small `ProgressView` (min height 40).
- Diffing itself: truncate both sides to 5000 lines each if either exceeds it (§1), then `LineDiff.diff(oldLines:newLines:)` (algorithm outside this task's Inputs — Heimdall's Rust `diff_lines`/`diff_file_versions` commands already own this per the IPC contract).
- Each `DiffLine` renders as: old-line-number gutter, new-line-number gutter (both right-aligned, width from the gutter formula in §1), a 20pt-wide `+`/`-`/` ` indicator, then the text. Colours: context = secondary/tertiary text, add = `diffAddText`/`diffAddBg`, remove = `diffRemoveText`/`diffRemoveBg`.
- Horizontally scrollable (`ScrollView(.horizontal)`), no line wrapping.
- Hover-revealed action buttons (top-trailing, same 3-state opacity pattern): **Save as .patch** (`SessionExporter.toPatch` + native save panel), **Copy as image** (renders a separate `DiffSnapshotView` at 2× scale to PNG, copies to pasteboard, also opens a save panel), **Copy diff text** (`LineDiff.unifiedText(from:)`). All three give 1.5 s checkmark feedback.

## 11. `ConversationContextCard.swift`

A 3-column card shown in the session header only when context data is non-empty (gate is in `SessionDetailView`, §5).

- **Context column**: `UsageGaugeRing` (50×50) driven by `context.fraction * 100`; label "Context window", `"<current> / <window>"` (formatted via `AnalyticsFormatters.formatLargeNumber`, outside this task's Inputs), and `"<Int(cacheHitRate*100)>% from cache"`.
- **Divider** (44pt tall, 40% opacity).
- **Token-mix column**: a 150×6 capsule-clipped stacked bar with three segments in this order — cache (`cacheReadTokens + cacheCreationTokens`, blue), input (teal), output (accent) — each segment's width is `barWidth × value/total` where `total = max(1, input+output+cache)`; a legend row below with a coloured dot + label + formatted count per segment.
- **Sparkline column** (only if `context.series.count > 1`, §1): label `"Context over <n> turns"`, then `ContextSparkline` — a `Shape` that normalizes each value in `series` against `ceiling = context.window` and draws a polyline, stroked at 1.5pt with the ring's severity tint (`ringWindow.severity.tint`).
- `context.fraction`, `context.cacheHitRate`, `UsageWindow`, and `UsageGaugeRing`'s severity mapping all live in files outside this task's Inputs (`ContextUsage.swift` and shared gauge components) — see Open questions.

## 12. `SessionTimelineView.swift`

A Gantt-style view of a session group (parent + sub-agents), reached via the timeline toggle (§3).

- **Entries** (`:22-52`): one per group member — the parent (label = `title`) plus each agent (label = `agentDescription ?? agentType ?? title`). `endDate` falls back to `Date()` (now) if the session hasn't ended yet, so in-flight agents show as bars extending to "now".
- **Header** (`:71-92`): "Session Timeline" label, `"<agentCount> agents"` purple pill, then the caller-supplied `trailing` view (the timeline/transcript toggle button from §3).
- **Gantt chart** (`:96-153`, Swift Charts `BarMark`): x = start/end date, y = entry label; bar colour = accent for the parent, else blue (`explore`)/green (`plan`)/purple (anything else) by `agentType.lowercased()`; hovered bar (`hoveredEntry`, tracked but not wired to a gesture in this file) gets full opacity vs. 0.85 otherwise. X-axis ticks show `hour:minute:second`; both axes use dashed gridlines. Chart height = `max(entries.count × 36, 120)` (§1). Tapping anywhere in the chart area resolves the tapped x-date to the entry whose `[startDate, endDate]` contains it and sets `appState.selectedSession = tapped.session` (`:128-142`) — this is the one "click to navigate" interaction in this view.
- **Legend** (`:157-175`): four static swatches — Parent (accent), Explore (blue), Plan (green), Other (purple).

## 13. `SessionFacetsCard.swift`

AI-generated per-session summary (`SessionFacets`, `Models/SessionFacets.swift`), shown above the transcript when present (§5 gate: `if let facets`).

- **Header** (default **expanded = true**, `:8`, `:33-67`): `sparkles` icon (purple), "AI Summary" label, then an **outcome badge** (`facets.outcomeLabel`: success→green, partially_achieved→orange, failure→red, else tertiary — computed from the raw `outcome` string, `SessionFacetsCard.swift:233-240`) and a **helpfulness badge** (`claudeHelpfulness`: very_helpful→green, slightly_helpful→accent, not_helpful→orange, harmful→red, else tertiary, `:242-249`), then a spacer, a **session-type badge** (plain tertiary text), and a chevron that rotates 90° when expanded (0.2 s, reduce-motion aware).
- **Expanded content** (`:71-100`), each part shown only if non-empty/non-zero:
  - `briefSummary` rendered as Markdown.
  - **Goal** section: `target` icon label "Goal" + `underlyingGoal` text, shown if `underlyingGoal` is non-empty.
  - **Goal categories**: a `FlowLayout` (custom wrapping layout, `:256-307` — simple left-to-right flow that wraps to a new line when a chip would overflow `proposal.width`) of chips, one per `sortedGoalCategories` (sorted by count desc), label = `SessionFacets.categoryLabel(name)` (snake_case → Capitalized Words), with a count suffix if `count > 1`.
  - **Friction** section (only if `totalFrictionCount > 0`): `exclamationmark.triangle` icon label "Friction", a `FlowLayout` of orange chips from `sortedFrictionItems` (same shape as goal categories, via `SessionFacets.frictionLabel`), and `frictionDetail` text below if non-empty.
- Card chrome: `bgCard` background, `Radius.md` border/clip.

## 14. `SessionTodosView.swift`

Session's todo list, shown above the transcript when non-empty (§5 gate).

- **Header** (default **expanded = true**, `:7`, `:36-76`): `checklist` icon (accent), "Todos" label, a **status summary** (`:80-92`) of up to three small pills — completed count (green), in-progress count (accent), pending count (tertiary), each shown only if > 0 — then a spacer, a total-count badge, and a rotating chevron (0.2 s, reduce-motion aware).
- **Expanded list** (`:108-121`): one row per todo, thin dividers between rows (indented under the status icon's width, 15% opacity).
- **Row** (`:123-167`): status icon (completed = filled green checkmark with a bounce symbol effect on status change; in-progress = accent dotted circle with a breathe effect gated by reduce-motion; pending = plain tertiary circle), then `content` text (strikethrough + tertiary colour when completed), and — only when `status == .inProgress` — a second line showing `activeForm` in accent colour.

## 15. `FileHistoryView.swift`

Full-pane file-backup browser, replacing the transcript when `appState.isShowingFileHistory` is true (§3).

- **Header** (`:61-98`): `clock.arrow.2.circlepath` icon tile (36×36, accent, bounce effect keyed to an `iconBounce` counter incremented on load), "File History" title, `"<n> file(s) changed"` subtitle, close button (`xmark.circle.fill`) sets `appState.isShowingFileHistory = false`.
- **Body** (`:36-57`): `isLoading` → `loadingState` (spinner + "Loading file history…"); else `entries.isEmpty` → `emptyState` (`doc.text.magnifyingglass` icon, "No File History", "No versioned file snapshots were found for this session."); else an `HSplitView` — file list (§1 widths: min 220/ideal 260/max 320) beside version detail (min 400/ideal 600).
- **File list row** (`fileRow`, `:114-167`): file-type icon (by extension — `.swift`→`swift`, JS/TS family→`curlybraces`, `.json`→`curlybraces.square`, `.py`→chevron-slash icon, `.md`/`.txt`→`doc.text`, YAML/TOML→list icon, HTML/CSS→`globe`, shell scripts→`terminal`, Rust/Go/Ruby/Java/Kotlin/C/C++→chevron-slash icon, else→`doc`, full mapping `:384-398`), file name (last path component) + directory path (if nested) on a second line, and an edit-count badge. Selecting sets `selectedFile` and resets `selectedVersionIndex` to `nil` (defaults to the latest version).
- **Version timeline** (`versionTimeline`, `:186-236`): horizontally scrolling row of version chips (`v<n>` + backup time `HH:mm:ss`), selected chip = the explicit `selectedVersionIndex` or, if `nil`, the last version. Tapping a chip selects it and triggers `loadVersionContent`.
- **Diff content** (`diffContent`, `:238-284`): for the first version (`effectiveIndex == 0`), shows a full "added" diff (`EditDiffView(oldString: "", newString: <content>)`) once loaded; for later versions, shows the diff between the previous and current version once **both** are loaded (`loadedContentKeys` tracks which backups have been fetched, distinguishing "not loaded yet" from "legitimately empty" — explicit comment at `:18-22` about a prior bug). While waiting, shows a centered `ProgressView`.
- **Placeholder state** (no file selected, `:319-331`): `doc.on.doc` icon (40% opacity) + "Select a file to view changes".
- Data loading (`:335-380`): `fileHistoryLoader.loadFileHistory(for:projectPath:)` on `.task`; auto-selects the first file and its latest version on load.

## 16. `DebugLogView.swift`

A modal-style window/sheet (min 700×500, ideal 800×600, §1) — presentation site (sheet/window) is not in this task's Inputs, only its own content.

- **Header** (`:81-145`): `ladybug.fill` icon tile (bounce effect keyed to `entries.count`), "Debug Log" title, `"<totalCount> entries"`, plus conditional error-count (red, `exclamationmark.triangle.fill`) and warn-count (orange, `exclamationmark.circle.fill`) pills computed from the **currently loaded** `entries` (not `totalCount` — so these counts grow as more pages load). Trailing: copy-log button (1.5 s checkmark feedback, copies **all** entries re-fetched via a fresh unpaged `DebugLogLoader(...).loadEntries(for:)` call, formatted `"<HH:mm:ss.SSS> [<LEVEL>] <message>"` per line) and a dismiss (`xmark`) button using `@Environment(\.dismiss)`.
- **Toolbar row** (`:194-331`): search field (max width 300, placeholder "Filter log entries...", live `"<filtered>/<total>"` counter and a clear button when non-empty) on the left; on the right, three **level filter toggle chips** (Debug/Warn/Error, from `DebugLogEntry.Level.allCases` — `Models/DebugLogEntry.swift:10-16`, default **all three selected**) each independently togglable, and a relative/absolute time toggle button (`clock.arrow.circlepath`/`clock`).
- **Filtering** (`filteredEntries`, `:45-57`): entry must have `selectedLevels.contains(entry.level)`; if `searchText` is non-empty, additionally must contain the (lowercased) query in `message` or in the level's raw string.
- **List** (`:339-394`): `isLoading` → 15-row `logSkeleton`; else `filteredEntries.isEmpty` → `emptyState` (two variants: "No debug log found" when `entries` itself is empty, vs. "No matching entries" when filters excluded everything); else a `LazyVStack` of `DebugLogEntryRow`, with an infinite-scroll trigger on the last visible row's `.onAppear` (`hasMoreEntries = entries.count < totalCount`) that calls `loadMoreEntries()`, plus a trailing spinner row while `hasMoreEntries` stays true.
- **Skeleton** (`logSkeleton`, `:430-448`, §1): 15 rows, each a shimmering rectangle triplet — 80×10 (timestamp), a level-badge rect cycling widths `[45,38,50]` by `index % 3` at height 16, and a message-width rect from a fixed 15-value width table (`skeletonMessageWidths`, `:430-432`) at height 10.
- **Entry row** (`DebugLogEntryRow`, `:531-652`): timestamp column (90pt, trailing-aligned) — absolute `HH:mm:ss.SSS` or, if `useRelativeTime`, `+<offset>` formatted as `m:ss.SSS` once past a minute or `s.SSSs` otherwise (`formatOffset`, `:637-651`), relative to the **first loaded** entry's timestamp; a 42pt level badge (tinted by level); the message (search-highlighted when `searchQuery` is set), selectable text, unlimited line count. Row background tints red/orange (with a slightly-stronger hover shade) for error/warn levels, or a neutral hover tint otherwise.
- **Loading** (`:458-489`): page size 200 (§1); on the very first load, if the loaded page contains an error entry, the view auto-scrolls to the **first** error in that page (`scrollTarget`/`firstErrorId`, `:471-475`) — note this only looks at the first fetched page, not the whole log.

## 17. `ExportOptionsView.swift`

Popover content anchored to the toolbar's Export button (§6, item 6). Fixed width 300pt (§1).

- Header: `square.and.arrow.up` icon + "Export Session".
- Four checkbox toggles (`.toggleStyle(.checkbox)`), bound to a local `ExportOptions()` instance (default field values live in `ExportOptions.swift`, outside this task's Inputs — flagged in Open questions; the field names map 1:1 to the IPC contract's `ExportOptions` type):
  - "Include timestamps" → `includeTimestamps`
  - "Include tool results" → `includeToolResults`
  - "Include thinking blocks" → `includeThinking`
  - "Include token usage" → `includeTokenUsage`
- Divider, then two actions: **Copy** (bordered button) → `SessionExporter.toMarkdown(session, options:)`, copies to pasteboard, `appState.showToast("Copied session as Markdown")`, dismisses the popover; **Export…** (prominent button) → same Markdown generation, then `SessionExporter.presentMarkdownSavePanel(content:sessionTitle:)` (a native save dialog — Heimdall's equivalent is the `export_session` command with `destination: "file"`), dismisses the popover.

## 18. `SessionSkeletonView.swift`

Loading placeholder shown in place of `SessionDetailView` while `appState.isLoadingSession` or the session has no messages yet but a file (§3, branch c). `.scrollDisabled(true)` — this skeleton never scrolls.

### Header skeleton (`headerSkeleton`, `:16-38`)

Same padding as the real header (`Spacing.xxxl` horizontal, `Spacing.lg` vertical, bottom divider). Contents, left to right / top to bottom:
- A 340×16 title bar.
- A row of: 100×18 pill (radius `xs`), 110×18 pill, 130×14 bar, a spacer, then four action-button placeholders sized 80×28, 110×28, 120×28, 70×28 (all radius `sm`) — standing in for the toolbar-area chips/buttons.

### Messages skeleton (`messagesSkeleton`, `:42-59`)

`GeometryReader` computing `bubbleWidth` with the same 75%-of-width formula as the live view (§1). A `ScrollView` (visually inert since scrolling is disabled at the root) containing 6 bubble placeholders (`Spacing.lg` gap, `Spacing.xxxl` outer padding), alternating by index parity: even index → user-bubble skeleton, odd → assistant-bubble skeleton (`:48-54`).

### User bubble skeleton (`userBubbleSkeleton`, `:63-91`)

Right-aligned (leading `Spacer`). `innerWidth = maxWidth - 28` (accounts for `Spacing.lg` padding both sides). Header row: 28×12 bar + 50×10 bar (stand-ins for "You" + timestamp). Then 2–3 text-line bars sized `innerWidth × ratio`, ratios by bubble index (`userLineRatios`, `:150-157`):
- index 0: `[0.85, 0.65]`
- index 2: `[0.75, 0.55]`
- index 4: `[0.70, 0.80, 0.40]`
- default (any other even index): `[0.90, 0.60]`

Card chrome: `bgElevated` fill, `Radius.md`, bordered, each line/placeholder has a `shimmer(cornerRadius:)` sweep effect.

### Assistant bubble skeleton (`assistantBubbleSkeleton`, `:95-134`)

Leading avatar placeholder 28×28 (radius `sm`). Text-bubble block: header row 44×12 + 50×10 bars ("Claude" + timestamp stand-ins), then 2–3 line bars using `assistantLineRatios` (`:159-166`):
- index 1: `[0.90, 0.85, 0.50]`
- index 3: `[0.85, 0.95, 0.65]`
- index 5: `[0.80, 0.70]`
- default: `[0.90, 0.75, 0.55]`

Card chrome: `bgCard` fill, `Radius.md`, bordered. **Only at index 1 and index 3** (of the 6 total bubbles), an extra tool-block placeholder is appended below the text bubble: a single 44pt-tall shimmering rect at full `maxWidth`, radius `md` (`:126-129`) — standing in for one collapsed `ToolBlockView` header row.

### Shared helper (`skeletonRect`, `:138-148`)

Every placeholder is a `RoundedRectangle` filled `bgCard`, sized `width×height` (or `maxWidth: .infinity` when `width` is `nil`), with a `shimmer(cornerRadius:)` modifier — the shimmer sweep implementation itself (`View.shimmer(...)`) is outside this task's Inputs; treat it as a generic "loading skeleton shimmer" animation, not one of the Lucide/AppIcon motion effects in `AI_INSTRUCTIONS.md`.

## 19. `ToastOverlay.swift` (cross-cutting)

Not one of the session-specific views, but every "Copied…" toast in this spec (§6) renders through it, so it is documented here for completeness.

- Shows only `appState.toastQueue.first` (FIFO, one at a time) — see `AppState.showToast`/`dismissCurrentToast` in §2.
- `toastCard`: icon (`toast.icon` override or `toast.style.defaultIcon` — success=`checkmark.circle.fill`/green, error=`xmark.circle.fill`/red, info=`info.circle.fill`/blue), message text (Markdown-parsed per-line, center-aligned), capsule chrome (`GlassBackground` + 15%-opacity style-colour fill, 30%-opacity style-colour border).
- Tapping the toast: if `toast.url` is set, opens it (`NSWorkspace.shared.open` — Heimdall must route through `open_external_url`), then always dismisses immediately regardless of the timer.
- Auto-dismiss timer: **10 s if `toast.url != nil`, else 4 s** (§1) — restarted via `.task(id: toast.id)` so it's cancelled/redone if the toast identity changes.
- Enter/exit transition: move from top edge + fade, animated over 0.25 s keyed to `currentToast?.id`. `allowsHitTesting` is false when no toast is shown (so the overlay doesn't block clicks).

## Open questions

1. **`ContextUsage.swift` is not in this task's Inputs.** `ConversationContextCard` (§11) consumes `context.fraction`, `context.cacheHitRate`, and `context.series`, but the exact formulas (how `fraction`/`cacheHitRate` are derived from the IPC contract's raw `ContextUsage` fields) live in that model file. The Vue task implementing this card needs those formulas from `ContextUsage.swift` directly, or a follow-up spec task should extract them.
2. **`UsageGaugeRing`/`UsageWindow`/`AnalyticsFormatters.formatLargeNumber`** (used by `ConversationContextCard`) and **`ProviderDescribing`** (`provider.toolIcon`/`toolDisplayName`/`assistantAvatarLetter`/`assistantName`/`cliPath`, used throughout §5–§7) are shared components/protocols outside this task's Inputs. Their exact behaviour (icon mapping, severity thresholds, number formatting) should be pulled from their own source files before the corresponding Vue components are built.
3. **`ExportOptions.swift`'s default toggle values** are not in this task's Inputs — `ExportOptionsView` (§17) instantiates `ExportOptions()` with defaults defined elsewhere. Confirm the four booleans' defaults before wiring the Vue export popover.
4. **`SessionExporter`, `LineDiff`, `JSONBeautifier`, `SystemContentParser`, `EditorLauncher`, `TerminalLauncher`, `HighlightedText`, `DebugLogLoader`, `TodoLoader`/`FacetsLoader`/`FileHistoryLoader`** are referenced by name throughout (e.g. `SessionExporter.toolBlockToMarkdown`, `SessionExporter.toPatch`, `LineDiff.diff`/`unifiedText`) but their internals are out of scope here; this spec only documents call sites and effects, not algorithms. Rust already owns the equivalent logic per the IPC contract's `diff_lines`/`diff_file_versions`/`export_session`/`export_edit_patch` commands.
5. **The `allBlocksExpanded` live-update asymmetry (§2, §6 item 8, §7/§8/§9)** is almost certainly an existing the Swift reference app bug (only `ToolBlockView` reacts live; thinking/system/tool-result blocks don't), not an intentional design. Decide explicitly whether Heimdall should reproduce the bug for pixel-parity or fix it — flag to `qa-expert`/product owner before the Vue task locks its behaviour.
6. **`⌘T` tool-filter shortcut** (`SessionToolbarContent.swift:212`) has no `.keyboardShortcut`/`.onKeyPress` in any file read for this task; it's presumably wired through the app's `Commands`/menu scene, which maps to `AppAction.toolFilter` in `docs/ipc-contract.md` §7. Confirm the binding site before the Vue keyboard-shortcut task assumes this key.
7. **`ConfigFilterField`, `ConfigEmptyState`, `GlassBackground`, `cardChrome(isHovered:isSelected:)`, `shimmerReveal(...)`** are shared UI components used by these views but not defined in them; their exact visuals should come from wherever those components are specified (likely a separate shared-components task), not re-derived here.
8. **`SessionGroup.group`'s "orphan sidechain" fallback** (`Models/SessionGroup.swift:22-26,35-38`) treats any sidechain session whose `parentSessionId` doesn't match a loaded parent as its own standalone one-member group. Confirm this is still correct once Rust's session loader (M0-05/M1-41) defines exactly which sessions are loaded together — a parent living in a different project file, for instance, would produce an orphan today.
