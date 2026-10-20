# M3-60 · Launch-at-login toggle

> Milestone M3 · Workstream: Feature screens · Scope: Later

## Goal

Settings → General offers a launch-at-login toggle whose state is read live from the OS and never stored as a preference.

## Inputs

- `$SWIFT_REF/Poirot/Sources/App/SettingsView.swift:320-350`

## Outputs

- `src/features/settings/SettingsPage.vue` — modify: launch at login

## Acceptance criteria

- [ ] The toggle reflects `isEnabled()` on open; failures show a toast.

## Dependencies

- M3-58 — Tray view and Menu Bar settings tab
- M3-59 — Grant launch-at-login permissions

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
