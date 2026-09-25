# M2-19 · Edit diff view

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

Edit-tool diffs render from `diff_lines` (computed in Rust), with a gutter sized to the largest line number, the 5000-line cap message and copy.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/EditDiffView.swift`
- `docs/spec/session-views.md`

## Outputs

- `src/features/session/blocks/EditDiff.vue` — create
- `src/features/session/blocks/DiffTable.vue` — create
- `src/features/session/blocks/EditDiff.spec.ts` — create

## Acceptance criteria

- [ ] DiffTable.vue is reusable by the file-history view.
- [ ] Spec covers the truncation message and gutter width.
- [ ] Save-as-patch and copy-as-image come later (M3-10, M3-39).

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M1-42 — Session data commands: todos, file history, diffs, debug log, facets
- M2-07 — Shared components
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
