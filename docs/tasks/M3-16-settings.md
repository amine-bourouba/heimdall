# M3-16 · Settings screen

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

An in-app settings screen (Mod+,) covers the General, Appearance, Viewer and Memory tabs with OS-appropriate editor and terminal choices.

## Inputs

- `$POIROT/Poirot/Sources/App/SettingsView.swift`
- `$POIROT/Poirot/Sources/Views/Components/ThemePicker.swift`
- `docs/adr/0007-preferences-storage-and-migration.md`
- `docs/port-plan.md` — decision P-6

## Outputs

- `src/features/settings/SettingsPage.vue` — modify placeholder
- `src/features/settings/ThemePicker.vue` — create
- `src/features/settings/SettingsPage.spec.ts` — create
- `src/features/settings/index.ts` — modify: available

## Acceptance criteria

- [ ] Every preference in parity-matrix section I except `showMenuBarIcon` and launch-at-login is editable here.
- [ ] Editor and terminal lists show only options valid for the running OS.
- [ ] The Menu Bar tab and launch-at-login toggle come later (M3-58, M3-60).

## Dependencies

- M1-39 — Preferences storage and preference commands
- M2-05 — Prefs, theme, toasts and platform stores
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
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
