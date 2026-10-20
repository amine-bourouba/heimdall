# M1-47 · Recursive watcher with debounce, classification, fallback and data-changed events

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

One recursive watcher over `~/.claude`, `~/.claude.json` and the custom memory folders drops matching cache entries and emits one coalesced `data-changed` event per debounce window, degrading safely when platform limits are hit.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Services/FileWatcher.swift`
- `$SWIFT_REF/Poirot/Sources/App/ContentView.swift:108-135, 410-429`
- `$SWIFT_REF/Poirot/Sources/Views/Memory/MemoryListView.swift:325-356`
- `docs/adr/0006-change-propagation-and-caching.md`
- `docs/ipc-contract.md` — §7 Events

## Outputs

- `src-tauri/src/watcher.rs` — modify
- `src-tauri/tests/watcher.rs` — create

## Acceptance criteria

- [ ] On all three CI OSes: create, append, atomic rename-over and delete of `projects/<p>/<s>.jsonl` produce one coalesced event within 2.5 s containing the transcript and sessions areas.
- [ ] 1000 rapid writes produce at most 3 events.
- [ ] An injected watch-registration failure switches to per-directory registration and emits `watcher-status` with mode `degraded`; a notify rescan or overflow emits area `all`.
- [ ] `~/.claude.json` is watched through its parent directory with a file filter; custom memory folders are re-registered when prefs change.
- [ ] A write made through the safe-write primitive triggers one reload and no further writes (idempotency test).

## Dependencies

- M1-34 — Changed-path to data-area classifier
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-38 — App state and the (mtime, size)-validated cache
- M1-39 — Preferences storage and preference commands

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
