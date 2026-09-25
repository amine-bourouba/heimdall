# M3-31 · Analytics store

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

A Pinia store turns the stats DTO into range-filtered series, heatmap data, cost breakdown and totals exactly as AnalyticsViewModel does.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/AnalyticsViewModel.swift`
- `$POIROT/PoirotTests/Views/AnalyticsViewModelTests.swift`
- `docs/spec/analytics.md`
- `$POIROT/PoirotTests/Models/StatsCacheTests.swift` — longestSessionFormatted case
- `docs/spec/test-carryover.md`

## Outputs

- `src/features/analytics/store.ts` — create
- `src/features/analytics/store.spec.ts` — create

## Acceptance criteria

- [ ] Carried-over AnalyticsViewModelTests pass with a fixed clock.

## Dependencies

- M0-10 — Behaviour spec for analytics, exports and the context ring
- M2-05 — Prefs, theme, toasts and platform stores
- M3-30 — Analytics and tray commands
- M0-06 — Map every PoirotTests case to its Heimdall destination
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
