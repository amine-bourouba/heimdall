# M3-33 · The six analytics charts

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The six Swift charts exist as vue-echarts components inside ChartCard, themed from CSS variables, with every data-derived tooltip string escaped.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/Charts/*.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Helpers/AnalyticsFormatters.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Helpers/AnalyticsColorPalette.swift`
- `docs/adr/0010-webview-trust-boundary.md` — Rendering

## Outputs

- `src/features/analytics/charts/{CostBreakdownChart,DailyActivityChart,HourlyActivityChart,ModelUsageChart,TokenUsageOverTimeChart,ToolCallsOverTimeChart}.vue` — create
- `src/features/analytics/charts/echartsTheme.ts` — create
- `src/features/analytics/charts/formatters.ts` — create
- `src/features/analytics/charts/formatters.spec.ts` — create

## Acceptance criteria

- [ ] Formatters port AnalyticsFormatters.swift with carried-over expectations.
- [ ] Every tooltip formatter passes data-derived strings through `echarts.format.encodeHTML`.
- [ ] Charts re-theme when the theme store changes.

## Dependencies

- M3-32 — Analytics dashboard, stat cards and CSV export menu

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester, security-auditor

## Size

L

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
