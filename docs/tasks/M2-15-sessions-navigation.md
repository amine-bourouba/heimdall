# M2-15 · Projects and sessions navigation list

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

The sessions navigation screen lists projects with sort, search, the agent-session toggle, skeleton and empty states, fully keyboard navigable.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/SessionsNavigationView.swift`
- `docs/spec/session-views.md`

## Outputs

- `src/features/session/SessionsPage.vue` — modify placeholder
- `src/features/session/SessionsNavigation.vue` — create
- `src/features/session/ProjectRow.vue` — create
- `src/features/session/SessionsNavigation.spec.ts` — create
- `src/features/session/index.ts` — modify: available

## Acceptance criteria

- [ ] Spec covers sorting, search filtering, the `showAgentSessions` toggle and keyboard selection.
- [ ] `session/index.ts` sets `available: true`.

## Dependencies

- M2-07 — Shared components
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-14 — Sessions store
- M0-08 — Behaviour spec for the session browser and conversation view

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
