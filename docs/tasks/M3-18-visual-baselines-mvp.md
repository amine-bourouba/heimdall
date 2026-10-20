# M3-18 · Visual baselines for the remaining MVP screens

> Milestone M3 · Workstream: Testing · Scope: MVP

## Goal

Committed Linux Playwright baselines cover history, plans, memory, todos, debug log, facets, file history, read-only config screens, search, onboarding and settings.

## Inputs

- `$SWIFT_REF/PoirotTests/Screenshots/ScreenshotTests_{History,Plans,Memory,Todos,DebugLog,Facets,FileHistory,ConfigScreens}.swift`
- `docs/spec/test-carryover.md`

## Outputs

- `e2e/visual/mvp-screens.spec.ts` — create
- `e2e/visual/mvp-screens.spec.ts-snapshots/**` — create

## Acceptance criteria

- [ ] Each listed Swift screenshot suite has a matching Playwright test.

## Dependencies

- M2-23 — Playwright and real-app E2E harness
- M3-01 — Universal search overlay
- M3-02 — Prompt history screen
- M3-03 — Plans list and detail
- M3-04 — Memory list and detail
- M3-05 — Cross-project TODOs overview
- M3-06 — Session todos and AI summary card
- M3-07 — Per-session file history viewer
- M3-08 — Debug log viewer
- M3-12 — Commands, skills and output styles screens (read-only)
- M3-13 — Hooks, plugins and models screens (read-only)
- M3-14 — MCP servers and sub-agents screens (read-only)
- M3-15 — Onboarding flow
- M3-16 — Settings screen

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
