# M3-06 · Session todos and AI summary card

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The conversation view shows the session's todo checklist and its AI summary card (goal, outcome, satisfaction, friction) when facets exist.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/SessionTodosView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionFacetsCard.swift`
- `$POIROT/Poirot/Sources/Models/SessionFacets.swift:50-91`

## Outputs

- `src/features/session/SessionTodos.vue` — modify placeholder
- `src/features/session/SessionFacetsCard.vue` — modify placeholder
- `src/features/session/SessionFacetsCard.spec.ts` — create

## Acceptance criteria

- [ ] Outcome and helpfulness labels match SessionFacets.swift; the card hides when no facets exist.

## Dependencies

- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M2-20 — Virtualised conversation view
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
