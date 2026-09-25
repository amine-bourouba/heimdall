# M2-11 · open_external_url command

> Milestone M2 · Workstream: Rust data core · Scope: MVP

## Goal

`open_external_url` opens only URLs the link policy allows, through the opener plugin's Rust API.

## Inputs

- `docs/adr/0010-webview-trust-boundary.md` — Links
- `docs/ipc-contract.md` — commands/launch.rs

## Outputs

- `src-tauri/src/commands/launch.rs` — modify: open_external_url only

## Acceptance criteria

- [ ] A test with an injected opener shows rejected URLs return `invalidInput` and never reach the opener.

## Dependencies

- M1-33 — External URL allowlist
- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
