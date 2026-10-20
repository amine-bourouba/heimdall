# M0-06 · Map every PoirotTests case to its Heimdall destination

> Milestone M0 · Workstream: Legacy parity extraction · Scope: Infra

## Goal

Every Swift test case in PoirotTests has one recorded destination: a named Rust test module, a Vitest spec, a Playwright spec, or a drop with its reason.

## Inputs

- `$SWIFT_REF/PoirotTests/` — all 72 files
- `docs/port-plan.md` — section 'Testing strategy'
- `docs/adr/0008-provider-abstraction-removal.md`

## Outputs

- `docs/spec/test-carryover.md` — create

## Acceptance criteria

- [ ] The row count equals the number of `@Test` attributes found by `grep -rho "@Test" $SWIFT_REF/PoirotTests | wc -l`, or the document explains any difference.
- [ ] Each destination names a file path that appears in some task's Outputs in docs/tasks/.
- [ ] Drops are limited to: ClaudeCodeProviderTests (ADR 0008), HighlightedTextTests' AttributedString assertions (covered by Shiki output assertions), *LoadingMock.swift (replaced by IPC mocks), UpdateCheckerTests (replaced by the updater plugin, ADR 0011), and the stale `ScreenshotTests_Usage` skip entry in unit-tests.yml.
- [ ] AnalyticsCSVExporterTests is mapped to Rust (`crates/heimdall-core/src/export/analytics_csv.rs`), and the row records why it departs from the brief's 'analytics tests go to Vitest' rule.

## Dependencies

- None

## Owning agent

legacy-modernizer (consults swift-expert, who only reads the Swift app)

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
