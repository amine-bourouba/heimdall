# M3-10 · Export options dialog and save-as-patch

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

The export dialog offers the four toggles with copy and save, and the edit diff gains a Save as patch action.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/ExportOptionsView.swift`
- `$POIROT/Poirot/Sources/Views/Session/EditDiffView.swift`

## Outputs

- `src/features/session/ExportOptionsDialog.vue` — modify placeholder
- `src/features/session/blocks/EditDiff.vue` — modify: save as patch
- `src/features/session/ExportOptionsDialog.spec.ts` — create

## Acceptance criteria

- [ ] Spec covers each toggle being passed to `export_session`.

## Dependencies

- M2-19 — Edit diff view
- M2-22 — Session toolbar
- M3-09 — Session export commands
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
