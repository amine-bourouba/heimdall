# M4-04 · Update commands

> Milestone M4 · Workstream: Rust data core · Scope: MVP

## Goal

`check_for_update` and `install_update` delegate to the updater module and return contract-shaped results, with release-notes URLs checked by the link policy.

## Inputs

- `docs/ipc-contract.md` — commands/update.rs
- `docs/adr/0010-webview-trust-boundary.md` — Links

## Outputs

- `src-tauri/src/commands/update.rs` — modify

## Acceptance criteria

- [ ] A release-notes URL that fails the link policy is dropped from the result (test).
- [ ] Errors map to `AppError` kinds, never panics.

## Dependencies

- M1-37 — Freeze the IPC contract as compiled stubs and generated TypeScript bindings
- M4-03 — Updater plugin wiring

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
