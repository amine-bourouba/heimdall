# M3-07 · Per-session file history viewer

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The file-history view lists files edited in a session with their versions and shows a diff between any two versions.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Session/FileHistoryView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/FileVersion.swift`

## Outputs

- `src/features/session/FileHistory.vue` — modify placeholder
- `src/features/session/FileHistory.spec.ts` — create

## Acceptance criteria

- [ ] Diffs come from `diff_file_versions` and render with DiffTable.vue.

## Dependencies

- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M2-19 — Edit diff view
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

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
