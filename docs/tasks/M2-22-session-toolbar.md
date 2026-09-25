# M2-22 · Session toolbar

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

The session toolbar offers copy resume command, copy session id, file-history toggle, debug-log opener (when a log exists), export, and expand-all.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/SessionToolbarContent.swift`
- `docs/spec/session-views.md`

## Outputs

- `src/features/session/SessionToolbar.vue` — create
- `src/features/session/SessionToolbar.spec.ts` — create
- `src/features/session/DebugLogSheet.vue` — create: placeholder
- `src/features/session/ExportOptionsDialog.vue` — create: placeholder

## Acceptance criteria

- [ ] The copied resume command is `claude --resume <sessionId>`.
- [ ] Terminal and reveal buttons come later (M3-55).

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-14 — Sessions store

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## Context

Stub rule (port-plan §5.4): this task creates placeholder files that later tasks own. A placeholder holds only a minimal component that renders nothing or an empty state. The later task replaces it completely.

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
