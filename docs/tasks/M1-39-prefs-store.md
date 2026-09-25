# M1-39 · Preferences storage and preference commands

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Preferences load and save through the store plugin in the OS app-config directory, and the webview reaches them only through the preference commands.

## Inputs

- `docs/adr/0007-preferences-storage-and-migration.md`
- `$POIROT/Poirot/Sources/App/MemorySourcesStore.swift`
- `$POIROT/PoirotTests/App/MemorySourcesStoreTests.swift`
- `docs/ipc-contract.md` — commands/prefs.rs

## Outputs

- `src-tauri/src/prefs.rs` — modify
- `src-tauri/src/commands/prefs.rs` — modify

## Acceptance criteria

- [ ] `get_prefs`, `update_prefs`, `add_memory_folder`, `remove_memory_folder` and `pick_config_project` work; folder pickers are opened from Rust.
- [ ] `update_prefs` validates fontScale (0.75–1.5 in 0.05 steps) and every enum; invalid input returns `invalidInput`.
- [ ] A `schemaVersion` migration hook exists (v1 identity) with a test.
- [ ] Carried-over MemorySourcesStoreTests pass in Rust.
- [ ] Nothing is written under `~/.claude` (test on a temp home).

## Dependencies

- M1-11 — Typed Prefs model with per-OS defaults
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M1-38 — App state and the (mtime, size)-validated cache

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
