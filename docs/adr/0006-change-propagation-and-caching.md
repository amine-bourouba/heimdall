# ADR 0006: Change propagation and caching

- **Status:** Accepted
- **Date:** 2026-09-25

## Context

Poirot's watcher (`Services/FileWatcher.swift`) watches one folder, non-recursively, with a hand-rolled 1 s debounce. It is started from 13 sites in 8 files (`App/ContentView.swift`, `Views/Memory/MemoryListView.swift`, and the hooks, MCP servers, sub-agents, history, plans and TODOs screens; `docs/spec/claude-home-inventory.md` §4), and two of them create folders under `~/.claude` just to watch them. Every event triggers a full rescan of all projects. The transcript cache (`AppState.swift:303`) is never invalidated, because `clearCache()` has no callers outside tests. Whether new transcripts deep inside project folders are even noticed is unverified [A-7].

## Decision

1. **One recursive watcher** over `~/.claude` (`notify`, `RecursiveMode::Recursive`), plus `~/.claude.json` through its parent folder with a file filter, plus each custom memory folder (re-registered when preferences change). Debounced with `notify-debouncer-full`, starting at 1 s.
2. **Classify, invalidate, signal.** Core's `classify(path)` maps each changed path to an area and optional key (contract §7). Matching cache entries are dropped, and one coalesced `data-changed` event carries `[{area, key}]` for the whole debounce window. Events never carry paths for the webview to re-request.
3. **Stores reload only what is on screen.** Each Pinia store subscribes to its areas and reloads only if its data is visible; otherwise it marks itself stale and reloads on next view.
4. **Cache keyed by path, validated by (mtime, size) on every read.** A missed event makes data late, never wrong. Start with full re-parse on change; add tail-only parsing only if a measured budget fails.
5. **Atomic-rename saves count as changes.** `notify` reports them as remove plus create; both classify to the same area.
6. **Platform limits.** If recursive registration fails (Linux `fs.inotify.max_user_watches`) the watcher falls back to registering known subfolders only and emits `watcher-status` with mode `degraded`. On overflow or a rescan flag (Windows `ReadDirectoryChangesW` bursts), it emits area `all` and stores reload what is on screen.
7. **Idempotent reloads.** Heimdall's own writes echo back through the watcher. Reloads never write, so there is no loop.
8. **Heimdall never creates folders under `~/.claude`** just to watch them (Poirot created `~/.claude/debug`, `ContentView.swift:415-423`).

## Consequences

- Live updates cover every data area, including ones Poirot never refreshed.
- Correctness does not depend on the watcher; freshness does.
- Linux users with very large trees may see the degraded mode; the event tells them why.

## Related

`docs/ipc-contract.md` §7 · ADR 0002, 0005 · Tasks M1-34, M1-38, M1-47, M2-03
