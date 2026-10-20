# M2-17 · Text, thinking and system-content blocks

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

Text, thinking and system-content blocks render through MarkdownView with the thinking block's 50-line truncation, default expansion from `autoExpandBlocks`, and a gated pulse while expanded.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Session/ThinkingBlockView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Session/SystemContentView.swift`
- `docs/spec/session-views.md`

## Outputs

- `src/features/session/blocks/TextBlock.vue` — create
- `src/features/session/blocks/ThinkingBlock.vue` — create
- `src/features/session/blocks/SystemContent.vue` — create
- `src/features/session/blocks/ThinkingBlock.spec.ts` — create

## Acceptance criteria

- [ ] Spec covers truncation at 50 lines, expand, copy, and no animation under reduced motion.

## Dependencies

- M0-08 — Behaviour spec for the session browser and conversation view
- M2-04 — Lucide icon component, motion utilities and the SF Symbol map
- M2-07 — Shared components
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
- M1-59 — M1 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
