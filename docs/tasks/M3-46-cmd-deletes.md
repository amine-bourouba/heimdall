# M3-46 · Delete commands for history, sessions and projects

> Milestone M3 · Workstream: Rust data core · Scope: Later

## Goal

History, session and project delete commands confirm natively and then delete through core.

## Inputs

- `docs/ipc-contract.md` — commands/history.rs, commands/sessions.rs

## Outputs

- `src-tauri/src/commands/history.rs` — modify: deletes
- `src-tauri/src/commands/sessions.rs` — modify: deletes

## Acceptance criteria

- [ ] Declining returns `cancelled`; tests use an injected dialog.

## Dependencies

- M1-41 — Session read commands
- M1-43 — History read command
- M3-43 — Core history, session and project deletes

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
