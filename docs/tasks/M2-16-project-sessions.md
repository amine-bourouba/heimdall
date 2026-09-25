# M2-16 · Project sessions grid and list

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

A project's sessions show as a grid or list (the `sessionLayout` preference) with sub-agent badges and marquee titles.

## Inputs

- `$POIROT/Poirot/Sources/Views/Project/ProjectSessionsView.swift`
- `docs/spec/session-views.md`

## Outputs

- `src/features/session/ProjectSessions.vue` — create
- `src/features/session/SessionCard.vue` — create
- `src/features/session/ProjectSessions.spec.ts` — create

## Acceptance criteria

- [ ] Spec covers both layouts and sub-agent sessions.

## Dependencies

- M2-07 — Shared components
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
