# M3-51 · Delete actions for history, sessions and projects

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Users can delete a history entry, clear history older than N days, delete a session and delete a project.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/History/HistoryListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Project/ProjectSessionsView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Session/SessionsNavigationView.swift`

## Outputs

- `src/features/history/HistoryPage.vue` — modify
- `src/features/session/ProjectSessions.vue` — modify
- `src/features/session/SessionsNavigation.vue` — modify

## Acceptance criteria

- [ ] Each action calls its delete command; a `cancelled` result changes nothing in the UI.

## Dependencies

- M2-15 — Projects and sessions navigation list
- M2-16 — Project sessions grid and list
- M3-02 — Prompt history screen
- M3-46 — Delete commands for history, sessions and projects

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
