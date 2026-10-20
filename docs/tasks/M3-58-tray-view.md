# M3-58 · Tray view and Menu Bar settings tab

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

The tray view shows header stats, a search field and up to 10 recent sessions that open in the main window, and Settings gains the Menu Bar tab.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/MenuBar/MenuBarView.swift`
- `$SWIFT_REF/Poirot/Sources/App/SettingsView.swift`

## Outputs

- `src/features/tray/TrayView.vue` — create
- `src/features/tray/index.ts` — modify: available
- `src/features/tray/TrayView.spec.ts` — create
- `src/features/settings/SettingsPage.vue` — modify: Menu Bar tab

## Acceptance criteria

- [ ] Search calls `get_tray_snapshot(query)`; selecting a session calls `open_in_main_window`.
- [ ] `tray/index.ts` reports availability so the 'Toggle Menu Bar Icon' quick action appears.

## Dependencies

- M3-16 — Settings screen
- M3-30 — Analytics and tray commands
- M3-57 — Separate frontend entry for the tray window

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

M

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
