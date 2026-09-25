# M2-18 · Tool-use and tool-result block

> Milestone M2 · Workstream: Feature screens · Scope: MVP

## Goal

The tool block reproduces ToolBlockView: 80-line truncation with show-all, a per-tool icon, the bash command line, result pairing, and copy and copy-as-Markdown.

## Inputs

- `$POIROT/Poirot/Sources/Views/Session/ToolBlockView.swift`
- `docs/spec/session-views.md`
- `docs/design/icon-map.md`

## Outputs

- `src/features/session/blocks/ToolBlock.vue` — create
- `src/features/session/blocks/ToolBlock.spec.ts` — create

## Acceptance criteria

- [ ] Spec covers truncation at 80 lines, show-all, `parseMarkdownInResults`, `wrapCodeLines` and copy.
- [ ] No open-in-editor or terminal buttons yet; M3-55 adds them.

## Dependencies

- M2-04 — Lucide icon component, motion utilities and the SF Symbol map
- M2-07 — Shared components
- M2-10 — renderMarkdown, Shiki highlighting and the single v-html component
- M0-08 — Behaviour spec for the session browser and conversation view
- M1-59 — M1 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
