# M3-03 · Plans list and detail

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Saved plans list in grid or list layout and open into a rendered Markdown detail with copy.

## Inputs

- `$POIROT/Poirot/Sources/Views/Plans/PlansListView.swift`
- `$POIROT/Poirot/Sources/Views/Plans/PlanDetailView.swift`

## Outputs

- `src/features/plans/PlansPage.vue` — modify placeholder
- `src/features/plans/PlanDetail.vue` — create
- `src/features/plans/store.ts` — create
- `src/features/plans/store.spec.ts` — create
- `src/features/plans/index.ts` — modify: available

## Acceptance criteria

- [ ] Layout choice persists per screen in `configLayouts`; detail renders through MarkdownView.

## Dependencies

- M1-44 — Config read commands
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
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
