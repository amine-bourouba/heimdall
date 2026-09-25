# M1-38 · App state and the (mtime, size)-validated cache

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

App state holds the Claude home and a path-keyed cache validated by (mtime, size) on every read, plus a helper that runs disk work off the event loop.

## Inputs

- `docs/adr/0006-change-propagation-and-caching.md`
- `$POIROT/Poirot/Sources/App/AppState.swift:303, 370-395`
- `$POIROT/PoirotTests/App/AppStateCacheTests.swift`

## Outputs

- `src-tauri/src/state.rs` — modify

## Acceptance criteria

- [ ] Tests port the AppStateCacheTests semantics: a hit, a stale entry after the file changes, and explicit invalidation by area.
- [ ] All loaders run through `spawn_blocking` (reviewer checks).
- [ ] A Claude-home override for tests (`HEIMDALL_CLAUDE_HOME`) exists only under `cfg(debug_assertions)`; a test builds release config and proves it is ignored.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
