# M3-05 · Cross-project TODOs overview

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The TODOs screen shows every session's todo list across projects, with project names from `list_projects` and no path decoding in the UI.

## Inputs

- `$POIROT/Poirot/Sources/Views/Todos/TodosOverviewView.swift`

## Outputs

- `src/features/todos/TodosPage.vue` — modify placeholder
- `src/features/todos/store.ts` — create
- `src/features/todos/store.spec.ts` — create
- `src/features/todos/index.ts` — modify: available

## Acceptance criteria

- [ ] Spec covers grouping by project and status counts; no string replaces `-` with `/` anywhere in the feature.

## Dependencies

- M1-41 — Session read commands
- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
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
