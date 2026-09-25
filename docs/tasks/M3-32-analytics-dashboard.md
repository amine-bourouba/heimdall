# M3-32 · Analytics dashboard, stat cards and CSV export menu

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The dashboard shows the range picker, stat cards, chart cards, a bespoke loading shimmer, the API-only message for subscription users and a CSV export menu.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/AnalyticsDashboardView.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Components/StatCard.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Components/ChartCard.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Components/AnalyticsShimmerView.swift`
- `docs/spec/analytics.md`

## Outputs

- `src/features/analytics/AnalyticsPage.vue` — modify placeholder
- `src/features/analytics/AnalyticsDashboard.vue` — create
- `src/features/analytics/components/{ChartCard,StatCard,AnalyticsShimmer,DateRangePicker}.vue` — create
- `src/features/analytics/index.ts` — modify: available
- `src/features/analytics/AnalyticsDashboard.spec.ts` — create

## Acceptance criteria

- [ ] The CSV menu offers the four kinds and calls `export_analytics_csv`.
- [ ] When all costUSD values are 0, cost areas show the API-only message.

## Dependencies

- M0-10 — Behaviour spec for analytics, exports and the context ring
- M2-06 — Router, navigation history and page placeholders
- M2-07 — Shared components
- M3-31 — Analytics store
- M3-36 — Analytics CSV export and PNG save command

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
