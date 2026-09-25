# M1-16 · Todo loader

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

The todos module loads `~/.claude/todos/<sessionId>[-agent-<agentId>].json` exactly as TodoLoader does.

## Inputs

- `$POIROT/Poirot/Sources/Services/TodoLoader.swift`
- `$POIROT/PoirotTests/Services/TodoLoaderTests.swift`

## Outputs

- `crates/heimdall-core/src/data/todos.rs` — fill the placeholder created by M1-05

## Acceptance criteria

- [ ] Carried-over TodoLoaderTests pass, including the agent-suffixed filename pattern.

## Dependencies

- M1-06 — Claude home resolution, path normalisation and comparison keys
- M1-09 — Serde models for stats, history, todos, facets, debug log and file history

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
