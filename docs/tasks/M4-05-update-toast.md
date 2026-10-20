# M4-05 · Check-for-updates flow in the UI

> Milestone M4 · Workstream: Feature screens · Scope: MVP

## Goal

The Check for Updates action shows 'Checking…', then 'New version vX' with an install button, or 'You're up to date vX', through the toast system.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/PoirotApp.swift:55-79`

## Outputs

- `src/features/update/useUpdateCheck.ts` — create
- `src/features/update/useUpdateCheck.spec.ts` — create

## Acceptance criteria

- [ ] The action runs from the macOS menu and the in-app menu through `useAppActions`.
- [ ] The checking toast uses the rotate effect (gated by reduced motion).

## Dependencies

- M2-05 — Prefs, theme, toasts and platform stores
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M4-04 — Update commands

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
