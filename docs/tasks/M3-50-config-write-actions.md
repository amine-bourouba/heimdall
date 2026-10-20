# M3-50 · Write actions on plugins, models, commands, skills and output styles

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Plugins can be enabled, disabled and removed; default and project models can be set; commands, skills and output styles can be created from templates and deleted.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/PluginsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ModelsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/CommandsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/SkillsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/OutputStylesListView.swift`

## Outputs

- `src/features/config/plugins/PluginsPage.vue` — modify
- `src/features/config/models/ModelsPage.vue` — modify
- `src/features/config/commands/CommandsPage.vue` — modify
- `src/features/config/skills/SkillsPage.vue` — modify
- `src/features/config/output-styles/OutputStylesPage.vue` — modify
- `src/features/config/plugins/PluginsPage.spec.ts` — create

## Acceptance criteria

- [ ] Each action calls its contract command and shows a toast on success or a clear error on failure.

## Dependencies

- M3-12 — Commands, skills and output styles screens (read-only)
- M3-13 — Hooks, plugins and models screens (read-only)
- M3-44 — Config write commands with native confirmations

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
