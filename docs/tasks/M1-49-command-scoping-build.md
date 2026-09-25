# M1-49 · Scope app commands with AppManifest and verify exposure

> Milestone M1 · Workstream: Tauri shell / OS integration · Scope: MVP

## Goal

`build.rs` declares every app command through `AppManifest::commands`, and a short note records, with evidence, how the pinned Tauri version exposes commands across windows.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Capabilities
- `docs/ipc-contract.md`

## Outputs

- `src-tauri/build.rs` — modify
- `docs/security/command-exposure.md` — create

## Acceptance criteria

- [ ] `build.rs` lists exactly the contract's commands (a test compares the lists).
- [ ] command-exposure.md cites the Tauri docs page and version, and records an experiment showing a command absent from a window's capability is refused.

## Dependencies

- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings

## Owning agent

rust-engineer

## Reviewer agent

security-engineer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
