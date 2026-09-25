# M0-10 · Behaviour spec for analytics, exports and the context ring

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

A spec captures analytics date ranges, derived series, formatters, the heatmap algorithm, CSV formats and the PNG snapshot layout.

## Inputs

- `$POIROT/Poirot/Sources/Views/Analytics/AnalyticsViewModel.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/AnalyticsDashboardView.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Helpers/AnalyticsFormatters.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Helpers/AnalyticsColorPalette.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Components/ContributionHeatmap.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsCSVExporter.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsImageExporter.swift`
- `$POIROT/Poirot/Sources/Views/Analytics/Export/AnalyticsSnapshotView.swift`
- `$POIROT/Poirot/Sources/Models/StatsCache.swift`
- `$POIROT/Poirot/Sources/Models/ContextUsage.swift`

## Outputs

- `docs/spec/analytics.md` — create

## Acceptance criteria

- [ ] Defines the 7d/30d/90d/All/Custom ranges and the reference date rule exactly as the view model computes them.
- [ ] Documents the four CSV formats column by column and the PNG snapshot (forced dark, branded header, 2x scale).
- [ ] Documents the Monday-first ISO-week heatmap grid algorithm and the context-window rule (standard versus extended window from ContextUsage.swift).

## Dependencies

- M0-05 — Exhaustive inventory of every ~/.claude path Poirot reads or writes

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
