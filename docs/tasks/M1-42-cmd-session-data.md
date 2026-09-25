# M1-42 · Session data commands: todos, file history, diffs, debug log, facets

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Session-level data commands return contract-shaped data, with diffs computed in Rust off the UI thread.

## Inputs

- `docs/ipc-contract.md` — commands/session_data.rs

## Outputs

- `src-tauri/src/commands/session_data.rs` — modify

## Acceptance criteria

- [ ] Tests cover each command on fixtures/synthetic, including `diff_lines` truncation at 5000 lines.

## Dependencies

- M1-16 — Todo loader
- M1-17 — Debug log loader
- M1-18 — Session facets loader
- M1-19 — File history loader with cross-OS file matching
- M1-35 — Line diff port
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
