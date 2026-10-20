# M3-39 · Analytics PNG export and diff copy-as-image

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The analytics snapshot exports as a branded, forced-dark, 2x PNG, and the edit diff copies as an image, using the approach chosen in the spike.

## Inputs

- `docs/spikes/png-export.md`
- `$SWIFT_REF/Poirot/Sources/Views/Analytics/Export/AnalyticsSnapshotView.swift`

## Outputs

- `src/features/analytics/export/AnalyticsSnapshot.vue` — create
- `src/lib/export/renderPng.ts` — create
- `src/features/session/blocks/EditDiff.vue` — modify: copy as image

## Acceptance criteria

- [ ] The PNG is saved through `save_png`; copy-as-image uses the clipboard-manager image permission.
- [ ] Output matches the spike's acceptance images on all three OSes.

## Dependencies

- M3-10 — Export options dialog and save-as-patch
- M3-32 — Analytics dashboard, stat cards and CSV export menu
- M3-36 — Analytics CSV export and PNG save command
- M3-38 — Add the PNG export dependency

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
