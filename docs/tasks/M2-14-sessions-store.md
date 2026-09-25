# M2-14 · Sessions store

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

A Pinia store owns projects, sessions, selection, transcript paging, tool filter and session search state, and reloads only visible data on `data-changed`.

## Inputs

- `$POIROT/Poirot/Sources/App/AppState.swift:82-120, 240-330`
- `$POIROT/PoirotTests/App/AppStateSessionLoadingTests.swift`
- `$POIROT/PoirotTests/App/AppStateTests.swift`
- `docs/spec/test-carryover.md`
- `docs/ipc-contract.md`
- `$POIROT/PoirotTests/Models/ProjectSortingTests.swift`
- `docs/spec/test-carryover.md`

## Outputs

- `src/features/session/store.ts` — create
- `src/features/session/store.spec.ts` — create

## Acceptance criteria

- [ ] Carried-over AppStateSessionLoadingTests and the session cases of AppStateTests pass.
- [ ] Project sort options match ProjectSortOption (recent activity, name, session count).
- [ ] A `data-changed` event for a project that is not on screen triggers no reload (spec).

## Dependencies

- M1-41 — Session read commands
- M2-03 — Typed IPC client wrapper and test helpers
- M2-05 — Prefs, theme, toasts and platform stores
- M2-06 — Router, navigation history and page placeholders
- M0-06 — Map every PoirotTests case to its Heimdall destination
- M1-59 — M1 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
