# M1-59 · M1 exit gate review

> Milestone M1 · Workstream: Testing · Scope: MVP

## Goal

The M1 exit criteria are checked per OS and the data-source and watcher rows of the parity matrix are updated.

## Inputs

- `docs/port-plan.md` — §7 M1 gate
- `docs/parity-matrix.md`

## Outputs

- `docs/parity-matrix.md` — modify: sections E, G

## Acceptance criteria

- [ ] Each M1 exit criterion has evidence (CI run links per OS) in the progress log.
- [ ] Parity rows in sections E and G show their per-OS status.

## Dependencies

- M1-40 — App commands: app info, sidebar counts, CLI detection
- M1-41 — Session read commands
- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M1-43 — History read command
- M1-44 — Config read commands
- M1-45 — MCP read commands
- M1-46 — Search command
- M1-47 — Recursive watcher with debounce, classification, fallback and data-changed events
- M1-48 — Per-window capability files
- M1-53 — Rust versus Swift golden parity tests
- M1-54 — Cross-OS fixture conformance tests
- M1-55 — Secrets never reach DTOs or debug output
- M1-56 — Core coverage measurement and 80% gate
- M1-57 — Load-time benchmark against the performance budgets
- M1-58 — Cross-OS CI for the Rust workspace

## Owning agent

test-automator

## Reviewer agent

qa-expert, architect-reviewer

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
