# M1-44 · Config read commands

> Milestone M1 · Workstream: Rust data core · Scope: MVP

## Goal

Config read commands return commands, skills, plans, output styles, agents, hooks, plugins, models and memory for global scope, a known project, or the picked folder.

## Inputs

- `docs/ipc-contract.md` — commands/config_read.rs
- `docs/adr/0010-webview-trust-boundary.md` — IPC shape rule 5

## Outputs

- `src-tauri/src/commands/config_read.rs` — modify

## Acceptance criteria

- [ ] Project scope resolves only a known project id or the folder chosen through `pick_config_project`; anything else returns `invalidInput`.
- [ ] Tests cover each command on fixtures/synthetic.

## Dependencies

- M1-24 — Loaders for commands, skills, plans, output styles and agents
- M1-25 — Settings and hooks readers
- M1-26 — MCP server (five sources) and plugin readers
- M1-27 — Memory and models readers
- M1-38 — App state and the (mtime, size)-validated cache
- M1-39 — Preferences storage and preference commands

## Owning agent

rust-engineer

## Reviewer agent

architect-reviewer, security-auditor

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
