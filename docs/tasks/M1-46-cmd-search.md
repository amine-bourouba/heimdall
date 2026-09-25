# M1-46 · Search command

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

`search(query, mode, scope)` returns grouped results from the core providers.

## Inputs

- `docs/ipc-contract.md` — commands/search.rs

## Outputs

- `src-tauri/src/commands/search.rs` — modify

## Acceptance criteria

- [ ] An integration test on fixtures/synthetic returns all 16 groups for a broad query.

## Dependencies

- M1-30 — Search providers for all 16 categories
- M1-38 — App state and the (mtime, size)-validated cache

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
