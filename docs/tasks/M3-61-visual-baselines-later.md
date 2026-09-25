# M3-61 · Visual baselines for later-scope screens

> Milestone M3 · Workstream: Testing · Scope: Later

## Goal

Committed Linux baselines cover analytics, config forms, the MCP wizard and the tray view.

## Inputs

- `$POIROT/PoirotTests/Screenshots/ScreenshotTests_Analytics.swift`

## Outputs

- `e2e/visual/later-screens.spec.ts` — create
- `e2e/visual/later-screens.spec.ts-snapshots/**` — create

## Acceptance criteria

- [ ] Analytics is captured with data and in its loading state.

## Dependencies

- M2-23 — Playwright and real-app E2E harness
- M3-32 — Analytics dashboard, stat cards and CSV export menu
- M3-33 — The six analytics charts
- M3-34 — Contribution heatmap
- M3-47 — Hook create/edit form and hook actions
- M3-48 — MCP setup wizard and server actions
- M3-49 — Sub-agent form, import and export
- M3-58 — Tray view and Menu Bar settings tab

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
