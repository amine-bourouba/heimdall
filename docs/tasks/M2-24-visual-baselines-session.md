# M2-24 · Visual baselines for the session screens

> Milestone M2 · Workstream: Testing · Scope: MVP

## Goal

Committed Linux Playwright baselines cover the sidebar, sessions navigation, project sessions, conversation view and all block types.

## Inputs

- `docs/spec/test-carryover.md` — Screenshots rows
- `$SWIFT_REF/PoirotTests/Screenshots/ScreenshotTests_{Sidebar,SessionDetail,ToolBlocks,ThinkingBlocks,EditDiff,FullViews,Standalone,Windows}.swift`

## Outputs

- `e2e/visual/session.spec.ts` — create
- `e2e/visual/session.spec.ts-snapshots/**` — create: Linux baselines

## Acceptance criteria

- [ ] Each listed Swift screenshot suite has a matching Playwright test.
- [ ] The conversation view is captured in 3 themes × 2 font scales.

## Dependencies

- M2-15 — Projects and sessions navigation list
- M2-16 — Project sessions grid and list
- M2-17 — Text, thinking and system-content blocks
- M2-18 — Tool-use and tool-result block
- M2-19 — Edit diff view
- M2-20 — Virtualised conversation view
- M2-21 — Context-window card and tool timeline
- M2-22 — Session toolbar
- M2-23 — Playwright and real-app E2E harness

## Owning agent

test-automator

## Reviewer agent

qa-expert

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
