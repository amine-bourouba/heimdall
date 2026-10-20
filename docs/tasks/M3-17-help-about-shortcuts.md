# M3-17 · Help, About and the full shortcut sheet

> Milestone M3 · Workstream: Feature screens · Scope: MVP

## Goal

Help, About and a platform-aware shortcut sheet listing every shortcut from the registry exist as in-app views.

## Inputs

- `$SWIFT_REF/Poirot/Sources/Views/Help/HelpView.swift`
- `$SWIFT_REF/Poirot/Sources/Views/Help/ShortcutHelpView.swift`
- `$SWIFT_REF/Poirot/Sources/App/AboutView.swift`
- `docs/adr/0011-app-identifier-and-release-repo.md`
- `docs/port-plan.md` — decision P-7

## Outputs

- `src/features/help/HelpPage.vue` — modify placeholder
- `src/features/help/ShortcutHelp.vue` — modify placeholder
- `src/features/about/AboutDialog.vue` — modify placeholder
- `src/features/help/index.ts` — modify: available
- `src/features/about/index.ts` — modify: available
- `src/features/help/ShortcutHelp.spec.ts` — create

## Acceptance criteria

- [ ] The shortcut sheet lists every entry of the `useShortcuts` registry with platform-aware labels (full list, decision P-7).
- [ ] About shows version and identifier, links to https://github.com/amine-bourouba/heimdall through `open_external_url`, and credits the Swift reference app under its MIT notice; upstream maintainer's Sponsors and social links are not carried over.

## Dependencies

- M1-40 — App commands: app info, sidebar counts, CLI detection
- M2-08 — Keyboard navigation, global shortcuts and app actions
- M2-09 — App shell: split view, sidebar with live counts, toolbar, overlays
- M2-11 — open_external_url command
- M2-30 — M2 exit gate review

## Owning agent

frontend-developer (with typescript-pro for typed IPC)

## Reviewer agent

code-reviewer, accessibility-tester

## Size

S

## When you finish

Append one entry to `docs/progress-log.md` (task ID, date, files touched, evidence for each acceptance criterion, follow-ups). Never edit another entry.

`$SWIFT_REF` is the read-only Swift reference checkout (`C:\Users\pc\Desktop\tech\poirot` on this machine -- see ADR 0001 for provenance), pinned at commit `a29dd6ee0e0133ef26bd067c164c66b160e13d86`. Never modify it. Every other path is relative to the Heimdall repository root.
