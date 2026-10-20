# M3-13 · Hooks, plugins and models screens (read-only)

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Hooks (by event and scope), installed plugins and the models catalog display read-only, including copy-as-JSON for hooks.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Config/HooksListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/PluginsListView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Config/ModelsListView.swift`
- `docs/spec/config-screens.md`

## Outputs

- `src/features/config/hooks/HooksPage.vue` — modify placeholder
- `src/features/config/plugins/PluginsPage.vue` — modify placeholder
- `src/features/config/models/ModelsPage.vue` — modify placeholder
- `src/features/config/hooks/index.ts` — modify: available
- `src/features/config/plugins/index.ts` — modify: available
- `src/features/config/models/index.ts` — modify: available
- `src/features/config/hooks/HooksPage.spec.ts` — create

## Acceptance criteria

- [ ] Hooks group by event with matcher and handler summaries; plugins show enabled state; models show default and project model.
- [ ] No mutating controls yet (M3-47, M3-50).

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

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
