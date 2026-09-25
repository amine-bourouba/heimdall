# M2-20 · Virtualised conversation view

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

The conversation view virtualises variable-height blocks with TanStack Virtual, pages messages over IPC, and supports tool filter, session search, expand-all and scroll restore.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/SessionDetailView.swift`
- `$POIROT/Poirot/Sources/Views/Session/SessionSkeletonView.swift`
- `docs/spec/session-views.md`
- `docs/port-plan.md` — §9 Performance budgets

## Outputs

- `src/features/session/SessionDetail.vue` — create
- `src/features/session/SessionSkeleton.vue` — create
- `src/features/session/SessionDetail.spec.ts` — create
- `src/features/session/SessionFacetsCard.vue` — create: placeholder
- `src/features/session/SessionTodos.vue` — create: placeholder
- `src/features/session/FileHistory.vue` — create: placeholder

## Acceptance criteria

- [ ] Blocks are measured with `measureElement` and re-measured after expand/collapse and font-scale changes.
- [ ] Messages load in pages of 20 through `get_session_messages` as the viewport nears the end.
- [ ] Mod+T toggles the tool filter with tool-name chips; session search highlights matches.
- [ ] Scroll position is restored on back/forward.

## Dependencies

- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-14 — Sessions store
- M2-17 — Text, thinking and system-content blocks
- M2-18 — Tool-use and tool-result block
- M2-19 — Edit diff view

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

L

## Context

Stub rule (port-plan §5.4): this task creates placeholder files that later tasks own. A placeholder holds only a minimal component that renders nothing or an empty state. The later task replaces it completely.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
