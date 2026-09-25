# M3-08 · Debug log viewer

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The debug-log sheet pages through a session's debug log with a level filter, search and copy, and refreshes on `data-changed`.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/DebugLogView.swift`

## Outputs

- `src/features/session/DebugLogSheet.vue` — modify placeholder
- `src/features/session/DebugLogSheet.spec.ts` — create

## Acceptance criteria

- [ ] Paging uses `get_debug_log_page`; the level filter covers DEBUG, WARN and ERROR.

## Dependencies

- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M2-22 — Session toolbar
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
