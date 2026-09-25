# M1-37 · Freeze the IPC contract as compiled stubs and generated TypeScript bindings

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Every command and event in docs/ipc-contract.md exists in src-tauri with its exact name and types, and `src/ipc/bindings.ts` is generated from them by tauri-specta and committed.

## Inputs

- `docs/ipc-contract.md`
- `docs/adr/0004-ipc-contract-and-type-generation.md`
- `docs/adr/0005-error-model.md`
- `docs/adr/0010-webview-trust-boundary.md` — IPC shape

## Outputs

- `src-tauri/src/commands/mod.rs` — modify
- `src-tauri/src/commands/{app,prefs,sessions,session_data,history,config_read,config_write,mcp,analytics,tray,search,export,launch,update}.rs` — modify: typed stubs
- `src-tauri/src/events.rs` — modify
- `src/ipc/bindings.ts` — create: generated
- `scripts/check-bindings.mjs` — create

## Acceptance criteria

- [ ] Every command body returns `Err(AppError::…)` with message `not implemented: <command>` until its owning task lands.
- [ ] `node scripts/check-bindings.mjs` regenerates the bindings and fails on any diff.
- [ ] A test lists registered command names and compares them with the contract table.
- [ ] No command takes a raw filesystem path from the webview (reviewer checks against ADR 0010 rule 5).
- [ ] The progress-log entry records the pinned tauri-specta version and its maintenance status on the day (latest release date, open Tauri-2 compatibility issues).

## Dependencies

- M1-03 — Scaffold the Tauri app crate with final identity, CSP and module stubs
- M1-04 — Scaffold the Vite + Vue 3 + TypeScript + Tailwind v4 frontend
- M1-05 — Declare every heimdall-core module and implement AppError
- M1-08 — Serde models for messages, sessions and projects
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history
- M1-10 — Config models, catalogs and the redacting Secret type
- M1-11 — Typed Prefs model with per-OS defaults

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
