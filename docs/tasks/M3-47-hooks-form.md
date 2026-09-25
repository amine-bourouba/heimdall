# M3-47 · Hook create/edit form and hook actions

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Hooks can be created, edited, deleted, imported and exported, with the form's ⌘./Return (Ctrl on other OSes) shortcuts.

## Inputs

- `$POIROT/Poirot/Sources/Views/Config/HookFormView.swift`
- `$POIROT/Poirot/Sources/Views/Config/HooksListView.swift`
- `$POIROT/Poirot/Sources/Models/Config/Hook.swift`
- `docs/spec/config-screens.md`

## Outputs

- `src/features/config/hooks/HookForm.vue` — create
- `src/features/config/hooks/HooksPage.vue` — modify: actions
- `src/features/config/hooks/HookForm.spec.ts` — create

## Acceptance criteria

- [ ] Matcher input appears only for events that support it (Hook.swift:52-60); default timeouts are 600 s for command and 30 s for http.

## Dependencies

- M0-09 — Behaviour spec for the configuration screens, forms and MCP wizard
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

`$POIROT` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot`, upstream https://github.com/a7t-ai/poirot at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`). Never modify it. Every other path is relative to the Heimdall repository root.
