# M2-21 · Context-window card and tool timeline

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

The context card shows fill percentage, cache-hit rate, token mix and a per-turn sparkline, and the timeline lists tool calls, both from Rust-computed data.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Session/ConversationContextCard.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Session/SessionTimelineView.swift`
- `$SWIFT_REF/Poirot/Sources/Models/ContextUsage.swift`

## Outputs

- `src/features/session/ConversationContextCard.vue` — create
- `src/features/session/SessionTimeline.vue` — create
- `src/features/session/ConversationContextCard.spec.ts` — create

## Acceptance criteria

- [ ] The sparkline is inline SVG; spec covers the empty-series case.

## Dependencies

- M2-07 — Shared components
- M2-14 — Sessions store

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
