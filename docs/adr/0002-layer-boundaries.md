# ADR 0002: Layer boundaries

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

The Swift reference app has no single service boundary. Six environment keys inject five loaders and a provider (`Services/Environment+Services.swift`); `ClaudeConfigLoader` is a static enum; `AppState.swift:247-268` builds loaders directly; views such as `SearchOverlayView` and `TodosOverviewView` read the file system themselves. The locked data flow requires all parsing in Rust and a webview that only renders.

## Decision

Three layers, each with a hard rule.

### `crates/heimdall-core` — parsing and domain logic

- No `tauri` dependency. CI fails if `cargo tree -p heimdall-core` mentions tauri (M1-58).
- Every function takes the Claude home (and any other root) as a parameter, so tests run on fixture trees on any OS.
- Pure platform logic lives here and returns data: CLI candidate paths, launcher argv or URLs, path→area classification. Functions take a `TargetOs` argument instead of using `#[cfg]`, so every OS variant is tested on every CI host (ADR 0003).
- Modules, mapped 1:1 from Swift:

| Swift | heimdall-core module |
|---|---|
| `Services/SessionLoader.swift` | `projects/` (discovery, sessions_index, subagents, project_path, delete) |
| `Services/TranscriptParser.swift`, `Utilities/SystemContentParser.swift`, `Utilities/JSONBeautifier.swift` | `transcript/` (full, header, summary, blocks, searchable, system_content, tool_input) |
| `Services/Config/{ClaudeConfigLoader,FrontmatterParser}.swift` | `config/read/` |
| `Services/Config/SettingsWriter.swift`, write parts of `ClaudeConfigLoader` | `config/write/` (json_doc, atomic, settings, mcp, files) |
| `Services/Config/MCPServerStatusChecker.swift` | `mcp_status.rs` |
| `Services/{FileHistoryLoader,HistoryLoader,TodoLoader,DebugLogLoader,FacetsLoader,StatsCacheLoader,StatsComputer}.swift`, `Views/MenuBar/MenuBarState.swift` | `data/` (one file each, plus `tray.rs`, `history_write.rs`) |
| `Services/Export/SessionExporter.swift`, `Views/Analytics/Export/AnalyticsCSVExporter.swift` | `export/` |
| `Utilities/{TerminalLauncher,EditorLauncher}.swift` (pure parts) | `os_integration/` (cli_locate, launch, links) |
| `Utilities/LineDiff.swift` | `diff.rs` |
| `Utilities/HighlightedText.swift` (matching), search in `Views/Search/SearchOverlayView.swift` | `search/` (matcher, providers) |
| `Models/*.swift` | `models/` (serde + specta types shared with commands) |
| (new) | `paths.rs`, `jsonl.rs`, `classify.rs`, `error.rs` |
| `Services/UpdateChecker.swift` | none: replaced by the updater plugin in src-tauri (port-plan P-14) |
| `Services/Providers/`, `Models/Provider/`, `Protocols/` | none: see ADR 0008 |

### `src-tauri` — thin shell

- `commands/`: thin `#[tauri::command]` wrappers that validate ids, call core through `spawn_blocking`, and map to contract types.
- `state.rs` (cache), `watcher.rs`, `prefs.rs`, `events.rs`; `tray.rs`, `menu.rs`, `window.rs`, `updater.rs`; `platform/` executes what core computed, per OS with `#[cfg(target_os)]`.
- No trait with a single implementation.

### `src/` — Vue renders and holds view state only

- No path joins, no parsing of Claude Code data, no file-system or shell permissions.
- IPC only through `src/ipc/client.ts` over generated bindings (ESLint enforced, M1-04).
- One feature folder per Swift reference view folder (`src/features/<feature>/`), one Pinia store per feature boundary, all components `<script setup lang="ts">`. The Swift reference's `@Observable AppState` slices map to: sessions/projects (`features/session/store.ts`), navigation (Vue Router plus `useNavigation`, port-plan P-3), config detail (`features/config/store.ts`), search (`features/search/store.ts`), toasts (`stores/toasts.ts`), theme and font (`stores/theme.ts`), preferences (`stores/prefs.ts`).
- View-model logic over already-parsed data may live in stores (analytics range filtering, history date buckets; port-plan P-1).

## Consequences

- Every behaviour of the data layer is testable without Tauri or a webview.
- Commands stay trivial, so the IPC contract is the only coupling between Rust and Vue (ADR 0004).
- Some Swift logic is split: parsing moves to core, presentation moves to Vue; each task names both sides explicitly.

## Related

Port-plan §5.1, §6 · ADR 0003, 0004, 0006, 0008 · Tasks M1-01, M1-03, M1-04, M1-05, M1-58
