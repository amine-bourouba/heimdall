# M3-12 · Commands, skills and output styles screens (read-only)

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Commands, skills and output styles list and open in detail from global and project scope, read-only.

## Inputs

- `$POIROT/Poirot/Sources/Views/Config/CommandsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/SkillsListView.swift`
- `$POIROT/Poirot/Sources/Views/Config/OutputStylesListView.swift`
- `docs/spec/config-screens.md`

## Outputs

- `src/features/config/commands/CommandsPage.vue` — modify placeholder
- `src/features/config/skills/SkillsPage.vue` — modify placeholder
- `src/features/config/output-styles/OutputStylesPage.vue` — modify placeholder
- `src/features/config/commands/index.ts` — modify: available
- `src/features/config/skills/index.ts` — modify: available
- `src/features/config/output-styles/index.ts` — modify: available
- `src/features/config/commands/CommandsPage.spec.ts` — create

## Acceptance criteria

- [ ] Copy `/name` works for commands (CommandsListView.swift:297-300).
- [ ] No create, edit or delete controls yet (M3-50).

## Dependencies

- M3-11 — Shared config list, detail, toolbar, picker and skeleton

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
