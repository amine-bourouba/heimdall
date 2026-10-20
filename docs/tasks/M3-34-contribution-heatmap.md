# M3-34 · Contribution heatmap

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The contribution heatmap is bespoke markup that builds a Monday-first ISO-week grid as ContributionHeatmap does, with grid ARIA semantics.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Analytics/Components/ContributionHeatmap.swift`
- `docs/spec/analytics.md`

## Outputs

- `src/features/analytics/components/ContributionHeatmap.vue` — create
- `src/features/analytics/components/ContributionHeatmap.spec.ts` — create

## Acceptance criteria

- [ ] Spec checks week alignment across a year boundary and cell intensity buckets.

## Dependencies

- M0-10 — Behaviour spec for analytics, exports and the context ring
- M3-31 — Analytics store

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
